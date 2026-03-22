use sha2::{Digest, Sha256};
use std::fmt;

use crate::http_client;

/// Supported runtime recipes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Runtime {
    None,
    Go,
    Rust,
    Node,
    Java,
    JavaGradle,
    Nim,
    C,
    DotnetNoble,
    DotnetDebian,
    DotnetBackup,
}

/// All available runtime variants, in display order.
pub const ALL_RUNTIMES: &[Runtime] = &[
    Runtime::None,
    Runtime::Go,
    Runtime::Rust,
    Runtime::Node,
    Runtime::Java,
    Runtime::JavaGradle,
    Runtime::Nim,
    Runtime::C,
    Runtime::DotnetNoble,
    Runtime::DotnetDebian,
    Runtime::DotnetBackup,
];

impl fmt::Display for Runtime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Runtime {
    /// The string used in pkg-builder.toml `profile = "..."`.
    pub fn as_str(&self) -> &'static str {
        match self {
            Runtime::None => "none",
            Runtime::Go => "go",
            Runtime::Rust => "rust",
            Runtime::Node => "node",
            Runtime::Java => "java",
            Runtime::JavaGradle => "java-gradle",
            Runtime::Nim => "nim",
            Runtime::C => "c",
            Runtime::DotnetNoble => "dotnet-noble",
            Runtime::DotnetDebian => "dotnet-debian",
            Runtime::DotnetBackup => "dotnet-backup",
        }
    }

    /// Parse from a string (e.g. CLI flag value).
    pub fn from_str(s: &str) -> Option<Runtime> {
        ALL_RUNTIMES.iter().find(|r| r.as_str() == s).copied()
    }

    /// Descriptors for fields that must be provided for this runtime.
    /// Each field has (key, label) — key is the TOML field name, label is a human-readable prompt.
    pub fn required_fields(&self) -> &'static [RuntimeField] {
        match self {
            Runtime::None | Runtime::C => &[],
            Runtime::Go => &[
                RuntimeField {
                    key: "binary_url",
                    label: "Go binary URL",
                },
                RuntimeField {
                    key: "binary_checksum",
                    label: "Go binary checksum (sha256)",
                },
            ],
            Runtime::Rust => &[
                RuntimeField {
                    key: "binary_url",
                    label: "Rust binary URL",
                },
                RuntimeField {
                    key: "binary_gpg_asc",
                    label: "Rust GPG signature (ASCII-armored)",
                },
            ],
            Runtime::Node => &[
                RuntimeField {
                    key: "binary_url",
                    label: "Node.js binary URL",
                },
                RuntimeField {
                    key: "binary_checksum",
                    label: "Node.js binary checksum (sha256)",
                },
            ],
            Runtime::Java => &[
                RuntimeField {
                    key: "binary_url",
                    label: "JDK download URL",
                },
                RuntimeField {
                    key: "binary_checksum",
                    label: "JDK checksum (sha256)",
                },
                RuntimeField {
                    key: "jdk_version",
                    label: "JDK version (e.g. 17.0.10)",
                },
            ],
            Runtime::JavaGradle => &[
                RuntimeField {
                    key: "binary_url",
                    label: "JDK download URL",
                },
                RuntimeField {
                    key: "binary_checksum",
                    label: "JDK checksum (sha256)",
                },
                RuntimeField {
                    key: "jdk_version",
                    label: "JDK version (e.g. 17.0.10)",
                },
                RuntimeField {
                    key: "gradle_binary_url",
                    label: "Gradle download URL",
                },
                RuntimeField {
                    key: "gradle_binary_checksum",
                    label: "Gradle checksum (sha256)",
                },
                RuntimeField {
                    key: "gradle_version",
                    label: "Gradle version (e.g. 8.7)",
                },
            ],
            Runtime::Nim => &[
                RuntimeField {
                    key: "binary_url",
                    label: "Nim binary URL",
                },
                RuntimeField {
                    key: "binary_checksum",
                    label: "Nim binary checksum (sha256sum format: 'hash  filename')",
                },
                RuntimeField {
                    key: "nim_version",
                    label: "Nim version",
                },
            ],
            Runtime::DotnetNoble | Runtime::DotnetDebian | Runtime::DotnetBackup => &[],
        }
    }

    /// Determine what setup this runtime needs.
    ///
    /// This is the single entry point for consumers to find out what to do
    /// for a given runtime. It tries auto-resolution where possible.
    pub fn setup(&self) -> RuntimeSetup {
        match self {
            Runtime::None | Runtime::C => RuntimeSetup::NoVars,
            Runtime::DotnetNoble | Runtime::DotnetDebian | Runtime::DotnetBackup => {
                RuntimeSetup::ManualOnly
            }
            Runtime::Java | Runtime::JavaGradle => RuntimeSetup::NeedsInput {
                fields: self.required_fields(),
            },
            _ => {
                // Go, Rust, Node, Nim: try auto-resolve
                match self.resolve_latest() {
                    Ok(latest) => RuntimeSetup::AutoResolved {
                        latest,
                        fields: self.required_fields(),
                        optional_fields: self.optional_fields(),
                    },
                    Err(_) => RuntimeSetup::NeedsInput {
                        fields: self.required_fields(),
                    },
                }
            }
        }
    }

    /// Given a user-specified version, resolve the runtime vars.
    ///
    /// For runtimes like Go/Node, this fetches the checksum for the specific version.
    /// For Rust, this fetches the GPG signature.
    /// For Nim, this downloads and computes the checksum.
    fn resolve_latest(&self) -> Result<LatestVersion, Box<dyn std::error::Error>> {
        match self {
            Runtime::Go => {
                let (version, url, checksum) = resolve_go_latest()?;
                Ok(LatestVersion {
                    version,
                    vars: vec![
                        ("binary_url".into(), url),
                        ("binary_checksum".into(), checksum),
                    ],
                })
            }
            Runtime::Node => {
                let (version, url, checksum) = resolve_node_latest()?;
                Ok(LatestVersion {
                    version,
                    vars: vec![
                        ("binary_url".into(), url),
                        ("binary_checksum".into(), checksum),
                    ],
                })
            }
            _ => Err("Auto-resolution not supported for this runtime".into()),
        }
    }

    /// Given a user-specified version, resolve the runtime vars.
    ///
    /// For runtimes like Go/Node, this fetches the checksum for the specific version.
    /// For Rust, this fetches the GPG signature.
    /// For Nim, this downloads and computes the checksum.
    /// Returns the vars that could be auto-resolved; missing ones need manual input.
    pub fn resolve_version(
        &self,
        version: &str,
    ) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
        match self {
            Runtime::Go => {
                let url = go_download_url(version);
                let checksum = fetch_go_checksum(version)?;
                Ok(vec![
                    ("binary_url".into(), url),
                    ("binary_checksum".into(), checksum),
                ])
            }
            Runtime::Rust => {
                let url = rust_download_url(version);
                let gpg_asc = fetch_rust_gpg_asc(&url)?;
                Ok(vec![
                    ("binary_url".into(), url),
                    ("binary_gpg_asc".into(), gpg_asc),
                ])
            }
            Runtime::Node => {
                let url = node_download_url(version);
                let checksum = fetch_node_checksum(version)?;
                Ok(vec![
                    ("binary_url".into(), url),
                    ("binary_checksum".into(), checksum),
                ])
            }
            Runtime::Nim => {
                let url = nim_download_url(version);
                let checksum = fetch_nim_checksum(version)?;
                Ok(vec![
                    ("binary_url".into(), url),
                    ("binary_checksum".into(), checksum),
                    ("nim_version".into(), version.to_string()),
                ])
            }
            _ => Err("Version resolution not supported for this runtime".into()),
        }
    }

    /// Optional extra fields that can be prompted after the main vars
    /// (e.g. yarn_version for Node).
    pub fn optional_fields(&self) -> &'static [RuntimeField] {
        match self {
            Runtime::Node => &[RuntimeField {
                key: "yarn_version",
                label: "Yarn version",
            }],
            _ => &[],
        }
    }
}

