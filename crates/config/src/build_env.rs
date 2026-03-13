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
    Trixie,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UbuntuCodename {
    Noble,
}

impl DebianCodename {
    pub fn as_str(&self) -> &'static str {
        match self {
            DebianCodename::Bookworm => "bookworm",
            DebianCodename::Trixie => "trixie",
        }
    }
}

impl UbuntuCodename {
    pub fn as_str(&self) -> &'static str {
        match self {
            UbuntuCodename::Noble => "noble numbat",
        }
    }

    pub fn as_short(&self) -> &'static str {
        match self {
            UbuntuCodename::Noble => "noble",
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

    pub fn trixie() -> Self {
        Distribution::Debian(DebianCodename::Trixie)
    }

    pub fn noble() -> Self {
        Distribution::Ubuntu(UbuntuCodename::Noble)
    }

    pub fn from_codename(codename: &str) -> Result<Self, DistributionError> {
        match codename {
            "bookworm" => Ok(Self::bookworm()),
            "trixie" => Ok(Self::trixie()),
            "noble" | "noble numbat" => Ok(Self::noble()),
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
    pub chroot_dir: PathBuf,
    pub workdir: PathBuf,
    pub testing: TestingConfig,
    pub tool_versions: ToolVersions,
    pub snapshot_date: Option<String>,
    pub snapshot_security_date: Option<String>,
}

impl BuildEnv {
    /// Returns the repo URL, using snapshot.debian.org when a snapshot date is set.
    pub fn repo_url(&self) -> String {
        match &self.snapshot_date {
            Some(date) => format!("http://snapshot.debian.org/archive/debian/{}/", date),
            None => self.distribution.repo_url().to_string(),
        }
    }

    /// Returns the security repo URL when a snapshot security date is set.
    pub fn security_repo_url(&self) -> Option<String> {
        self.snapshot_security_date.as_ref().map(|date| {
            format!(
                "http://snapshot.debian.org/archive/debian-security/{}/",
                date
            )
        })
    }

    /// Whether this build env uses snapshot pinning.
    pub fn uses_snapshot(&self) -> bool {
        self.snapshot_date.is_some()
    }
}

#[derive(Debug, Clone)]
pub struct TestingConfig {
    pub run_lintian: bool,
    pub run_piuparts: bool,
    pub run_autopkgtest: bool,
}

#[derive(Debug, Clone)]
pub struct ToolVersions {
    pub debcrafter: String,
    pub sbuild: String,
    pub lintian: String,
    pub piuparts: String,
    pub autopkgtest: String,
}

/// Normalize a snapshot date: accept `YYYYMMDD` (append `T000000Z`) or full `YYYYMMDDTHHMMSSZ`.
pub fn normalize_snapshot_date(date: &str) -> Result<String, String> {
    // Full format: YYYYMMDDTHHMMSSZ (16 chars)
    if date.len() == 16 && date.chars().nth(8) == Some('T') && date.ends_with('Z') {
        let digits_ok = date[..8].chars().all(|c| c.is_ascii_digit())
            && date[9..15].chars().all(|c| c.is_ascii_digit());
        if digits_ok {
            return Ok(date.to_string());
        }
    }
    // Short format: YYYYMMDD (8 chars)
    if date.len() == 8 && date.chars().all(|c| c.is_ascii_digit()) {
        return Ok(format!("{}T000000Z", date));
    }
    Err(format!(
        "Invalid snapshot date '{}': expected YYYYMMDD or YYYYMMDDTHHMMSSZ",
        date
    ))
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
            Distribution::from_codename("trixie"),
            Ok(Distribution::Debian(DebianCodename::Trixie))
        ));
        assert!(Distribution::from_codename("unknown").is_err());
    }

    #[test]
    fn test_distribution_as_short() {
        assert_eq!(Distribution::bookworm().as_short(), "bookworm");
        assert_eq!(Distribution::trixie().as_short(), "trixie");
        assert_eq!(Distribution::noble().as_short(), "noble");
    }

    #[test]
    fn test_distribution_repo_url() {
        assert_eq!(
            Distribution::bookworm().repo_url(),
            "http://deb.debian.org/debian"
        );
        assert_eq!(
            Distribution::trixie().repo_url(),
            "http://deb.debian.org/debian"
        );
        assert_eq!(
            Distribution::noble().repo_url(),
            "http://archive.ubuntu.com/ubuntu"
        );
    }

    #[test]
    fn test_distribution_keyring() {
        assert!(Distribution::bookworm()
            .keyring()
            .contains("debian-archive-keyring"));
        assert!(Distribution::trixie()
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
    fn test_trixie_no_extra_chroot_commands() {
        assert!(Distribution::trixie().extra_chroot_commands().is_empty());
    }

    #[test]
    fn test_ubuntu_lintian_suppressions() {
        let noble = Distribution::noble();
        let supprs = noble.lintian_suppressions();
        assert!(supprs.contains(&"malformed-deb-archive"));
    }

    #[test]
    fn test_debian_no_lintian_suppressions() {
        assert!(Distribution::bookworm().lintian_suppressions().is_empty());
        assert!(Distribution::trixie().lintian_suppressions().is_empty());
    }

    #[test]
    fn test_is_ubuntu_debian() {
        assert!(Distribution::noble().is_ubuntu());
        assert!(!Distribution::noble().is_debian());
        assert!(Distribution::bookworm().is_debian());
        assert!(!Distribution::bookworm().is_ubuntu());
        assert!(Distribution::trixie().is_debian());
        assert!(!Distribution::trixie().is_ubuntu());
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

    fn test_build_env(snapshot_date: Option<&str>, snapshot_security_date: Option<&str>) -> BuildEnv {
        BuildEnv {
            distribution: Distribution::bookworm(),
            arch: Architecture::Amd64,
            pkg_builder_version: "0.3.1".to_string(),
            chroot_dir: PathBuf::from("/tmp/cache"),
            workdir: PathBuf::from("/tmp/work"),
            testing: TestingConfig {
                run_lintian: false,
                run_piuparts: false,
                run_autopkgtest: false,
            },
            tool_versions: ToolVersions {
                debcrafter: "8189263".to_string(),
                sbuild: "0.85.6".to_string(),
                lintian: "2.116.3".to_string(),
                piuparts: "1.1.7".to_string(),
                autopkgtest: "5.28".to_string(),
            },
            snapshot_date: snapshot_date.map(String::from),
            snapshot_security_date: snapshot_security_date.map(String::from),
        }
    }

    #[test]
    fn test_repo_url_without_snapshot() {
        let env = test_build_env(None, None);
        assert_eq!(env.repo_url(), "http://deb.debian.org/debian");
        assert!(!env.uses_snapshot());
    }

    #[test]
    fn test_repo_url_with_snapshot() {
        let env = test_build_env(Some("20250101T000000Z"), None);
        assert_eq!(
            env.repo_url(),
            "http://snapshot.debian.org/archive/debian/20250101T000000Z/"
        );
        assert!(env.uses_snapshot());
    }

    #[test]
    fn test_security_repo_url_without_snapshot() {
        let env = test_build_env(None, None);
        assert!(env.security_repo_url().is_none());
    }

    #[test]
    fn test_security_repo_url_with_snapshot() {
        let env = test_build_env(Some("20250101T000000Z"), Some("20250115T000000Z"));
        assert_eq!(
            env.security_repo_url().unwrap(),
            "http://snapshot.debian.org/archive/debian-security/20250115T000000Z/"
        );
    }

    #[test]
    fn test_normalize_snapshot_date_full() {
        assert_eq!(
            normalize_snapshot_date("20250101T120000Z").unwrap(),
            "20250101T120000Z"
        );
    }

    #[test]
    fn test_normalize_snapshot_date_short() {
        assert_eq!(
            normalize_snapshot_date("20250101").unwrap(),
            "20250101T000000Z"
        );
    }

    #[test]
    fn test_normalize_snapshot_date_invalid() {
        assert!(normalize_snapshot_date("2025-01-01").is_err());
        assert!(normalize_snapshot_date("abc").is_err());
        assert!(normalize_snapshot_date("").is_err());
    }
}
