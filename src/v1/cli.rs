use super::cli_config::CliConfig;
use super::packager::{DistributionPackager};
use std::{fs, path::Path};
use std::path::PathBuf;
use thiserror::Error;
use toml;
use crate::v1::debcrafter_helper::Error;
use crate::v1::packager;

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

pub fn run_cli() -> Result<(), CliConfigError> {
    let path = Path::new(
        "examples/bookworm/virtual-package/pkg-builder.toml",
    );
    let config_file_path = fs::canonicalize(path)?;
    let config_root = config_file_path.parent().unwrap().to_str().unwrap().to_string();
    let config = read_config(path)?;

    let distribution = DistributionPackager::new(config, config_root);
    distribution.package()?;

    Ok(())
}
