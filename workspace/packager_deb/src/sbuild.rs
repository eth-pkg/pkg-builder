use crate::distribution::DistributionTrait;
use crate::pkg_config::PkgConfig;
use crate::pkg_config_verify::PkgVerifyConfig;
use crate::tools::autopkgtest_tool::AutopkgtestTool;
use crate::tools::lintian_tool::LintianTool;
use crate::tools::piuparts_tool::PiupartsTool;
use crate::tools::tool_runner::ToolRunner;
use crate::utils::{
    calculate_sha1, ensure_parent_dir, remove_file_or_directory,
};
use debian::autopkgtest::AutopkgtestError;
use debian::autopkgtest_image::AutopkgtestImageError;
use debian::execute::Execute;
use debian::lintian::LintianError;
use debian::piuparts::PiupartsError;
use debian::sbuild::{SbuildBuilder, SbuildCmdError};
use debian::sbuild_create_chroot::{SbuildCreateChroot, SbuildCreateChrootError};
use log::info;
use rand::random;
use std::path::{Path, PathBuf};
use std::{env, fs};
use thiserror::Error;

pub struct Sbuild {
    pub(crate) config: PkgConfig,
    pub(crate) build_files_dir: String,
    pub(crate) cache_dir: PathBuf,
}

impl Sbuild {
    pub fn new(config: PkgConfig, build_files_dir: String) -> Self {
        let cache_dir = config
            .build_env
            .sbuild_cache_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from("~/.cache/sbuild"));

        Self {
            cache_dir,
            config,
            build_files_dir,
        }
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
    // Add an error variant for verification errors
    #[error("Verification error: {0}")]
    VerificationError(String),
    // Add a generic error variant
    #[error("{0}")]
    GenericError(String),

    #[error(transparent)]
    SemverError(#[from] cargo_metadata::semver::Error),
}

impl Sbuild {
    pub fn clean(&self) -> Result<(), SbuildError> {
        let cache_file = self.get_cache_file();
        info!("Cleaning cached build: {}", cache_file);
        remove_file_or_directory(&cache_file, false)?;
        Ok(())
    }

    pub fn create(&self) -> Result<(), SbuildError> {
        let temp_dir = env::temp_dir().join(format!("temp_{}", random::<u32>()));
        fs::create_dir(&temp_dir)?;

        let cache_file = self.get_cache_file();
        ensure_parent_dir(&cache_file)?;

        let codename = &self.config.build_env.codename;

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

    pub fn package(&self) -> Result<(), SbuildError> {
        let codename = &self.config.build_env.codename;
        let cache_file = self.get_cache_file();
        let build_chroot_setup_commands = self.build_chroot_setup_commands();
        let run_lintian = self.config.build_env.run_lintian.unwrap_or(false);
        let run_piuparts = self.config.build_env.run_piuparts.unwrap_or(false);
        let run_autopkgtest = self.config.build_env.run_autopkgtest.unwrap_or(false);
        let builder = SbuildBuilder::new()
            .distribution(codename)
            .build_arch_all()
            .build_source()
            .cache_file(&cache_file)
            .verbose()
            .chroot_mode_unshare()
            .setup_commands(&build_chroot_setup_commands)
            .no_run_piuparts()
            .no_apt_upgrades()
            .run_lintian(run_lintian)
            .no_run_autopkgtest()
            .working_dir(Path::new(&self.build_files_dir));

        builder.execute()?;

        if run_piuparts {
            self.run_piuparts()?;
        }
        if run_autopkgtest {
            self.run_autopkgtests()?;
        }
        Ok(())
    }

    pub fn verify(&self, verify_config: PkgVerifyConfig) -> Result<(), SbuildError> {
        let output_dir =
            Path::new(&self.build_files_dir)
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
        let tool = LintianTool::new(
            self.config.build_env.lintian_version.clone(),
            self.get_changes_file(),
            self.config.build_env.codename.clone(),
        );
        ToolRunner::new().run_tool(tool)
    }

    pub fn run_piuparts(&self) -> Result<(), SbuildError> {
        let tool = PiupartsTool::new(
            self.config.build_env.lintian_version.clone(),
            self.config.build_env.codename.clone(),
            self.get_deb_dir(),
            self.get_deb_name(),
            self.language_env()
        );
        ToolRunner::new().run_tool(tool)
    }

    pub fn run_autopkgtests(&self) -> Result<(), SbuildError> {
        let tool = AutopkgtestTool::new(
            self.config.build_env.lintian_version.clone(),
            self.get_changes_file(),
            self.config.build_env.codename.clone(),
            self.get_deb_dir(),
            self.get_test_deps_not_in_debian(),
            self.cache_dir.clone(),
            self.config.build_env.arch.clone()
        );
        ToolRunner::new().run_tool(tool)
    }
}

#[cfg(test)]
mod tests {
    use crate::pkg_config::{
        BuildEnv, DefaultPackageTypeConfig, LanguageEnv, PackageFields, PackageType,
    };

    use super::*;
    use env_logger::Env;
    use std::fs::{create_dir_all, File};
    use std::path::Path;
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
                debcrafter_version: Version::try_from("1.0.0").unwrap(),
                sbuild_cache_dir: Some(sbuild_cache_dir.clone()),
                docker: None,
                run_lintian: None,
                run_piuparts: None,
                run_autopkgtest: None,
                lintian_version: Version::try_from("1.0.0").unwrap(),
                piuparts_version: Version::try_from("1.0.0").unwrap(),
                autopkgtest_version: Version::try_from("1.0.0").unwrap(),
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
        let build_env = Sbuild::new(config, build_files_dir);

        let result = build_env.clean();
        let cache_file = build_env.get_cache_file();

        assert!(result.is_ok());
        assert!(!Path::new(&cache_file).exists());
    }

    #[test]
    fn test_clean_with_existing_file() {
        setup();
        let (config, build_files_dir, cache_dir) = create_base_config();

        let build_env = Sbuild::new(config, build_files_dir);
        let cache_file = build_env.get_cache_file();

        create_dir_all(cache_dir).unwrap();
        File::create(&cache_file).unwrap();
        assert!(Path::new(&cache_file).exists());

        let result = build_env.clean();
        assert!(result.is_ok());
        assert!(!Path::new(&cache_file).exists());
    }

    #[test]
    #[ignore = "Only run on CI"]
    fn test_create_environment() {
        setup();
        let (config, build_files_dir, _) = create_base_config();
        let build_env = Sbuild::new(config, build_files_dir);

        build_env.clean().unwrap();
        let cache_file = build_env.get_cache_file();
        assert!(!Path::new(&cache_file).exists());

        let result = build_env.create();
        assert!(result.is_ok());
        assert!(Path::new(&cache_file).exists());
    }
}
