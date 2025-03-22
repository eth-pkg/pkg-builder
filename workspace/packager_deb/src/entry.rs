use types::config::ConfigFile;


pub fn invoke_package(config: ConfigFile) {
        // let mut config = get_config::<PkgConfig>(config_file)
    //     .map_err(|e| PkgBuilderError::ConfigParse(e.to_string()))?;
    // fail_compare_versions(
    //     config.build_env.pkg_builder_version.clone(),
    //     program_version,
    //     program_name,
    // )?;
    // let sbuild_version = config.build_env.sbuild_version.clone();
    // if let ActionType::Package(command) = &args.action {
    //     if let Some(run_piuparts) = command.run_piuparts {
    //         config.build_env.run_piuparts = Some(run_piuparts);
    //     }
    //     if let Some(run_autopkgttests) = command.run_autopkgtest {
    //         config.build_env.run_autopkgtest = Some(run_autopkgttests);
    //     }
    //     if let Some(run_lintian) = command.run_lintian {
    //         config.build_env.run_lintian = Some(run_lintian);
    //     }
    // }
    // let distribution = get_distribution(config, config_file)?;

    // match args.action {
    //     ActionType::Verify(command) => {
    //         let verify_config_file =
    //             get_config_file(command.verify_config, VERIFY_CONFIG_FILE_NAME)?;
    //         let verify_config_file = get_config::<PkgVerifyConfig>(verify_config_file.clone())
    //             .map_err(|e| PkgBuilderError::ConfigParse(e.to_string()))?;
    //         let no_package = command.no_package.unwrap_or_default();
    //         distribution.verify(verify_config_file, !no_package)?;
    //     }
    //     ActionType::Lintian(_) => {
    //         distribution.run_lintian()?;
    //     }
    //     ActionType::Piuparts(_) => {
    //         distribution.run_piuparts()?;
    //     }
    //     ActionType::Autopkgtest(_) => {
    //         distribution.run_autopkgtests()?;
    //     }
    //     ActionType::Package(_) => {
    //         check_sbuild_version(sbuild_version)?;
    //         distribution.package()?;
    //     }
    //     ActionType::Env(build_env_action) => {
    //         match build_env_action.build_env_sub_command {
    //             BuildEnvSubCommand::Create(_) => {
    //                 distribution.create()?;
    //             }
    //             BuildEnvSubCommand::Clean(_) => {
    //                 distribution.clean()?;
    //             }
    //         };
    //     }
    //     ActionType::Version => {
    //         println!("Version: {}", env!("CARGO_PKG_VERSION"));
    //     }
    // }
}