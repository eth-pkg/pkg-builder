use thiserror::Error;

use types::{
    config::{Config, ConfigError, ConfigFile, ConfigType},
    debian::DebCommandPayload,
    defaults::{CONFIG_FILE_NAME, VERIFY_CONFIG_FILE_NAME},
};

use crate::{
    configs::{pkg_config::PkgConfig, pkg_config_verify::PkgVerifyConfig},
    misc::{build_pipeline::BuildError, validation::ValidationError},
    sbuild::{Sbuild, SbuildError},
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
        ConfigFile::<PkgConfig>::load_and_parse(Some(config.path.display().to_string()))?
            .resolve_paths(config.path.parent().unwrap().to_path_buf());

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

    // do not run piuparts or autopkgtest on verify
    if let DebCommandPayload::Verify { .. } = &cmd_payload {
        pkg_config.build_env.run_piuparts = Some(false);
        pkg_config.build_env.run_autopkgtest = Some(false);
    }

    let packager: Sbuild = pkg_config.try_into()?;
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
