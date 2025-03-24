use std::fmt;

use serde::de;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DistributionError {
    #[error("Unsupported distribution codename: {0}")]
    UnsupportedCodename(String),
}

/// Represents supported Linux distributions for VM image creation
///
/// Each variant contains the specific codename for the distribution
/// which is used to identify compatible build commands and arguments.
#[derive(Debug, Clone, PartialEq)]
pub enum Distribution {
    /// Debian distribution
    Debian(DebianCodename),
    /// Ubuntu distribution
    Ubuntu(UbuntuCodename),
}
impl fmt::Display for Distribution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Distribution::Debian(debian) => debian.fmt(f),
            Distribution::Ubuntu(ubuntu) => ubuntu.fmt(f),
        }
    }
}
/// Supported Debian codenames
#[derive(Debug, Clone, PartialEq)]
pub enum DebianCodename {
    Bookworm,
}

impl DebianCodename {
    /// Get the string representation of the codename
    pub fn as_str(&self) -> &'static str {
        match self {
            DebianCodename::Bookworm => "bookworm",
        }
    }
}
impl fmt::Display for DebianCodename {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
/// Supported Ubuntu codenames
#[derive(Debug, Clone, PartialEq)]
pub enum UbuntuCodename {
    Noble,
    Jammy,
}

impl UbuntuCodename {
    /// Get the string representation of the codename
    pub fn as_str(&self) -> &'static str {
        match self {
            UbuntuCodename::Noble => "noble numbat",
            UbuntuCodename::Jammy => "jammy jellyfish",
        }
    }
}

impl fmt::Display for UbuntuCodename {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Distribution {
    /// Creates a Debian Bookworm distribution
    pub fn bookworm() -> Self {
        Distribution::Debian(DebianCodename::Bookworm)
    }

    /// Creates an Ubuntu Noble Numbat distribution
    pub fn noble() -> Self {
        Distribution::Ubuntu(UbuntuCodename::Noble)
    }

    /// Creates an Ubuntu Jammy Jellyfish distribution
    pub fn jammy() -> Self {
        Distribution::Ubuntu(UbuntuCodename::Jammy)
    }

    /// Creates a Distribution from a codename string
    ///
    /// # Arguments
    /// * `codename` - The distribution codename (e.g., "bookworm", "noble")
    ///
    /// # Returns
    /// * `Result<Self>` - A Distribution instance or an error if unsupported
    pub fn from_codename(codename: &str) -> Result<Self, DistributionError> {
        match codename {
            "bookworm" => Ok(Self::bookworm()),
            "noble" | "noble numbat" => Ok(Self::noble()),
            "jammy" | "jammy jellyfish" => Ok(Self::jammy()),
            _ => Err(DistributionError::UnsupportedCodename(codename.to_string())),
        }
    }
}

impl Serialize for Distribution {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Distribution::Debian(codename) => serializer.serialize_str(codename.as_str()),
            Distribution::Ubuntu(codename) => serializer.serialize_str(codename.as_str()),
        }
    }
}

impl<'de> Deserialize<'de> for Distribution {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let codename = String::deserialize(deserializer)?;
        Distribution::from_codename(&codename).map_err(|e| de::Error::custom(format!("{}", e)))
    }
}
