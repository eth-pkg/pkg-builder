use super::args::{ActionType, BuildEnvSubCommand, PkgBuilderArgs};
use clap::Parser;
use env_logger::Env;
use log::error;
use packager_deb::handler::PackageError;
use packager_deb::handler::dispatch_package_operation;
use std::env;
use thiserror::Error;
use types::config::Config;
use types::config::ConfigError;
use types::config::ConfigFile;
use types::debian::DebCommandPayload;

#[derive(Error, Debug)]
pub enum PkgBuilderError {
    #[error(transparent)]
    PackageError(#[from] PackageError),

    #[error(transparent)]
    ConfigError(#[from] ConfigError),
}

type Result<T> = std::result::Result<T, PkgBuilderError>;

pub fn run_cli() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let args = PkgBuilderArgs::parse();
    let program_version: &str = env!("CARGO_PKG_VERSION");

    if let ActionType::Version = &args.action {
        let program_name: &str = env!("CARGO_PKG_NAME");

        println!("{} version: {}", program_name, program_version);
        return Ok(());
    }

    let config_path = match &args.action {
        ActionType::Verify(command) => command.config.clone(),
        ActionType::Package(command) => command.config.clone(),
        ActionType::Env(command) => match &command.build_env_sub_command {
            BuildEnvSubCommand::Create(sub_command) => sub_command.config.clone(),
            BuildEnvSubCommand::Clean(sub_command) => sub_command.config.clone(),
        },
        ActionType::Piuparts(command) => command.config.clone(),
        ActionType::Autopkgtest(command) => command.config.clone(),
        ActionType::Lintian(command) => command.config.clone(),
        ActionType::Version => None, // Special case already handled above
    };
    let config_file = ConfigFile::<Config>::load(config_path)?;
    let build_env = config_file
        .clone()
        .parse()?
        .build_env
        .validate_and_apply_defaults(program_version)?;

    let cmd_payload = match args.action {
        ActionType::Verify(command) => Ok(DebCommandPayload::Verify {
            verify_config: command.verify_config,
            no_package: command.no_package,
        }),
        ActionType::Lintian(_) => Ok(DebCommandPayload::Lintian),
        ActionType::Piuparts(_) => Ok(DebCommandPayload::Piuparts),
        ActionType::Autopkgtest(_) => Ok(DebCommandPayload::Autopkgtest),
        ActionType::Package(cmd) => Ok(DebCommandPayload::Package {
            run_autopkgtest: cmd.run_autopkgtest,
            run_piuparts: cmd.run_piuparts,
            run_lintian: cmd.run_lintian,
        }),
        ActionType::Env(build_env_action) => match build_env_action.build_env_sub_command {
            BuildEnvSubCommand::Create(_) => Ok(DebCommandPayload::EnvCreate),
            BuildEnvSubCommand::Clean(_) => Ok(DebCommandPayload::EnvClean),
        },
        ActionType::Version => Err("Version has no payload."),
    };

    match build_env.codename {
        types::distribution::Distribution::Debian(_)
        | types::distribution::Distribution::Ubuntu(_) => {
            dispatch_package_operation(config_file, cmd_payload.unwrap())?
        }
    };

    Ok(())
}
