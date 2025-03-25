use types::{
    config::{Config, ConfigFile, ConfigType},
    debian::DebCommandPayload,
    defaults::{CONFIG_FILE_NAME, VERIFY_CONFIG_FILE_NAME},
};

// Organize imports by scope with clear grouping and alphabetical ordering
use crate::{
    pkg_config::PkgConfig,
    pkg_config_verify::PkgVerifyConfig,
    sbuild_packager::{PackageError, SbuildPackager},
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

    let packager = SbuildPackager::new(pkg_config, config.path.parent().unwrap().to_path_buf());

    match cmd_payload {
        DebCommandPayload::Verify {
            verify_config,
            no_package,
        } => {
            let pkg_verify_config_file =
                ConfigFile::<PkgVerifyConfig>::load_and_parse(verify_config)?;
            packager.verify(pkg_verify_config_file, no_package.unwrap_or_default())
        }
        DebCommandPayload::Lintian => packager.run_lintian(),
        DebCommandPayload::Piuparts => packager.run_piuparts(),
        DebCommandPayload::Autopkgtest => packager.run_autopkgtests(),
        DebCommandPayload::Package { .. } => packager.run_package(),
        DebCommandPayload::EnvCreate => packager.run_env_create(),
        DebCommandPayload::EnvClean => packager.run_env_clean(),
    }
}
