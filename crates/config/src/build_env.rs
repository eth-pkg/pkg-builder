use std::fmt;
use std::path::PathBuf;

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DistributionError {
    #[error("Unsupported distribution codename: {0}")]
    UnsupportedCodename(String),
}

/// Supported Linux distributions.
#[derive(Debug, Clone, PartialEq)]
pub enum Distribution {
    Debian(DebianCodename),
    Ubuntu(UbuntuCodename),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DebianCodename {
    Bookworm,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UbuntuCodename {
    Noble,
    Jammy,
}

impl DebianCodename {
    pub fn as_str(&self) -> &'static str {
        match self {
            DebianCodename::Bookworm => "bookworm",
        }
    }
}

impl UbuntuCodename {
    pub fn as_str(&self) -> &'static str {
        match self {
            UbuntuCodename::Noble => "noble numbat",
            UbuntuCodename::Jammy => "jammy jellyfish",
        }
    }

    pub fn as_short(&self) -> &'static str {
        match self {
            UbuntuCodename::Noble => "noble",
            UbuntuCodename::Jammy => "jammy",
        }
    }
}

impl fmt::Display for DebianCodename {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Display for UbuntuCodename {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Distribution {
    pub fn bookworm() -> Self {
        Distribution::Debian(DebianCodename::Bookworm)
    }

    pub fn noble() -> Self {
        Distribution::Ubuntu(UbuntuCodename::Noble)
    }

    pub fn jammy() -> Self {
        Distribution::Ubuntu(UbuntuCodename::Jammy)
    }

    pub fn from_codename(codename: &str) -> Result<Self, DistributionError> {
        match codename {
            "bookworm" => Ok(Self::bookworm()),
            "noble" | "noble numbat" => Ok(Self::noble()),
            "jammy" | "jammy jellyfish" => Ok(Self::jammy()),
            _ => Err(DistributionError::UnsupportedCodename(codename.to_string())),
        }
    }

    pub fn as_short(&self) -> &str {
        match self {
            Distribution::Debian(c) => c.as_str(),
            Distribution::Ubuntu(c) => c.as_short(),
        }
    }

    pub fn codename(&self) -> &str {
        self.as_short()
    }

    pub fn repo_url(&self) -> &str {
        match self {
            Distribution::Debian(_) => "http://deb.debian.org/debian",
            Distribution::Ubuntu(_) => "http://archive.ubuntu.com/ubuntu",
        }
    }

    pub fn keyring(&self) -> &str {
        match self {
            Distribution::Debian(_) => "/usr/share/keyrings/debian-archive-keyring.gpg",
            Distribution::Ubuntu(_) => "/usr/share/keyrings/ubuntu-archive-keyring.gpg",
        }
    }

    /// Extra chroot-setup-commands needed for this distribution.
    pub fn extra_chroot_commands(&self) -> Vec<String> {
        match self {
            Distribution::Ubuntu(UbuntuCodename::Noble) => vec![
                "apt install -y software-properties-common".to_string(),
                "add-apt-repository universe".to_string(),
                "add-apt-repository restricted".to_string(),
                "add-apt-repository multiverse".to_string(),
                "apt update".to_string(),
            ],
            _ => vec![],
        }
    }

    /// Tags that lintian should suppress for this distribution.
    pub fn lintian_suppressions(&self) -> Vec<&str> {
        match self {
            Distribution::Ubuntu(_) => vec!["malformed-deb-archive"],
            Distribution::Debian(_) => vec![],
        }
    }

    pub fn is_ubuntu(&self) -> bool {
        matches!(self, Distribution::Ubuntu(_))
    }

    pub fn is_debian(&self) -> bool {
        matches!(self, Distribution::Debian(_))
    }
}

impl AsRef<str> for Distribution {
    fn as_ref(&self) -> &str {
        match self {
            Distribution::Debian(c) => c.as_str(),
            Distribution::Ubuntu(c) => c.as_str(),
        }
    }
}

impl fmt::Display for Distribution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Distribution::Debian(c) => c.fmt(f),
            Distribution::Ubuntu(c) => c.fmt(f),
        }
    }
}

impl Serialize for Distribution {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_ref())
    }
}

impl<'de> Deserialize<'de> for Distribution {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let codename = String::deserialize(deserializer)?;
        Distribution::from_codename(&codename).map_err(|e| de::Error::custom(e.to_string()))
    }
}

/// Target architecture.
#[derive(Debug, Clone, PartialEq)]
pub enum Architecture {
    Amd64,
}

impl fmt::Display for Architecture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Architecture::Amd64 => write!(f, "amd64"),
        }
    }
}

impl<'de> Deserialize<'de> for Architecture {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "amd64" => Ok(Architecture::Amd64),
            _ => Err(de::Error::custom(format!("Unsupported architecture: {}", s))),
        }
    }
}