/// Descriptor for a configuration field (used for both runtime and source fields).
pub struct RuntimeField {
    /// The TOML key (e.g. "binary_url").
    pub key: &'static str,
    /// Human-readable label for prompting.
    pub label: &'static str,
}

/// Result of auto-resolving the latest version of a runtime.
pub struct LatestVersion {
    /// The resolved version string (e.g. "1.22.2").
    pub version: String,
    /// The auto-resolved runtime vars.
    pub vars: Vec<(String, String)>,
}

/// What a consumer needs to do to configure a runtime.
///
/// Returned by `Runtime::setup()`. The consumer matches on this and
/// acts accordingly (prompt, confirm, skip, etc.).
pub enum RuntimeSetup {
    /// No runtime vars needed (none, c).
    NoVars,

    /// Latest version auto-resolved. Consumer should confirm or let the user
    /// override with a custom version (using `Runtime::resolve_version()`).
    /// On override failure, fall back to prompting for `fields`.
    AutoResolved {
        latest: LatestVersion,
        fields: &'static [RuntimeField],
        optional_fields: &'static [RuntimeField],
    },

    /// No auto-resolution available. Consumer should prompt for each field.
    NeedsInput { fields: &'static [RuntimeField] },

    /// Runtime is too complex for auto-population. Consumer should inform the
    /// user to fill in vars manually after init.
    ManualOnly,
}

// ---- Internal resolution functions ----

fn resolve_go_latest() -> Result<(String, String, String), Box<dyn std::error::Error>> {
    let resp = http_client().get("https://go.dev/dl/?mode=json").send()?;
    let releases: serde_json::Value = resp.json()?;
    let arr = releases.as_array().ok_or("Expected array")?;
    let release = arr.first().ok_or("No Go releases found")?;
    let version = release["version"]
        .as_str()
        .ok_or("No version field")?
        .strip_prefix("go")
        .unwrap_or(release["version"].as_str().unwrap());
    let files = release["files"].as_array().ok_or("No files")?;
    let file = files
        .iter()
        .find(|f| {
            f["os"].as_str() == Some("linux")
                && f["arch"].as_str() == Some("amd64")
                && f["kind"].as_str() == Some("archive")
        })
        .ok_or("No linux-amd64 archive found")?;
    let checksum = file["sha256"].as_str().ok_or("No sha256")?;
    let filename = file["filename"].as_str().ok_or("No filename")?;
    let url = format!("https://go.dev/dl/{}", filename);
    Ok((version.to_string(), url, checksum.to_string()))
}

fn fetch_go_checksum(version: &str) -> Result<String, Box<dyn std::error::Error>> {
    let resp = http_client().get("https://go.dev/dl/?mode=json").send()?;
    let releases: serde_json::Value = resp.json()?;
    let target = format!("go{}", version);
    let arr = releases.as_array().ok_or("Expected array")?;
    let release = arr
        .iter()
        .find(|r| r["version"].as_str() == Some(target.as_str()))
        .ok_or("Version not found")?;
    let files = release["files"].as_array().ok_or("No files")?;
    let file = files
        .iter()
        .find(|f| {
            f["os"].as_str() == Some("linux")
                && f["arch"].as_str() == Some("amd64")
                && f["kind"].as_str() == Some("archive")
        })
        .ok_or("No linux-amd64 archive")?;
    Ok(file["sha256"].as_str().ok_or("No sha256")?.to_string())
}

pub fn go_download_url(version: &str) -> String {
    format!("https://go.dev/dl/go{}.linux-amd64.tar.gz", version)
}

fn fetch_rust_gpg_asc(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let asc_url = format!("{}.asc", url);
    let resp = http_client().get(&asc_url).send()?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()).into());
    }
    Ok(resp.text()?)
}

