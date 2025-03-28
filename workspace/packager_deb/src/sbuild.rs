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
use std::path::Path;
use std::{env, fs};
use thiserror::Error;

pub struct Sbuild {
    args: SbuildArgs,
}

impl TryFrom<PkgConfig> for Sbuild {
    type Error = SbuildError;

    fn try_from(config: PkgConfig) -> Result<Sbuild, Self::Error> {
        let args: SbuildArgs = SbuildArgs::try_from(config)?;
        Ok(Sbuild { args })
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
        let args = self.args.get_sbuild_tool_args();

        let tool = SbuildTool::new(args);
        ToolRunner::new().run_tool(tool)?;
        if self.args.run_piuparts() {
            self.run_piuparts()?;
        }
        if self.args.run_autopkgtests() {
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
        let args = self.args.get_lintian_tool_args();
        let tool = LintianTool::new(args);
        ToolRunner::new().run_tool(tool)
    }

    pub fn run_piuparts(&self) -> Result<(), SbuildError> {
        let args = self.args.get_piuparts_tool_args();
        let tool = PiupartsTool::new(args);
        ToolRunner::new().run_tool(tool)
    }

    pub fn run_autopkgtests(&self) -> Result<(), SbuildError> {
        let args = self.args.get_autopkg_tool_args();
        let tool = AutopkgtestTool::new(args);
        ToolRunner::new().run_tool(tool)
    }
}

#[cfg(test)]
mod tests {

    use crate::configs::autopkgtest_version::AutopkgtestVersion;
    use crate::configs::pkg_config::{
        BuildEnv, DefaultPackageTypeConfig, LanguageEnv, PackageFields, PackageType,
    };
    use crate::configs::sbuild_version::SbuildVersion;

    use super::*;
    use env_logger::Env;
    use types::config::Architecture;
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

    fn create_base_config() -> PkgConfig {
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
                arch: Architecture::Amd64,
                pkg_builder_version: Version::try_from("1.0.0").unwrap(),
                debcrafter_version: "1.0.0".into(),
                sbuild_cache_dir: Some(sbuild_cache_dir.clone()),
                docker: None,
                run_lintian: None,
                run_piuparts: None,
                run_autopkgtest: None,
                lintian_version: Version::try_from("1.0.0").unwrap(),
                piuparts_version: Version::try_from("1.0.0").unwrap(),
                autopkgtest_version: AutopkgtestVersion::try_from("2.5").unwrap(),
                sbuild_version: SbuildVersion::try_from(
                    "sbuild (Debian sbuild) 0.85.6 (26 February 2024)",
                )
                .unwrap(),
                workdir: PathBuf::from(""),
            },
            _config_root: Some(tempdir().unwrap().path().to_path_buf()),
        };

        config
    }

    #[test]
    fn test_clean_when_file_missing() {
        setup();
        let config = create_base_config();
        let args = SbuildArgs::try_from(config).unwrap();
        let build_env = Sbuild { args: args.clone() };

        let result = build_env.run_env_clean();
        let cache_file = args.get_cache_file();

        assert!(result.is_ok());
        assert!(!Path::new(&cache_file).exists());
    }

    #[test]
    fn test_clean_with_existing_file() {
        setup();
        let config = create_base_config();
        let cache_dir = config.build_env.sbuild_cache_dir.clone().unwrap();

        let args = SbuildArgs::try_from(config).unwrap();

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
        let config = create_base_config();

        let args = SbuildArgs::try_from(config).unwrap();

        let build_env = Sbuild { args: args.clone() };
        let cache_file = args.get_cache_file();

        build_env.run_env_clean().unwrap();
        assert!(!Path::new(&cache_file).exists());

        let result = build_env.run_env_create();
        assert!(result.is_ok());
        assert!(Path::new(&cache_file).exists());
    }
}
