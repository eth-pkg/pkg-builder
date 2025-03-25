use log::warn;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    cmp::Ordering,
    env, fs,
    io::{self, ErrorKind},
    marker::PhantomData,
    path::PathBuf,
};
use thiserror::Error;

use crate::{
    defaults::{CONFIG_FILE_NAME, WORKDIR_ROOT},
    distribution::Distribution,
};

/// Represents the raw configuration file content
#[derive(Debug, Clone)]
pub struct ConfigFile<T> {
    content: Cow<'static, str>,
    _marker: PhantomData<T>,
    pub path: PathBuf,
}
impl<T> AsRef<str> for ConfigFile<T> {
    fn as_ref(&self) -> &str {
        &self.content
    }
}

/// Errors that can occur during configuration handling
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Error parsing TOML content
    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),

    /// I/O error during file operations
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Version parsing error
    #[error("Version parsing error: {0}")]
    VersionParse(#[from] semver::Error),

    /// Incompatible package version
    #[error("{0}")]
    IncompatibleVersion(String),
}

pub trait ConfigType {
    fn default_config_path() -> &'static str;
}

impl<T: ConfigType> ConfigFile<T> {
    /// Loads configuration from the specified location or the current directory
    ///
    /// # Arguments
    ///
    /// * `config_path` - Optional path to configuration file or directory
    ///
    /// # Returns
    ///
    /// * `Result<ConfigFile<T>, ConfigError>` - The loaded configuration or an error
    pub fn load(config_path: Option<String>) -> Result<Self, ConfigError> {
        let path = Self::resolve_config_path(config_path)?;
        let content = fs::read_to_string(&path).map_err(ConfigError::Io)?;
        Ok(ConfigFile {
            content: Cow::Owned(content),
            _marker: PhantomData,
            path,
        })
    }

    /// Resolves the configuration file path
    fn resolve_config_path(config_path: Option<String>) -> Result<PathBuf, ConfigError> {
        let path = match config_path {
            Some(location) => {
                let path = PathBuf::from(location);
                if path.is_dir() {
                    path.join(T::default_config_path())
                } else {
                    path
                }
            }
            None => env::current_dir()
                .map_err(ConfigError::Io)?
                .join(T::default_config_path()),
        };

        if !path.exists() {
            return Err(ConfigError::Io(io::Error::new(
                ErrorKind::NotFound,
                format!("Path does not exist: {}", path.display()),
            )));
        }

        Ok(path)
    }

    /// Parses the configuration content into the generic type T
    ///
    /// # Returns
    ///
    /// * `Result<T, ConfigError>` - The parsed configuration or an error
    pub fn parse(self) -> Result<T, ConfigError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        toml::from_str::<T>(&self.content).map_err(ConfigError::from)
    }

    /// Convenience method to load and parse in one operation
    ///
    /// # Arguments
    ///
    /// * `config_path` - Optional path to configuration file or directory
    ///
    /// # Returns
    ///
    /// * `Result<T, ConfigError>` - The parsed configuration or an error
    pub fn load_and_parse(config_path: Option<String>) -> Result<T, ConfigError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        Self::load(config_path)?.parse()
    }
}

// not needed, just for wrapping the header
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Config {
    pub build_env: BuildEnv,
}
impl ConfigType for Config {
    fn default_config_path() -> &'static str {
        &CONFIG_FILE_NAME
    }
}

/// Build environment configuration
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct BuildEnv {
    /// Distribution codename
    pub codename: Distribution,

    /// Working directory path
    pub workdir: PathBuf,

    /// Required pkg-builder version
    pub pkg_builder_version: Version,
}

impl BuildEnv {
    /// Validate and apply defaults to the build environment configuration
    ///
    /// # Arguments
    ///
    /// * `current_pkg_version` - The current pkg-builder version to validate against
    ///
    /// # Returns
    ///
    /// * `Result<BuildEnv, ConfigError>` - The validated configuration or an error
    pub fn validate_and_apply_defaults(
        mut self,
        current_pkg_version: &str,
    ) -> Result<Self, ConfigError> {
        // Parse the current version for comparison
        let current_version =
            Version::parse(current_pkg_version).map_err(ConfigError::VersionParse)?;

        let required_version = &self.pkg_builder_version;

        match required_version.cmp(&current_version) {
            Ordering::Greater => {
                let incompatible_version = ConfigError::IncompatibleVersion(format!(
                    "Required pkg-builder version {} is higher than current version {}",
                    required_version, current_version
                ));
                return Err(incompatible_version);
            }
            Ordering::Less => {
                warn!(
                    "Required pkg-builder version {} is lower than current version {}. This may cause compatibility issues.",
                    required_version, current_version
                );
            }
            Ordering::Equal => {
                // Versions match, no action needed
            }
        }

        if self.workdir.as_os_str().is_empty() {
            let default_path = format!("{}/{}", WORKDIR_ROOT, self.codename);
            self.workdir = PathBuf::from(default_path);
        }

        Ok(self)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_config_file_load() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join(CONFIG_FILE_NAME);
        let mut file = File::create(&config_path).unwrap();

        writeln!(
            file,
            r#"
            [build_env]
            codename = "noble numbat"
            workdir = "/tmp/test"
            pkg_builder_version = "1.0.0"
        "#
        )
        .unwrap();

        let config_file =
            ConfigFile::<Config>::load(Some(dir.path().to_string_lossy().to_string())).unwrap();
        assert!(config_file.content.contains("noble numbat"));
    }

    #[test]
    fn test_config_file_parse() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join(CONFIG_FILE_NAME);
        let mut file = File::create(&config_path).unwrap();

        writeln!(
            file,
            r#"
            [build_env]
            codename = "noble numbat"
            workdir = "/tmp/test"
            pkg_builder_version = "1.0.0"
        "#
        )
        .unwrap();

        let config_file =
            ConfigFile::<Config>::load(Some(dir.path().to_string_lossy().to_string())).unwrap();
        let config = config_file.parse();
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.build_env.codename, Distribution::noble());
        assert_eq!(config.build_env.workdir, PathBuf::from("/tmp/test"));
        assert_eq!(config.build_env.pkg_builder_version, Version::new(1, 0, 0));
    }
}
