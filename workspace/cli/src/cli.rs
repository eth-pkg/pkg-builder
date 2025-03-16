use crate::validation::get_config;

use super::args::{ActionType, BuildEnvSubCommand, PkgBuilderArgs};
use cargo_metadata::semver;
use clap::Parser;
use env_logger::Env;
use log::{error, info, warn};
use packager_deb::sbuild_packager::{PackageError, SbuildPackager};
use regex::Regex;
use semver::Version;
use std::io;
use std::process::Command;
use std::{env, fs, path::Path};
use thiserror::Error;
use types::build::Packager;
use types::pkg_config::PkgConfig;
use types::pkg_config_verify::PkgVerifyConfig;

const CONFIG_FILE_NAME: &str = "pkg-builder.toml";
const VERIFY_CONFIG_FILE_NAME: &str = "pkg-builder-verify.toml";

#[derive(Error, Debug)]
pub enum PkgBuilderError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Failed to find {0} in {1}")]
    ConfigNotFound(String, String),

    #[error("Directory or file does not exist: {0}")]
    PathNotExists(String),

    #[error("{0} version {1} is older than expected {2}!")]
    VersionTooOld(String, String, String),

    #[error("Failed to execute sbuild --version")]
    SbuildExecutionFailed,

    #[error("Failed to parse config: {0}")]
    ConfigParse(String),

    #[error("Invalid codename '{0}' specified")]
    InvalidCodename(String),

    #[error(transparent)]
    PackageError(#[from] PackageError),
}

type Result<T> = std::result::Result<T, PkgBuilderError>;

pub fn run_cli() -> Result<()> {
    let args = PkgBuilderArgs::parse();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let program_name: &str = env!("CARGO_PKG_NAME");
    let program_version: &str = env!("CARGO_PKG_VERSION");
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
        ActionType::Version => None, // Special case
    };
    let config_file = get_config_file(config_path.clone(), CONFIG_FILE_NAME)?;
    let mut config = get_config::<PkgConfig>(config_file.clone())
        .map_err(|e| PkgBuilderError::ConfigParse(e.to_string()))?;
    fail_compare_versions(
        config.build_env.pkg_builder_version.clone(),
        program_version,
        program_name,
    )?;
    let sbuild_version = config.build_env.sbuild_version.clone();
    if let ActionType::Package(command) = &args.action {
        if let Some(run_piuparts) = command.run_piuparts {
            config.build_env.run_piuparts = Some(run_piuparts);
        }
        if let Some(run_autopkgttests) = command.run_autopkgtest {
            config.build_env.run_autopkgtest = Some(run_autopkgttests);
        }
        if let Some(run_lintian) = command.run_lintian {
            config.build_env.run_lintian = Some(run_lintian);
        }
    }
    let distribution = get_distribution(config, config_file)?;

    match args.action {
        ActionType::Verify(command) => {
            let verify_config_file =
                get_config_file(command.verify_config, VERIFY_CONFIG_FILE_NAME)?;
            let verify_config_file = get_config::<PkgVerifyConfig>(verify_config_file.clone())
                .map_err(|e| PkgBuilderError::ConfigParse(e.to_string()))?;
            let no_package = command.no_package.unwrap_or_default();
            distribution.verify(verify_config_file, !no_package)?;
        }
        ActionType::Lintian(_) => {
            distribution.run_lintian()?;
        }
        ActionType::Piuparts(_) => {
            distribution.run_piuparts()?;
        }
        ActionType::Autopkgtest(_) => {
            distribution.run_autopkgtests()?;
        }
        ActionType::Package(_) => {
            check_sbuild_version(sbuild_version)?;
            distribution.package()?;
        }
        ActionType::Env(build_env_action) => {
            match build_env_action.build_env_sub_command {
                BuildEnvSubCommand::Create(_) => {
                    distribution.create()?;
                }
                BuildEnvSubCommand::Clean(_) => {
                    distribution.clean()?;
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
    let output = Command::new("sbuild").arg("--version").output()?;

    if output.status.success() {
        let actual_version = String::from_utf8_lossy(&output.stdout).to_string();
        let actual_version = get_first_line(&actual_version);
        let actual_version = extract_version(actual_version).unwrap();
        info!("sbuild version {}", actual_version);
        fail_compare_versions(expected_version, &actual_version, "sbuild")?;
        Ok(())
    } else {
        Err(PkgBuilderError::SbuildExecutionFailed)
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
    text.split_once('\n')
        .map_or(text, |(first_line, _rest)| first_line)
}

pub fn fail_compare_versions(
    expected_version: String,
    actual_version: &str,
    program_name: &str,
) -> Result<()> {
    let expected_version_parsed = Version::parse(&expected_version).unwrap();
    let actual_version_parsed = Version::parse(actual_version).unwrap();

    match expected_version_parsed.cmp(&actual_version_parsed) {
        std::cmp::Ordering::Less => {
            warn!(
                "Warning: {} using newer version ({}) than expected version ({})",
                program_name, actual_version, expected_version
            );
            Ok(())
        }
        std::cmp::Ordering::Greater => {
            error!(
                "Error: Actual version ({}) is less than expected ({}). Halting. Please install newer version.",
                actual_version, expected_version
            );
            Err(PkgBuilderError::VersionTooOld(
                program_name.to_string(),
                actual_version.to_string(),
                expected_version,
            ))
        }
        std::cmp::Ordering::Equal => {
            info!(
                "{} version {} matches expected. Proceeding.",
                program_name, expected_version
            );
            Ok(())
        }
    }
}

pub fn get_distribution(config: PkgConfig, config_file_path: String) -> Result<SbuildPackager> {
    let path = Path::new(&config_file_path);
    let config_file_path = fs::canonicalize(path)?;
    let config_root = config_file_path
        .parent()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    match config.build_env.codename.as_str() {
        "bookworm" | "noble numbat" | "jammy jellyfish" => {
            Ok(SbuildPackager::new(config.clone(), config_root.clone()))
        }
        _ => Err(PkgBuilderError::InvalidCodename(config.build_env.codename)),
    }
}

pub fn get_config_file(config: Option<String>, config_file_name: &str) -> Result<String> {
    return if let Some(location) = config {
        let path = Path::new(&location);
        if !path.exists() {
            return Err(PkgBuilderError::PathNotExists(location));
        }
        if path.is_dir() {
            let config_file = path.join(config_file_name);
            if config_file.exists() {
                return Ok(config_file.to_str().unwrap().to_string());
            }
            return Err(PkgBuilderError::ConfigNotFound(
                config_file_name.to_string(),
                path.to_str().unwrap().to_string(),
            ));
        }
        Ok(location)
    } else {
        let path = env::current_dir().unwrap();
        let config_file = path.join(config_file_name);
        if config_file.exists() {
            return Ok(config_file.to_str().unwrap().to_string());
        }
        Err(PkgBuilderError::ConfigNotFound(
            config_file_name.to_string(),
            path.to_str().unwrap().to_string(),
        ))
    };
}
