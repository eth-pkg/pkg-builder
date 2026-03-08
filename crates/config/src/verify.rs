use std::path::Path;

use serde::Deserialize;

use crate::{ConfigError, VERIFY_CONFIG_FILE_NAME};

/// Hash entry for a package file.
#[derive(Debug, Clone, Deserialize)]
pub struct PackageHash {
    pub name: String,
    pub hash: String,
}

/// Verification configuration, containing expected hashes.
#[derive(Debug, Clone, Deserialize)]
pub struct VerifyConfig {
    pub package_hash: Vec<PackageHash>,
}

/// Top-level verify config file structure.
#[derive(Debug, Clone, Deserialize)]
pub struct PkgVerifyConfig {
    pub verify: VerifyConfig,
}

impl PkgVerifyConfig {
    /// Load and parse a verify config file.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let config_path = if path.is_dir() {
            path.join(VERIFY_CONFIG_FILE_NAME)
        } else {
            path.to_path_buf()
        };

        let content = std::fs::read_to_string(&config_path).map_err(|e| ConfigError::ReadError {
            path: config_path.clone(),
            source: e,
        })?;

        toml::from_str(&content).map_err(|e| ConfigError::ParseError {
            path: config_path,
            source: e,
        })
    }
}
