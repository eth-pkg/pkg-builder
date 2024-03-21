use super::packager::DistributionPackagerConfig;
use super::packager::DistributionPackager;

use std::{fs, path::Path};
use toml;
use thiserror::Error; 

#[derive(Debug, Error)]
pub enum CliConfigError {
    #[error("Failed to read the packageconfig file: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Failed to parse TOML content of the packagefile: {0}")]
    ParseError(#[from] toml::de::Error),
}


fn read_config(path: &Path) -> Result<DistributionPackagerConfig, CliConfigError> {
    let toml_content = fs::read_to_string(path)?;

    let config: DistributionPackagerConfig = toml::from_str(&toml_content)?;

    Ok(config)
}

pub fn run_cli() -> Result<(), CliConfigError> {
    let path = Path::new("config.toml");

    let config = read_config(&path)?;

    let distribution = DistributionPackager::new(config);

    let _ = distribution.package();

    Ok(())
}