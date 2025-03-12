use crate::packager::DistributionPackager;
use crate::validation::get_config;

use super::args::{ActionType, BuildEnvSubCommand, PkgBuilderArgs};
use clap::Parser;
use common::pkg_config::PkgConfig;
use common::pkg_config_verify::PkgVerifyConfig;
use env_logger::Env;
use eyre::{eyre, Result};
use std::{env, fs, path::Path};
use std::process::Command;
use cargo_metadata::semver;
use log::{error, info, warn};
use semver::Version;
use regex::Regex;

const CONFIG_FILE_NAME: &str = "pkg-builder.toml";
const VERIFY_CONFIG_FILE_NAME: &str = "pkg-builder-verify.toml";


pub fn run_cli() -> Result<()> {
    let args = PkgBuilderArgs::parse();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let program_name: &str = env!("CARGO_PKG_NAME");
    let program_version: &str = env!("CARGO_PKG_VERSION");
    match args.action {
        ActionType::Verify(command) => {
            let config_file = get_config_file(command.config, CONFIG_FILE_NAME)?;
            let config = get_config::<PkgConfig>(config_file.clone())?;

            fail_compare_versions(config.build_env.pkg_builder_version.clone(), program_version, program_name)?;

            let distribution = get_distribution(config, config_file)?;
            let verify_config_file = get_config_file(command.verify_config, VERIFY_CONFIG_FILE_NAME)?;
            let verify_config_file = get_config::<PkgVerifyConfig>(verify_config_file.clone())?;
            let no_package = command.no_package.unwrap_or_default();
            distribution.verify(verify_config_file, !no_package)?;
        }
        ActionType::Lintian(command) => {
            let config_file = get_config_file(command.config, CONFIG_FILE_NAME)?;
            let config = get_config::<PkgConfig>(config_file.clone())?;

            fail_compare_versions(config.build_env.pkg_builder_version.clone(), program_version, program_name)?;

            let distribution = get_distribution(config, config_file)?;
            distribution.run_lintian()?;
        }
        ActionType::Piuparts(command) => {
            let config_file = get_config_file(command.config, CONFIG_FILE_NAME)?;
            let config = get_config::<PkgConfig>(config_file.clone())?;
            fail_compare_versions(config.build_env.pkg_builder_version.clone(), program_version, program_name)?;

            let distribution = get_distribution(config, config_file)?;
            distribution.run_piuparts()?;
        }
        ActionType::Autopkgtest(command) => {
            let config_file = get_config_file(command.config, CONFIG_FILE_NAME)?;
            let config = get_config::<PkgConfig>(config_file.clone())?;
            fail_compare_versions(config.build_env.pkg_builder_version.clone(), program_version, program_name)?;

            let distribution = get_distribution(config, config_file)?;
            distribution.run_autopkgtests()?;
        }
        ActionType::Package(command) => {
            let config_file = get_config_file(command.config, CONFIG_FILE_NAME)?;
            let mut config = get_config::<PkgConfig>(config_file.clone())?;
            fail_compare_versions(config.build_env.pkg_builder_version.clone(), program_version, program_name)?;

            check_sbuild_version(config.build_env.sbuild_version.clone())?;
            if let Some(run_piuparts) = command.run_piuparts {
                config.build_env.run_piuparts = Some(run_piuparts);
            }
            if let Some(run_autopkgttests) = command.run_autopkgtest {
                config.build_env.run_autopkgtest = Some(run_autopkgttests);
            }
            if let Some(run_lintian) = command.run_lintian {
                config.build_env.run_lintian = Some(run_lintian);
            }
            let distribution = get_distribution(config, config_file)?;
            distribution.package()?;
        }
        ActionType::Env(build_env_action) => {
            match build_env_action.build_env_sub_command {
                BuildEnvSubCommand::Create(sub_command) => {
                    let config_file = get_config_file(sub_command.config, CONFIG_FILE_NAME)?;
                    let config = get_config::<PkgConfig>(config_file.clone())?;
                    fail_compare_versions(config.build_env.pkg_builder_version.clone(), program_version, program_name)?;

                    let distribution = get_distribution(config, config_file)?;
                    distribution.create_build_env()?;
                }
                BuildEnvSubCommand::Clean(sub_command) => {
                    let config_file = get_config_file(sub_command.config, CONFIG_FILE_NAME)?;
                    let config = get_config::<PkgConfig>(config_file.clone())?;
                    fail_compare_versions(config.build_env.pkg_builder_version.clone(), program_version, program_name)?;
                    let distribution = get_distribution(config, config_file)?;
                    distribution.clean_build_env()?;
                }
            };
        }
        ActionType::Version => {
            println!("Version: {}", env!("CARGO_PKG_VERSION"));
        }
    }
    Ok(())
}

pub fn check_sbuild_version(expected_version: String) -> Result<()> {
    let output = Command::new("sbuild")
        .arg("--version")
        .output()?;

    if output.status.success() {
        let actual_version = String::from_utf8_lossy(&output.stdout).to_string();
        let actual_version = get_first_line(&actual_version);
        let actual_version = extract_version(actual_version).unwrap();
        info!("sbuild version {}", actual_version);
        fail_compare_versions(expected_version, &actual_version, "sbuild")?;
        Ok(())
    } else {
        Err(eyre!("Failed to execute sbuild --version"))
    }
}

fn extract_version(input: &str) -> Option<&str> {
    // Define a regular expression pattern to match the version number
    let re = Regex::new(r"sbuild \(Debian sbuild\) ([\d.]+)").unwrap();

    // Use the regular expression to capture the version number
    if let Some(captures) = re.captures(input) {
        if let Some(version) = captures.get(1) {
            return Some(version.as_str());
        }
    }
    None
}
fn get_first_line(text: &str) -> &str {
    text.split_once('\n').map_or(text, |(first_line, _rest)| first_line)
}

pub fn fail_compare_versions(expected_version: String, actual_version: &str, program_name: &str) -> Result<()> {
    let expected_version_parsed = Version::parse(&expected_version).unwrap();
    let actual_version_parsed = Version::parse(actual_version).unwrap();
    
    match expected_version_parsed.cmp(&actual_version_parsed) {
        std::cmp::Ordering::Less => {
            warn!("Warning: {} using newer version ({}) than expected version ({})", 
                  program_name, actual_version, expected_version);
            Ok(())
        }
        std::cmp::Ordering::Greater => {
            error!("Error: Actual version ({}) is less than expected ({}). Halting. Please install newer version.", 
                   actual_version, expected_version);
            Err(eyre!("{} version {} is older than expected {}!", 
                      program_name, actual_version, expected_version))
        }
        std::cmp::Ordering::Equal => {
            info!("{} version {} matches expected. Proceeding.", program_name, expected_version);
            Ok(())
        }
    }
}

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


pub fn get_config_file(config: Option<String>, config_file_name: &str) -> Result<String> {
    return if let Some(location) = config {
        let path = Path::new(&location);
        if !path.exists() {
            return Err(eyre!("Directory or file does not exist {}", location));
        }
        if path.is_dir() {
            let config_file = path.join(config_file_name);
            if config_file.exists() {
                return Ok(config_file.to_str().unwrap().to_string());
            }
            return Err(eyre!("Could not find {} in dir: {}", config_file_name, path.to_str().unwrap()));
        }
        Ok(location)
    } else {
        let path = env::current_dir().unwrap();
        let config_file = path.join(config_file_name);
        if config_file.exists() {
            return Ok(config_file.to_str().unwrap().to_string());
        }
        Err(eyre!("Could not find {} in current directory.", config_file_name))
    };
}