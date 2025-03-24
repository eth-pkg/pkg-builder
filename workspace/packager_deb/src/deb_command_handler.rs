use types::{
    config::{Config, ConfigFile, ConfigType},
    debian::DebCommandPayload,
    defaults::VERIFY_CONFIG_FILE_NAME,
};

// Organize imports by scope with clear grouping and alphabetical ordering
use crate::{
    pkg_config::PkgConfig,
    pkg_config_verify::PkgVerifyConfig,
    sbuild_packager::{PackageError, SbuildPackager},
    validation::parse,
};

impl ConfigType for PkgVerifyConfig {
    fn default_config_path() -> &'static str {
        VERIFY_CONFIG_FILE_NAME
    }
}

pub fn dispatch_package_operation(
    config: ConfigFile<Config>,
    cmd_payload: DebCommandPayload,
) -> Result<(), PackageError> {
    // Parse config first
    let mut pkg_config = parse::<PkgConfig>(config.as_ref())?;

    // Apply CLI overrides if needed
    if let DebCommandPayload::Package {
        run_autopkgtest,
        run_lintian,
        run_piuparts,
    } = &cmd_payload
    {
        // Using pattern matching with references to avoid cloning
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

    // Create packager with parsed config
    let packager = SbuildPackager::new(pkg_config, config.path.parent().unwrap().to_path_buf());

    // Dispatch to appropriate operation based on command payload
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
