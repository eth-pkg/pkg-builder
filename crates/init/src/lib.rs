mod generate;
pub mod runtime;
mod tarball;

pub use generate::{generate_sps, generate_sss, generate_toml};
pub use runtime::{LatestVersion, Runtime, RuntimeField, RuntimeSetup, ALL_RUNTIMES};
pub use tarball::{
    download_and_hash_tarball, sha256_file, try_verify_upstream_hash, TarballResult,
};

use log::warn;
use std::fmt;
use std::fs;
use std::path::Path;
use std::time::Duration;

/// Create an HTTP client with a 30-second timeout.
pub fn http_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client")
}

// ---- Source type ----

/// Supported source types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceType {
    Tarball,
    Git,
    Virtual,
}

/// All available source types, in display order.
pub const ALL_SOURCE_TYPES: &[SourceType] =
    &[SourceType::Tarball, SourceType::Git, SourceType::Virtual];

impl fmt::Display for SourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl SourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceType::Tarball => "tarball",
            SourceType::Git => "git",
            SourceType::Virtual => "virtual",
        }
    }

    pub fn from_str(s: &str) -> Option<SourceType> {
        ALL_SOURCE_TYPES.iter().find(|t| t.as_str() == s).copied()
    }

    /// Fields the consumer needs to provide for this source type.
    pub fn required_fields(&self) -> &'static [RuntimeField] {
        match self {
            SourceType::Virtual => &[],
            SourceType::Git => &[
                RuntimeField {
                    key: "url",
                    label: "Git repository URL",
                },
                RuntimeField {
                    key: "tag",
                    label: "Git tag",
                },
            ],
            SourceType::Tarball => &[RuntimeField {
                key: "url",
                label: "Tarball URL",
            }],
        }
    }
}

// ---- Distribution ----

/// Supported target distributions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Distribution {
    Bookworm,
    Trixie,
    Noble,
}

/// All available distributions, in display order.
pub const ALL_DISTRIBUTIONS: &[Distribution] = &[
    Distribution::Bookworm,
    Distribution::Trixie,
    Distribution::Noble,
];

impl fmt::Display for Distribution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Distribution {
    pub fn as_str(&self) -> &'static str {
        match self {
            Distribution::Bookworm => "bookworm",
            Distribution::Trixie => "trixie",
            Distribution::Noble => "noble",
        }
    }

    pub fn from_str(s: &str) -> Option<Distribution> {
        ALL_DISTRIBUTIONS.iter().find(|d| d.as_str() == s).copied()
    }
}

// ---- Debian section ----

/// All valid Debian sections, for use in selection prompts.
pub const DEBIAN_SECTIONS: &[&str] = &[
    "admin",
    "cli-mono",
    "comm",
    "database",
    "debug",
    "devel",
    "doc",
    "editors",
    "education",
    "electronics",
    "embedded",
    "fonts",
    "games",
    "gnome",
    "gnu-r",
    "gnustep",
    "golang",
    "graphics",
    "hamradio",
    "haskell",
    "httpd",
    "interpreters",
    "introspection",
    "java",
    "javascript",
    "kde",
    "kernel",
    "libdevel",
    "libs",
    "lisp",
    "localization",
    "mail",
    "math",
    "metapackages",
    "misc",
    "net",
    "news",
    "ocaml",
    "oldlibs",
    "otherosfs",
    "perl",
    "php",
    "python",
    "ruby",
    "rust",
    "science",
    "shells",
    "sound",
    "tasks",
    "tex",
    "text",
    "utils",
    "vcs",
    "video",
    "web",
    "x11",
    "xfce",
    "zope",
];

// ---- Source config ----

/// Resolved source configuration for an init config.
#[derive(Debug, Clone)]
pub enum SourceConfig {
    Virtual,
    Git { url: String, tag: String },
    Tarball { url: String, hash: Option<String> },
}

/// Resolve source details based on source type.
///
/// For `Tarball`: if `url` is a remote URL, downloads it to `output_dir`,
/// computes sha256 hash, and optionally verifies against `upstream_hash`.
/// For local file URLs, hashes the file in place.
pub fn resolve_source(
    source_type: SourceType,
    url: Option<&str>,
    tag: Option<&str>,
    output_dir: &Path,
    upstream_hash: Option<&str>,
) -> Result<SourceConfig, Box<dyn std::error::Error>> {
    match source_type {
        SourceType::Virtual => Ok(SourceConfig::Virtual),
        SourceType::Git => {
            let url = url.ok_or("Git source type requires a URL")?;
            let tag = tag.ok_or("Git source type requires a tag")?;
            Ok(SourceConfig::Git {
                url: url.to_string(),
                tag: tag.to_string(),
            })
        }
        SourceType::Tarball => {
            let url = url.ok_or("Tarball source type requires a URL")?;

            // Local file: hash it directly
            if !url.starts_with("http://") && !url.starts_with("https://") {
                let hash = if Path::new(url).exists() {
                    Some(sha256_file(url)?)
                } else {
                    None
                };
                return Ok(SourceConfig::Tarball {
                    url: url.to_string(),
                    hash,
                });
            }

            // Remote URL: download, hash, save
            match download_and_hash_tarball(url, output_dir, upstream_hash) {
                Ok(result) => Ok(SourceConfig::Tarball {
                    url: url.to_string(),
                    hash: Some(result.hash),
                }),
                Err(e) => {
                    warn!("Failed to download tarball: {}", e);
                    Ok(SourceConfig::Tarball {
                        url: url.to_string(),
                        hash: None,
                    })
                }
            }
        }
    }
}

// ---- Init config ----

/// All the data needed to generate init files, fully resolved (no prompts).
pub struct InitConfig {
    pub name: String,
    pub version: String,
    pub revision: String,
    pub homepage: String,
    pub maintainer_name: String,
    pub maintainer_email: String,
    pub section: String,
    pub summary: String,
    pub source: SourceConfig,
    pub distribution: Distribution,
    pub arch: String,
    pub runtime: Runtime,
    pub runtime_vars: Vec<(String, String)>,
}

/// Paths of the generated files.
pub struct GeneratedFiles {
    pub toml_path: std::path::PathBuf,
    pub sss_path: std::path::PathBuf,
    pub sps_path: std::path::PathBuf,
}

/// Generate and write the three init files to `output_dir`.
///
/// Returns an error if any of the target files already exist.
pub fn write_init_files(
    config: &InitConfig,
    output_dir: &Path,
) -> Result<GeneratedFiles, Box<dyn std::error::Error>> {
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    let toml_path = output_dir.join("pkg-builder.toml");
    let sss_path = output_dir.join(format!("{}.sss", config.name));
    let sps_path = output_dir.join(format!("{}.sps", config.name));

    for path in [&toml_path, &sss_path, &sps_path] {
        if path.exists() {
            return Err(format!("{} already exists", path.display()).into());
        }
    }

    fs::write(&toml_path, generate_toml(config))?;
    fs::write(&sss_path, generate_sss(config))?;
    fs::write(&sps_path, generate_sps(config))?;

    Ok(GeneratedFiles {
        toml_path,
        sss_path,
        sps_path,
    })
}
