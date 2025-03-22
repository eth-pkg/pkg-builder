use super::args::{ActionType, BuildEnvSubCommand, PkgBuilderArgs};
use clap::Parser;
use env_logger::Env;
use log::error;
use packager_deb::entry::invoke_package;
use packager_deb::sbuild_packager::PackageError;
use std::env;
use thiserror::Error;
use types::config::ConfigError;
use types::config::ConfigFile;

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
    let config_file = ConfigFile::load(config_path)?;
    let build_env = config_file.clone()
        .parse()?
        .validate_and_apply_defaults(program_version)?;

    match build_env.codename {
        types::distribution::Distribution::Debian(_)
        | types::distribution::Distribution::Ubuntu(_) => invoke_package(config_file),
    };

    Ok(())
}