/// Build environment configuration.
#[derive(Debug, Clone)]
pub struct BuildEnv {
    pub distribution: Distribution,
    pub arch: Architecture,
    pub pkg_builder_version: String,
    pub debcrafter_version: String,
    pub sbuild_cache_dir: PathBuf,
    pub workdir: PathBuf,
    pub testing: TestingConfig,
    pub tool_versions: ToolVersions,
}

#[derive(Debug, Clone)]
pub struct TestingConfig {
    pub run_lintian: bool,
    pub run_piuparts: bool,
    pub run_autopkgtest: bool,
}

#[derive(Debug, Clone)]
pub struct ToolVersions {
    pub sbuild: String,
    pub lintian: String,
    pub piuparts: String,
    pub autopkgtest: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distribution_from_codename() {
        assert!(matches!(
            Distribution::from_codename("bookworm"),
            Ok(Distribution::Debian(DebianCodename::Bookworm))
        ));
        assert!(matches!(
            Distribution::from_codename("noble"),
            Ok(Distribution::Ubuntu(UbuntuCodename::Noble))
        ));
        assert!(matches!(
            Distribution::from_codename("noble numbat"),
            Ok(Distribution::Ubuntu(UbuntuCodename::Noble))
        ));
        assert!(matches!(
            Distribution::from_codename("jammy"),
            Ok(Distribution::Ubuntu(UbuntuCodename::Jammy))
        ));
        assert!(matches!(
            Distribution::from_codename("jammy jellyfish"),
            Ok(Distribution::Ubuntu(UbuntuCodename::Jammy))
        ));
        assert!(Distribution::from_codename("unknown").is_err());
    }

    #[test]
    fn test_distribution_as_short() {
        assert_eq!(Distribution::bookworm().as_short(), "bookworm");
        assert_eq!(Distribution::noble().as_short(), "noble");
        assert_eq!(Distribution::jammy().as_short(), "jammy");
    }

    #[test]
    fn test_distribution_repo_url() {
        assert_eq!(
            Distribution::bookworm().repo_url(),
            "http://deb.debian.org/debian"
        );
        assert_eq!(
            Distribution::noble().repo_url(),
            "http://archive.ubuntu.com/ubuntu"
        );
        assert_eq!(
            Distribution::jammy().repo_url(),
            "http://archive.ubuntu.com/ubuntu"
        );
    }

    #[test]
    fn test_distribution_keyring() {
        assert!(Distribution::bookworm()
            .keyring()
            .contains("debian-archive-keyring"));
        assert!(Distribution::noble()
            .keyring()
            .contains("ubuntu-archive-keyring"));
    }

    #[test]
    fn test_noble_has_extra_chroot_commands() {
        let cmds = Distribution::noble().extra_chroot_commands();
        assert!(!cmds.is_empty());
        assert!(cmds.iter().any(|c| c.contains("software-properties-common")));
        assert!(cmds.iter().any(|c| c.contains("universe")));
        assert!(cmds.iter().any(|c| c.contains("multiverse")));
    }

    #[test]
    fn test_bookworm_no_extra_chroot_commands() {
        assert!(Distribution::bookworm().extra_chroot_commands().is_empty());
    }

    #[test]
    fn test_jammy_no_extra_chroot_commands() {
        assert!(Distribution::jammy().extra_chroot_commands().is_empty());
    }

    #[test]
    fn test_ubuntu_lintian_suppressions() {
        let noble = Distribution::noble();
        let supprs = noble.lintian_suppressions();
        assert!(supprs.contains(&"malformed-deb-archive"));

        let jammy = Distribution::jammy();
        let supprs = jammy.lintian_suppressions();
        assert!(supprs.contains(&"malformed-deb-archive"));
    }

    #[test]
    fn test_debian_no_lintian_suppressions() {
        assert!(Distribution::bookworm().lintian_suppressions().is_empty());
    }

    #[test]
    fn test_is_ubuntu_debian() {
        assert!(Distribution::noble().is_ubuntu());
        assert!(!Distribution::noble().is_debian());
        assert!(Distribution::bookworm().is_debian());
        assert!(!Distribution::bookworm().is_ubuntu());
    }

    #[test]
    fn test_distribution_serde_roundtrip() {
        let json = serde_json::to_string(&Distribution::bookworm()).unwrap();
        assert_eq!(json, "\"bookworm\"");

        // Deserialize via serde_json as a proxy
        let dist: Distribution = serde_json::from_str("\"noble\"").unwrap();
        assert!(matches!(dist, Distribution::Ubuntu(UbuntuCodename::Noble)));
    }

    #[test]
    fn test_architecture_display() {
        assert_eq!(Architecture::Amd64.to_string(), "amd64");
    }
}
