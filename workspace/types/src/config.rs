use log::warn;
use semver::Version;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::{
    borrow::Cow,
    cmp::Ordering,
    env, fs,
    io::{self, ErrorKind},
    path::PathBuf,
};
use thiserror::Error;

use crate::{defaults::{CONFIG_FILE_NAME, WORKDIR_ROOT}, distribution::Distribution};

/// Represents the raw configuration file content
#[derive(Debug, Clone)]
pub struct ConfigFile(Cow<'static, str>);

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

impl ConfigFile {
    /// Loads configuration from the specified location or the current directory
    ///
    /// # Arguments
    ///
    /// * `config_path` - Optional path to configuration file or directory
    ///
    /// # Returns
    ///
    /// * `Result<ConfigFile, ConfigError>` - The loaded configuration or an error
    pub fn load(config_path: Option<String>) -> Result<Self, ConfigError> {
        let path = Self::resolve_config_path(config_path)?;
        let content = fs::read_to_string(path).map_err(ConfigError::Io)?;
        Ok(ConfigFile(Cow::Owned(content)))
    }

    /// Resolves the configuration file path
    fn resolve_config_path(config_path: Option<String>) -> Result<PathBuf, ConfigError> {
        let path = match config_path {
            Some(location) => {
                let path = PathBuf::from(location);
                if path.is_dir() {
                    path.join(CONFIG_FILE_NAME)
                } else {
                    path
                }
            }
            None => env::current_dir()
                .map_err(ConfigError::Io)?
                .join(CONFIG_FILE_NAME),
        };

        if !path.exists() {
            return Err(ConfigError::Io(io::Error::new(
                ErrorKind::NotFound,
                format!("Path does not exist: {}", path.display()),
            )));
        }

        Ok(path)
    }

    /// Parses the configuration content into a BuildEnv
    ///
    /// # Returns
    ///
    /// * `Result<BuildEnv, ConfigError>` - The parsed configuration or an error
    pub fn parse(self) -> Result<BuildEnv, ConfigError> {
        let configuration = toml::from_str::<Config>(&self.0)?;
        Ok(configuration.build_env)
    }

    /// Convenience method to load and parse in one operation
    ///
    /// # Arguments
    ///
    /// * `config_path` - Optional path to configuration file or directory
    ///
    /// # Returns
    ///
    /// * `Result<BuildEnv, ConfigError>` - The parsed configuration or an error
    pub fn load_and_parse(config_path: Option<String>) -> Result<BuildEnv, ConfigError> {
        Self::load(config_path)?.parse()
    }
}

// not needed, just for wrapping the header
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Config {
    pub build_env: BuildEnv,
}

/// Build environment configuration
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct BuildEnv {
    /// Distribution codename
    pub codename: Distribution,

    /// Working directory path
    pub workdir: PathBuf,

    /// Required pkg-builder version
    pub pkg_builder_version: PkgBuilderVersion,
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

        let required_version = self.pkg_builder_version.as_ref();

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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PkgBuilderVersion(Version);

impl PkgBuilderVersion {
    pub fn new(version: Version) -> Self {
        Self(version)
    }

    pub fn version(&self) -> &Version {
        &self.0
    }
}

impl From<Version> for PkgBuilderVersion {
    fn from(version: Version) -> Self {
        Self(version)
    }
}

impl AsRef<Version> for PkgBuilderVersion {
    fn as_ref(&self) -> &Version {
        &self.0
    }
}

impl Serialize for PkgBuilderVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for PkgBuilderVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let version_str = String::deserialize(deserializer)?;
        Version::parse(&version_str)
            .map(PkgBuilderVersion)
            .map_err(|e| de::Error::custom(e.to_string()))
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

        let config_file = ConfigFile::load(Some(dir.path().to_string_lossy().to_string())).unwrap();
        assert!(config_file.0.contains("noble numbat"));
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

        let config_file = ConfigFile::load(Some(dir.path().to_string_lossy().to_string())).unwrap();
        let config = config_file.parse();
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.codename, Distribution::noble());
        assert_eq!(config.workdir, PathBuf::from("/tmp/test"));
        assert_eq!(
            config.pkg_builder_version,
            PkgBuilderVersion(Version::new(1, 0, 0))
        );
    }
}
