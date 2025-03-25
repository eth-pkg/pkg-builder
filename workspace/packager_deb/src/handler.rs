use std::path::PathBuf;
use thiserror::Error;

use types::{
    config::{Config, ConfigError, ConfigFile, ConfigType},
    debian::DebCommandPayload,
    defaults::{CONFIG_FILE_NAME, VERIFY_CONFIG_FILE_NAME, WORKDIR_ROOT},
};

use crate::{
    configs::{pkg_config::PkgConfig, pkg_config_verify::PkgVerifyConfig},
    misc::build_pipeline::BuildError,
    misc::validation::ValidationError,
    sbuild::{Sbuild, SbuildError},
    sbuild_args::expand_path,
};

impl ConfigType for PkgVerifyConfig {
    fn default_config_path() -> &'static str {
        VERIFY_CONFIG_FILE_NAME
    }
}

impl ConfigType for PkgConfig {
    fn default_config_path() -> &'static str {
        CONFIG_FILE_NAME
    }
}

#[derive(Debug, Error)]
pub enum PackageError {
    #[error(transparent)]
    BuildError(#[from] BuildError),
    #[error(transparent)]
    SbuildError(#[from] SbuildError),
    #[error(transparent)]
    ConfigError(#[from] ConfigError),
    #[error(transparent)]
    ValidationError(#[from] ValidationError),
}

pub fn dispatch_package_operation(
    config: ConfigFile<Config>,
    cmd_payload: DebCommandPayload,
) -> Result<(), PackageError> {
    // ReParse config first
    let mut pkg_config =
        ConfigFile::<PkgConfig>::load_and_parse(Some(config.path.display().to_string()))?;

    // Apply CLI overrides if needed
    if let DebCommandPayload::Package {
        run_autopkgtest,
        run_lintian,
        run_piuparts,
    } = &cmd_payload
    {
        if let Some(run_piuparts_value) = run_piuparts {
            pkg_config.build_env.run_piuparts = Some(*run_piuparts_value);
        }
        if let Some(run_autopkgtest_value) = run_autopkgtest {
            pkg_config.build_env.run_autopkgtest = Some(*run_autopkgtest_value);
        }
        if let Some(run_lintian_value) = run_lintian {
            pkg_config.build_env.run_lintian = Some(*run_lintian_value);
        }
    }

    let config_root = config.path.parent().unwrap().to_path_buf();
    normalize_config(&mut pkg_config, config_root.clone());
    let packager = Sbuild::new(pkg_config, config_root.clone());
    match cmd_payload {
        DebCommandPayload::Verify {
            verify_config,
            no_package,
        } => {
            let pkg_verify_config_file =
                ConfigFile::<PkgVerifyConfig>::load_and_parse(verify_config)?;
            packager.run_verify(pkg_verify_config_file, no_package.unwrap_or_default())
        }
        DebCommandPayload::Lintian => packager.run_lintian(),
        DebCommandPayload::Piuparts => packager.run_piuparts(),
        DebCommandPayload::Autopkgtest => packager.run_autopkgtests(),
        DebCommandPayload::Package { .. } => packager.run_package(),
        DebCommandPayload::EnvCreate => packager.run_env_create(),
        DebCommandPayload::EnvClean => packager.run_env_clean(),
    }?;
    Ok(())
}

pub fn normalize_config(config: &mut PkgConfig, config_root: PathBuf) -> () {
    let mut default_work_dir = PathBuf::from(WORKDIR_ROOT);
    default_work_dir.push(config.build_env.codename.as_ref());
    let mut workdir = config.build_env.workdir.clone();
    if workdir.as_os_str().is_empty() {
        workdir = default_work_dir;
    }
    let workdir = expand_path(&workdir, None);
    config.build_env.workdir = workdir;

    // Update the spec file path
    let config_root_path = PathBuf::from(&config_root);
    let spec_file = config.package_fields.spec_file.clone();
    let spec_file_canonical = config_root_path.join(spec_file);
    config.package_fields.spec_file = spec_file_canonical.to_str().unwrap().to_string();
}
