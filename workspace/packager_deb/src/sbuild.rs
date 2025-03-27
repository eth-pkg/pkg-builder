use crate::configs::pkg_config::PkgConfig;
use crate::configs::pkg_config_verify::PkgVerifyConfig;
use crate::misc::build_pipeline::BuildError;
use crate::misc::distribution::DistributionTrait;
use crate::misc::utils::{calculate_sha1, ensure_parent_dir, remove_file_or_directory};
use crate::sbuild_args::SbuildArgs;
use crate::tools::autopkgtest_tool::AutopkgtestTool;
use crate::tools::lintian_tool::LintianTool;
use crate::tools::piuparts_tool::PiupartsTool;
use crate::tools::sbuild_tool::SbuildTool;
use crate::tools::tool_runner::ToolRunner;
use debian::autopkgtest::AutopkgtestError;
use debian::autopkgtest_image::AutopkgtestImageError;
use debian::execute::Execute;
use debian::lintian::LintianError;
use debian::piuparts::PiupartsError;
use debian::sbuild::SbuildCmdError;
use debian::sbuild_create_chroot::{SbuildCreateChroot, SbuildCreateChrootError};
use log::info;
use rand::random;
use std::path::{Path, PathBuf};
use std::{env, fs};
use thiserror::Error;

pub struct Sbuild {
    args: SbuildArgs,
}

impl Sbuild {
    pub fn new(config: PkgConfig, config_root: PathBuf) -> Self {
        let args = SbuildArgs::new(config, config_root);

        Sbuild { args }
    }
}

