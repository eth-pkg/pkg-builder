use super::args::{ActionType, BuildEnvSubCommand, PkgBuilderArgs};
use super::packager::DistributionPackager;
use crate::v1::pkg_config::{get_config, PkgConfig};
use clap::Parser;
use env_logger::Env;
use eyre::{Result};
use std::{fs, path::Path};
use crate::v1::pkg_config_verify::PkgVerifyConfig;


pub fn get_distribution(config: PkgConfig, config_file_path: String) -> Result<DistributionPackager> {
    let path = Path::new(&config_file_path);
    let config_file_path = fs::canonicalize(path)?;
    let config_root = config_file_path
        .parent()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    Ok(DistributionPackager::new(config, config_root))
}
pub fn run_cli() -> Result<()> {
    let args = PkgBuilderArgs::parse();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    match args.action {
        ActionType::Verify(command) => {
            let config_file = command.config_file;
            let config = get_config(config_file.clone())?;
            let distribution = get_distribution(config, config_file)?;
            let verify_config_file = command.verify_config_file;
            let verify_config_file = get_config::<PkgVerifyConfig>(verify_config_file.clone())?;
            distribution.verify(verify_config_file)?;
        }
        ActionType::Lintian(command) => {
            let config_file = command.config_file;
            let config = get_config(config_file.clone())?;
            let distribution = get_distribution(config, config_file)?;
            distribution.run_lintian()?;
        }
        ActionType::Piuparts(command) => {
            let config_file = command.config_file;
            let config = get_config(config_file.clone())?;
            let distribution = get_distribution(config, config_file)?;
            distribution.run_piuparts()?;
        }
        ActionType::Autopkgtest(command) => {
            let config_file = command.config_file;
            let config = get_config(config_file.clone())?;
            let distribution = get_distribution(config, config_file)?;
            distribution.run_autopkgtests()?;
        }
        ActionType::Package(command) => {
            let config_file = command.config_file;
            let mut config = get_config::<PkgConfig>(config_file.clone())?;
            if let Some(run_piuparts) = command.run_piuparts {
                config.build_env.run_piuparts = Some(run_piuparts);
            }
            if let Some(run_autopkgttests) = command.run_autopkgtests {
                config.build_env.run_autopkgtest = Some(run_autopkgttests);
            }
            if let Some(run_lintian) = command.run_lintian {
                config.build_env.run_lintian = Some(run_lintian);
            }
            let distribution = get_distribution(config, config_file)?;
            distribution.package()?;
        }
        ActionType::BuildEnv(build_env_action) => {
            match build_env_action.build_env_sub_command {
                BuildEnvSubCommand::Create(sub_command) => {
                    let config_file = sub_command.config_file;
                    let config = get_config(config_file.clone())?;
                    let distribution = get_distribution(config, config_file)?;
                    distribution.create_build_env()?;
                }
                BuildEnvSubCommand::Clean(sub_command) => {
                    let config_file = sub_command.config_file;
                    let config = get_config(config_file.clone())?;
                    let distribution = get_distribution(config, config_file)?;
                    distribution.clean_build_env()?;
                }
            };
        }
    }
    Ok(())
}
