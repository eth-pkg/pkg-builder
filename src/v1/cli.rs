use super::args::{ActionType, BuildEnvSubCommand, PkgBuilderArgs};
use super::packager::DistributionPackager;
use crate::v1::pkg_config::{parse, PkgConfig};
use clap::Parser;
use env_logger::Env;
use eyre::{Result};
use std::{fs, path::Path};

fn read_config(path: &Path) -> Result<PkgConfig> {
    let toml_content = fs::read_to_string(path)?;

    let config: PkgConfig =
        parse(&toml_content)?;

    Ok(config)
}

fn get_distribution(config_file: String) -> Result<DistributionPackager> {
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
pub fn run_cli() -> Result<()> {
    let args = PkgBuilderArgs::parse();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
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
                }
                BuildEnvSubCommand::Clean(sub_command) => {
                    let config_file = sub_command.config_file;
                    let distribution = get_distribution(config_file)?;
                    distribution.clean_build_env()?;
                }
            };
        }
    }
    Ok(())
}