#[derive(Debug, Error)]
pub enum SbuildError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    SbuildCreateChrootError(#[from] SbuildCreateChrootError),

    #[error(transparent)]
    AutopkgtestImageError(#[from] AutopkgtestImageError),

    #[error(transparent)]
    PiupartsError(#[from] PiupartsError),

    #[error(transparent)]
    LintianError(#[from] LintianError),

    #[error(transparent)]
    AutopkgtestError(#[from] AutopkgtestError),

    #[error(transparent)]
    SbuildCmdError(#[from] SbuildCmdError),

    #[error("Verification error: {0}")]
    VerificationError(String),

    #[error("{0}")]
    GenericError(String),

    #[error(transparent)]
    SemverError(#[from] cargo_metadata::semver::Error),

    #[error(transparent)]
    BuildError(#[from] BuildError),
}

impl Sbuild {
    pub fn run_env_clean(&self) -> Result<(), SbuildError> {
        let cache_file = self.args.get_cache_file();
        info!("Cleaning cached build: {:?}", cache_file);
        remove_file_or_directory(&cache_file, false)?;
        Ok(())
    }

    pub fn run_env_create(&self) -> Result<(), SbuildError> {
        let temp_dir = env::temp_dir().join(format!("temp_{}", random::<u32>()));
        fs::create_dir(&temp_dir)?;

        let cache_file = self.args.get_cache_file();
        ensure_parent_dir(&cache_file)?;

        let codename = &self.args.codename();

        SbuildCreateChroot::new()
            .chroot_mode("unshare")
            .make_tarball()
            .cache_file(&cache_file)
            .codename(codename)
            .temp_dir(&temp_dir)
            .repo_url(codename.get_repo_url())
            .execute()?;

        Ok(())
    }

    pub fn run_package(&self) -> Result<(), SbuildError> {
        let sbuild_version = self.args.sbuild_version();
        let codename = self.args.codename();
        let cache_file = self.args.get_cache_file();
        let build_chroot_setup_commands = self.args.build_chroot_setup_commands();
        let run_lintian = self.args.run_lintian();
        let run_piuparts = self.args.run_piuparts();
        let run_autopkgtest = self.args.run_autopkgtests();
        let build_files_dir = self.args.build_files_dir();
        let package_type = self.args.package_type();
        let context = self.args.context();

        let tool = SbuildTool::new(
            sbuild_version,
            codename.clone(),
            cache_file,
            build_chroot_setup_commands,
            run_lintian,
            build_files_dir.clone(),
            package_type,
            context,
        );
        ToolRunner::new().run_tool(tool)?;
        if run_piuparts {
            self.run_piuparts()?;
        }
        if run_autopkgtest {
            self.run_autopkgtests()?;
        }
        Ok(())
    }

    pub fn run_verify(
        &self,
        verify_config: PkgVerifyConfig,
        no_package: bool,
    ) -> Result<(), SbuildError> {
        if !no_package {
            self.run_package()?;
        }
        let output_dir =
            Path::new(self.args.build_files_dir())
                .parent()
                .ok_or(SbuildError::GenericError(
                    "Invalid build files dir".to_string(),
                ))?;

        let errors: Vec<_> = verify_config
            .verify
            .package_hash
            .iter()
            .filter_map(|output| {
                let file_path = output_dir.join(&output.name);
                if !file_path.exists() {
                    return Some(format!("Verification file missing: {}", output.name));
                }

                let buffer = match std::fs::read(&file_path) {
                    Ok(buf) => buf,
                    Err(_) => return Some(format!("Failed to read file: {}", output.name)),
                };

                let actual_sha1 = match calculate_sha1(&buffer) {
                    Ok(hash) => hash,
                    Err(_) => {
                        return Some(format!("Failed to calculate hash for: {}", output.name))
                    }
                };

                (actual_sha1 != output.hash).then(|| {
                    format!(
                        "SHA1 mismatch for {}: expected {}, got {}",
                        output.name, output.hash, actual_sha1
                    )
                })
            })
            .collect();

        if errors.is_empty() {
            info!("Verification successful!");
            Ok(())
        } else {
            // Convert the error collection to a proper error
            Err(SbuildError::VerificationError(errors.join("; ")))
        }
    }

    pub fn run_lintian(&self) -> Result<(), SbuildError> {
        let lintian_version = self.args.lintian_version();
        let changes_file = self.args.get_changes_file();
        let codename = self.args.codename().clone();

        let tool = LintianTool::new(lintian_version, changes_file, codename);
        ToolRunner::new().run_tool(tool)
    }

    pub fn run_piuparts(&self) -> Result<(), SbuildError> {
        let piuparts_version = self.args.piuparts_version();
        let codename = self.args.codename().clone();
        let deb_dir = self.args.get_deb_dir();
        let deb_name = self.args.get_deb_name();
        let language_env = self.args.language_env();

        let tool = PiupartsTool::new(piuparts_version, codename, deb_dir, deb_name, language_env);
        ToolRunner::new().run_tool(tool)
    }

    pub fn run_autopkgtests(&self) -> Result<(), SbuildError> {
        let autopkgtest_version = self.args.autopkgtest_version();
        let changes_file = self.args.get_changes_file();
        let codename = self.args.codename().clone();
        let deb_dir = self.args.get_deb_dir();
        let test_deps = self.args.get_test_deps_not_in_debian();
        let cache_dir = self.args.cache_dir().clone();
        let arch = self.args.arch();

        let tool = AutopkgtestTool::new(
            autopkgtest_version,
            changes_file,
            codename,
            deb_dir,
            test_deps,
            cache_dir,
            arch,
        );
        ToolRunner::new().run_tool(tool)
    }
}

#[cfg(test)]
mod tests {

    use crate::configs::autopkg_version::AutopkgVersion;
    use crate::configs::pkg_config::{
        BuildEnv, DefaultPackageTypeConfig, LanguageEnv, PackageFields, PackageType,
    };

    use super::*;
    use env_logger::Env;
    use std::fs::{create_dir_all, File};
    use std::path::{Path, PathBuf};
    use std::sync::Once;
    use tempfile::tempdir;
    use types::distribution::Distribution;
    use types::version::Version;

    static INIT: Once = Once::new();

    // Initialize logger once for all tests
    fn setup() {
        INIT.call_once(|| {
            env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
        });
    }

    fn create_base_config() -> (PkgConfig, String, PathBuf) {
        let sbuild_cache_dir = tempdir().unwrap().path().to_path_buf();
        let config = PkgConfig {
            package_fields: PackageFields {
                spec_file: "".into(),
                package_name: "".into(),
                version_number: Version::try_from("1.0.0").unwrap(),
                revision_number: "".into(),
                homepage: "".into(),
            },
            package_type: PackageType::Default(DefaultPackageTypeConfig {
                tarball_url: "test.tar.gz".into(),
                tarball_hash: Some("".into()),
                language_env: LanguageEnv::C,
            }),
            build_env: BuildEnv {
                codename: Distribution::bookworm(),
                arch: "amd64".into(),
                pkg_builder_version: Version::try_from("1.0.0").unwrap(),
                debcrafter_version: "1.0.0".into(),
                sbuild_cache_dir: Some(sbuild_cache_dir.clone()),
                docker: None,
                run_lintian: None,
                run_piuparts: None,
                run_autopkgtest: None,
                lintian_version: Version::try_from("1.0.0").unwrap(),
                piuparts_version: Version::try_from("1.0.0").unwrap(),
                autopkgtest_version: AutopkgVersion::try_from("2.5").unwrap(),
                sbuild_version: Version::try_from("1.0.0").unwrap(),
                workdir: PathBuf::from(""),
            },
        };
        let build_files_dir = tempdir().unwrap().path().to_str().unwrap().to_string();

        (config, build_files_dir, sbuild_cache_dir)
    }

    #[test]
    fn test_clean_when_file_missing() {
        setup();
        let (config, build_files_dir, _) = create_base_config();
        let args = SbuildArgs::new(config, build_files_dir.into());
        let build_env = Sbuild { args: args.clone() };

        let result = build_env.run_env_clean();
        let cache_file = args.get_cache_file();

        assert!(result.is_ok());
        assert!(!Path::new(&cache_file).exists());
    }

    #[test]
    fn test_clean_with_existing_file() {
        setup();
        let (config, build_files_dir, cache_dir) = create_base_config();

        let args = SbuildArgs::new(config, build_files_dir.into());
        let build_env = Sbuild { args: args.clone() };
        let cache_file = args.get_cache_file();

        create_dir_all(cache_dir).unwrap();
        File::create(&cache_file).unwrap();
        assert!(Path::new(&cache_file).exists());

        let result = build_env.run_env_clean();
        assert!(result.is_ok());
        assert!(!Path::new(&cache_file).exists());
    }

    #[test]
    #[ignore = "Only run on CI"]
    fn test_create_environment() {
        setup();
        let (config, build_files_dir, _) = create_base_config();
        let args = SbuildArgs::new(config, build_files_dir.into());
        let build_env = Sbuild { args: args.clone() };
        let cache_file = args.get_cache_file();

        build_env.run_env_clean().unwrap();
        assert!(!Path::new(&cache_file).exists());

        let result = build_env.run_env_create();
        assert!(result.is_ok());
        assert!(Path::new(&cache_file).exists());
    }
}