pub fn rust_download_url(version: &str) -> String {
    format!(
        "https://static.rust-lang.org/dist/rust-{}-x86_64-unknown-linux-gnu.tar.xz",
        version
    )
}

fn resolve_node_latest() -> Result<(String, String, String), Box<dyn std::error::Error>> {
    let resp = http_client()
        .get("https://nodejs.org/dist/index.json")
        .send()?;
    let releases: serde_json::Value = resp.json()?;
    let arr = releases.as_array().ok_or("Expected array")?;
    let release = arr
        .iter()
        .find(|r| r["lts"].is_string())
        .or_else(|| arr.first())
        .ok_or("No Node.js releases found")?;
    let version = release["version"]
        .as_str()
        .ok_or("No version")?
        .strip_prefix('v')
        .unwrap_or(release["version"].as_str().unwrap());
    let url = node_download_url(version);
    let checksum = fetch_node_checksum(version)?;
    Ok((version.to_string(), url, checksum))
}

fn fetch_node_checksum(version: &str) -> Result<String, Box<dyn std::error::Error>> {
    let sums_url = format!(
        "https://nodejs.org/download/release/v{}/SHASUMS256.txt",
        version
    );
    let resp = http_client().get(&sums_url).send()?;
    let text = resp.text()?;
    let target = format!("node-v{}-linux-x64.tar.gz", version);
    for line in text.lines() {
        if line.contains(&target) {
            return Ok(line.split_whitespace().next().unwrap_or("").to_string());
        }
    }
    Err("Checksum not found in SHASUMS256.txt".into())
}

pub fn node_download_url(version: &str) -> String {
    format!(
        "https://nodejs.org/download/release/v{}/node-v{}-linux-x64.tar.gz",
        version, version
    )
}

pub fn nim_download_url(version: &str) -> String {
    format!(
        "https://nim-lang.org/download/nim-{}-linux_x64.tar.xz",
        version
    )
}

fn fetch_nim_checksum(version: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url = nim_download_url(version);
    let resp = http_client().get(&url).send()?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()).into());
    }
    let bytes = resp.bytes()?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let hash = format!("{:x}", hasher.finalize());
    Ok(format!("{}  nim-{}-linux_x64.tar.xz", hash, version))
}
