use super::args::{ActionType, BuildEnvCommand, BuildEnvSubCommand, PkgBuilderArgs};
use super::cli_config::CliConfig;
use super::packager;
use super::packager::DistributionPackager;
use clap::Parser;
use std::{fs, path::Path};
use thiserror::Error;
use toml;

#[derive(Debug, Error)]
pub enum CliConfigError {
    #[error("Failed to read the packageconfig file: {0}")]
    ConfigRead(#[from] std::io::Error),

    #[error("Failed to parse TOML content of the packagefile: {0}")]
    ConfigParse(#[from] toml::de::Error),

    #[error("Failed to package: {0}")]
    Runtime(#[from] packager::Error),
}

fn read_config(path: &Path) -> Result<CliConfig, CliConfigError> {
    let toml_content = fs::read_to_string(path)?;

    let config: CliConfig = toml::from_str(&toml_content)?;

    Ok(config)
}

fn get_distribution(config_file: String) -> Result<DistributionPackager, CliConfigError> {
    let path = Path::new(&config_file);
    let config_file_path = fs::canonicalize(path)?;
    let config_root = config_file_path
        .parent()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let config = read_config(path)?;

    Ok(DistributionPackager::new(config, config_root))
}
pub fn run_cli() -> Result<(), CliConfigError> {
    let args = PkgBuilderArgs::parse();
    match args.action {
        ActionType::Package(command) => {
            let config_file = command.config_file;
            let distribution = get_distribution(config_file)?;
            distribution.package()?;
        }
        ActionType::BuildEnv(build_env_action) => {
            match build_env_action.build_env_sub_command {
                BuildEnvSubCommand::Create(sub_command) => {
                    let config_file = sub_command.config_file;
                    let distribution = get_distribution(config_file)?;
                    distribution.create_build_env()?;
                },
                BuildEnvSubCommand::Clean(sub_command) => {
                    let config_file = sub_command.config_file;
                    let distribution = get_distribution(config_file)?;
                    distribution.clean_build_env()?;
                }
            };
        },
    }
    Ok(())
}
