pub mod build_env;
pub mod package;
pub mod runtime;
pub mod source;
pub mod validation;
pub mod verify;

use std::path::{Path, PathBuf};

use build_env::BuildEnv;
use package::PackageFields;
use runtime::RuntimeConfig;
use source::SourceKind;
use thiserror::Error;
use validation::validate_config;
use verify::VerifyConfig;

/// The main configuration struct, parsed from pkg-builder.toml.
/// Parsed once, validated once, immutable after construction.
#[derive(Debug, Clone)]
pub struct PkgConfig {
    pub package: PackageFields,
    pub source: SourceKind,
    pub build_env: BuildEnv,
    pub runtime: Option<RuntimeConfig>,
    pub verify: Option<VerifyConfig>,
    /// Root directory of the config file (for resolving relative paths)
    pub config_root: PathBuf,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config at {path}: {source}")]
    ReadError {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("Failed to parse config at {path}: {source}")]
    ParseError {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("pkg-builder version mismatch: config requires {required}, running {actual}")]
    VersionMismatch { required: String, actual: String },
}

/// Name of the default configuration file
pub const CONFIG_FILE_NAME: &str = "pkg-builder.toml";
pub const WORKDIR_ROOT: &str = "~/.pkg-builder/packages";

impl PkgConfig {
    /// Single entry point: load, parse, validate, resolve paths.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let config_path = if path.is_dir() {
            path.join(CONFIG_FILE_NAME)
        } else {
            path.to_path_buf()
        };

        let content = std::fs::read_to_string(&config_path).map_err(|e| ConfigError::ReadError {
            path: config_path.clone(),
            source: e,
        })?;

        let raw: RawConfig =
            toml::from_str(&content).map_err(|e| ConfigError::ParseError {
                path: config_path.clone(),
                source: e,
            })?;

        let config_root = config_path
            .parent()
            .unwrap_or(Path::new("."))
            .to_path_buf();

        let mut config = raw.into_pkg_config(config_root)?;

        validate_config(&mut config)?;

        // Resolve paths after validation
        config.resolve_paths();

        Ok(config)
    }

    /// Resolve relative paths to absolute paths using config_root.
    fn resolve_paths(&mut self) {
        // Resolve workdir
        if self.build_env.workdir.as_os_str().is_empty() {
            let default = format!("{}/{}", WORKDIR_ROOT, self.build_env.distribution.as_short());
            self.build_env.workdir = PathBuf::from(default);
        }
        self.build_env.workdir = expand_path(&self.build_env.workdir, None);

        // Resolve spec relative to config root
        if self.package.spec.is_relative() {
            self.package.spec = self.config_root.join(&self.package.spec);
        }

        // Resolve tarball URL if it's a local path
        if let SourceKind::Tarball { ref mut url, .. } = self.source {
            if !url.starts_with("http") {
                let resolved = expand_path(&PathBuf::from(&*url), Some(&self.config_root));
                *url = resolved.display().to_string();
            }
        }

        // Resolve chroot_dir
        self.build_env.chroot_dir = expand_path(&self.build_env.chroot_dir, None);
    }
}

/// Expand a path: handle ~, relative paths.
pub fn expand_path(path: &Path, relative_to: Option<&Path>) -> PathBuf {
    let path_str = path.to_string_lossy();
    if path_str.starts_with('~') {
        let expanded = shellexpand::tilde(&path_str).to_string();
        PathBuf::from(expanded)
    } else if path.is_relative() {
        if let Some(base) = relative_to {
            base.join(path)
        } else {
            path.to_path_buf()
        }
    } else {
        path.to_path_buf()
    }
}

// ---- Raw deserialization types (internal) ----

/// Raw TOML structure matching the new flat config format.
#[derive(Debug, serde::Deserialize)]
struct RawConfig {
    package: PackageFields,
    source: RawSource,
    build: RawBuild,
    #[serde(default)]
    runtime: Option<RuntimeConfig>,
    #[serde(default)]
    testing: Option<RawTesting>,
    tools: RawTools,
    #[serde(default)]
    verify: Option<VerifyConfig>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum RawSource {
    Tarball {
        url: String,
        #[serde(default)]
        hash: Option<String>,
    },
    Git {
        url: String,
        tag: String,
        #[serde(default)]
        submodules: Vec<source::Submodule>,
    },
    Virtual,
}

#[derive(Debug, serde::Deserialize)]
struct RawBuild {
    distribution: build_env::Distribution,
    arch: build_env::Architecture,
    #[serde(default)]
    workdir: PathBuf,
    #[serde(default)]
    chroot_dir: Option<PathBuf>,
    #[serde(default)]
    snapshot_date: Option<String>,
    #[serde(default)]
    snapshot_security_date: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct RawTesting {
    #[serde(default)]
    lintian: bool,
    #[serde(default)]
    piuparts: bool,
    #[serde(default)]
    autopkgtest: bool,
}

#[derive(Debug, serde::Deserialize)]
struct RawTools {
    pkg_builder: String,
    debcrafter: String,
    sbuild: String,
    lintian: String,
    piuparts: String,
    autopkgtest: String,
}

impl RawConfig {
    fn into_pkg_config(self, config_root: PathBuf) -> Result<PkgConfig, ConfigError> {
        let source = match self.source {
            RawSource::Tarball { url, hash } => SourceKind::Tarball { url, hash },
            RawSource::Git {
                url,
                tag,
                submodules,
            } => SourceKind::Git {
                url,
                tag,
                submodules,
            },
            RawSource::Virtual => SourceKind::Virtual,
        };

        let testing = self.testing.unwrap_or(RawTesting {
            lintian: false,
            piuparts: false,
            autopkgtest: false,
        });

        let build_env = BuildEnv {
            distribution: self.build.distribution,
            arch: self.build.arch,
            pkg_builder_version: self.tools.pkg_builder,
            chroot_dir: self
                .build
                .chroot_dir
                .unwrap_or_else(|| PathBuf::from("~/.cache/sbuild")),
            workdir: self.build.workdir,
            testing: build_env::TestingConfig {
                run_lintian: testing.lintian,
                run_piuparts: testing.piuparts,
                run_autopkgtest: testing.autopkgtest,
            },
            tool_versions: build_env::ToolVersions {
                debcrafter: self.tools.debcrafter,
                sbuild: self.tools.sbuild,
                lintian: self.tools.lintian,
                piuparts: self.tools.piuparts,
                autopkgtest: self.tools.autopkgtest,
            },
            snapshot_date: self.build.snapshot_date,
            snapshot_security_date: self.build.snapshot_security_date,
        };

        Ok(PkgConfig {
            package: self.package,
            source,
            build_env,
            runtime: self.runtime,
            verify: self.verify,
            config_root,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write_config(dir: &Path, content: &str) {
        fs::write(dir.join(CONFIG_FILE_NAME), content).unwrap();
    }

    fn base_default_config() -> &'static str {
        r#"
[package]
name = "hello"
version = "1.0.0"
revision = "1"
homepage = "https://example.com"
spec = "hello.sss"

[source]
type = "tarball"
url = "https://example.com/hello-1.0.0.tar.gz"
hash = "abc123"

[build]
distribution = "bookworm"
arch = "amd64"
workdir = "/tmp/test"

[tools]
pkg_builder = "0.3.1"
sbuild = "0.85.6"
lintian = "2.116.3"
piuparts = "1.1.7"
autopkgtest = "5.28"
"#
    }

    #[test]
    fn test_load_default_package() {
        let dir = tempdir().unwrap();
        write_config(dir.path(), base_default_config());

        let config = PkgConfig::load(dir.path()).unwrap();
        assert_eq!(config.package.name, "hello");
        assert!(matches!(config.source, SourceKind::Tarball { .. }));
        assert!(matches!(
            config.build_env.distribution,
            build_env::Distribution::Debian(_)
        ));
    }

    #[test]
    fn test_load_virtual_package() {
        let dir = tempdir().unwrap();
        write_config(
            dir.path(),
            r#"
[package]
name = "test-virtual"
version = "1.0.0"
revision = "1"
homepage = "https://example.com"
spec = "test.sss"

[source]
type = "virtual"

[build]
distribution = "bookworm"
arch = "amd64"
workdir = "/tmp/test"

[tools]
pkg_builder = "0.3.1"
sbuild = "0.85.6"
lintian = "2.116.3"
piuparts = "1.1.7"
autopkgtest = "5.28"
"#,
        );

        let config = PkgConfig::load(dir.path()).unwrap();
        assert!(matches!(config.source, SourceKind::Virtual));
    }

    #[test]
    fn test_load_git_package() {
        let dir = tempdir().unwrap();
        write_config(
            dir.path(),
            r#"
[package]
name = "test-git"
version = "1.0.0"
revision = "1"
homepage = "https://example.com"
spec = "test.sss"

[source]
type = "git"
url = "https://github.com/example/repo.git"
tag = "v1.0.0"
submodules = []

[build]
distribution = "noble"
arch = "amd64"
workdir = "/tmp/test"

[runtime]
recipe = "go"
binary_url = "https://go.dev/dl/go1.22.2.linux-amd64.tar.gz"
binary_checksum = "abc123"

[tools]
pkg_builder = "0.3.1"
sbuild = "0.85.6"
lintian = "2.116.3"
piuparts = "1.1.7"
autopkgtest = "5.28"
"#,
        );

        let config = PkgConfig::load(dir.path()).unwrap();
        assert!(matches!(config.source, SourceKind::Git { .. }));
        assert!(config.runtime.is_some());
        assert_eq!(config.runtime.as_ref().unwrap().recipe, "go");
    }

    #[test]
    fn test_load_by_file_path() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("custom-name.toml");
        fs::write(&file_path, base_default_config()).unwrap();

        let config = PkgConfig::load(&file_path).unwrap();
        assert_eq!(config.package.name, "hello");
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = PkgConfig::load("/nonexistent/path/pkg-builder.toml");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::ReadError { .. }));
    }

    #[test]
    fn test_load_invalid_toml() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join(CONFIG_FILE_NAME), "invalid toml {{{").unwrap();

        let result = PkgConfig::load(dir.path());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::ParseError { .. }));
    }

    #[test]
    fn test_cache_dir_defaults_to_sbuild_cache() {
        let dir = tempdir().unwrap();
        write_config(
            dir.path(),
            r#"
[package]
name = "test"
version = "1.0.0"
revision = "1"
homepage = "https://example.com"
spec = "test.sss"

[source]
type = "virtual"

[build]
distribution = "bookworm"
arch = "amd64"
workdir = "/tmp/test"

[tools]
pkg_builder = "0.3.1"
sbuild = "0.85.6"
lintian = "2.116.3"
piuparts = "1.1.7"
autopkgtest = "5.28"
"#,
        );

        let config = PkgConfig::load(dir.path()).unwrap();
        let cache_str = config.build_env.chroot_dir.display().to_string();
        assert!(
            cache_str.contains(".cache/sbuild"),
            "Expected default cache dir, got: {}",
            cache_str
        );
    }

    #[test]
    fn test_cache_dir_overridden() {
        let dir = tempdir().unwrap();
        write_config(
            dir.path(),
            r#"
[package]
name = "test"
version = "1.0.0"
revision = "1"
homepage = "https://example.com"
spec = "test.sss"

[source]
type = "virtual"

[build]
distribution = "bookworm"
arch = "amd64"
workdir = "/tmp/test"
chroot_dir = "/custom/cache"

[tools]
pkg_builder = "0.3.1"
sbuild = "0.85.6"
lintian = "2.116.3"
piuparts = "1.1.7"
autopkgtest = "5.28"
"#,
        );

        let config = PkgConfig::load(dir.path()).unwrap();
        assert_eq!(
            config.build_env.chroot_dir,
            PathBuf::from("/custom/cache")
        );
    }

    #[test]
    fn test_tarball_url_http_unchanged() {
        let dir = tempdir().unwrap();
        write_config(dir.path(), base_default_config());

        let config = PkgConfig::load(dir.path()).unwrap();
        match &config.source {
            SourceKind::Tarball { url, .. } => {
                assert_eq!(url, "https://example.com/hello-1.0.0.tar.gz");
            }
            _ => panic!("Expected Tarball"),
        }
    }

    #[test]
    fn test_tarball_url_local_resolved() {
        let dir = tempdir().unwrap();
        write_config(
            dir.path(),
            r#"
[package]
name = "test"
version = "1.0.0"
revision = "1"
homepage = "https://example.com"
spec = "test.sss"

[source]
type = "tarball"
url = "test.tar.gz"

[build]
distribution = "bookworm"
arch = "amd64"
workdir = "/tmp/test"

[tools]
pkg_builder = "0.3.1"
sbuild = "0.85.6"
lintian = "2.116.3"
piuparts = "1.1.7"
autopkgtest = "5.28"
"#,
        );

        let config = PkgConfig::load(dir.path()).unwrap();
        match &config.source {
            SourceKind::Tarball { url, .. } => {
                assert!(
                    url.ends_with("test.tar.gz"),
                    "URL should end with filename: {}",
                    url
                );
                assert!(
                    url.contains(dir.path().to_str().unwrap()),
                    "URL should contain config root: {}",
                    url
                );
            }
            _ => panic!("Expected Tarball"),
        }
    }

    #[test]
    fn test_testing_defaults_to_false() {
        let dir = tempdir().unwrap();
        write_config(
            dir.path(),
            r#"
[package]
name = "test"
version = "1.0.0"
revision = "1"
homepage = "https://example.com"
spec = "test.sss"

[source]
type = "virtual"

[build]
distribution = "bookworm"
arch = "amd64"
workdir = "/tmp/test"

[tools]
pkg_builder = "0.3.1"
sbuild = "0.85.6"
lintian = "2.116.3"
piuparts = "1.1.7"
autopkgtest = "5.28"
"#,
        );

        let config = PkgConfig::load(dir.path()).unwrap();
        assert!(!config.build_env.testing.run_lintian);
        assert!(!config.build_env.testing.run_piuparts);
        assert!(!config.build_env.testing.run_autopkgtest);
    }

    #[test]
    fn test_spec_resolved_relative_to_config_root() {
        let dir = tempdir().unwrap();
        write_config(dir.path(), base_default_config());

        let config = PkgConfig::load(dir.path()).unwrap();
        assert!(
            config.package.spec.is_absolute(),
            "spec should be absolute: {:?}",
            config.package.spec
        );
        assert!(config
            .package
            .spec
            .display()
            .to_string()
            .ends_with("hello.sss"));
    }

    #[test]
    fn test_expand_path_tilde() {
        let result = expand_path(Path::new("~/test"), None);
        assert!(!result.to_string_lossy().starts_with('~'));
        assert!(result.to_string_lossy().ends_with("test"));
    }

    #[test]
    fn test_expand_path_absolute() {
        let result = expand_path(Path::new("/absolute/path"), None);
        assert_eq!(result, PathBuf::from("/absolute/path"));
    }

    #[test]
    fn test_expand_path_relative_with_base() {
        let result = expand_path(
            Path::new("relative/file.txt"),
            Some(Path::new("/base/dir")),
        );
        assert_eq!(result, PathBuf::from("/base/dir/relative/file.txt"));
    }

    #[test]
    fn test_expand_path_relative_without_base() {
        let result = expand_path(Path::new("relative/file.txt"), None);
        assert_eq!(result, PathBuf::from("relative/file.txt"));
    }

    #[test]
    fn test_runtime_with_vars() {
        let dir = tempdir().unwrap();
        write_config(
            dir.path(),
            r#"
[package]
name = "test-go"
version = "1.0.0"
revision = "1"
homepage = "https://example.com"
spec = "test.sss"

[source]
type = "tarball"
url = "https://example.com/test.tar.gz"
hash = "abc123"

[build]
distribution = "bookworm"
arch = "amd64"
workdir = "/tmp/test"

[runtime]
recipe = "go"
binary_url = "https://go.dev/dl/go1.22.2.linux-amd64.tar.gz"
binary_checksum = "5901c52b"

[tools]
pkg_builder = "0.3.1"
sbuild = "0.85.6"
lintian = "2.116.3"
piuparts = "1.1.7"
autopkgtest = "5.28"
"#,
        );

        let config = PkgConfig::load(dir.path()).unwrap();
        let rt = config.runtime.as_ref().unwrap();
        assert_eq!(rt.recipe, "go");
        assert_eq!(
            rt.vars.get("binary_url").and_then(|v| v.as_str()),
            Some("https://go.dev/dl/go1.22.2.linux-amd64.tar.gz")
        );
    }

    #[test]
    fn test_runtime_with_packages_list() {
        let dir = tempdir().unwrap();
        write_config(
            dir.path(),
            r#"
[package]
name = "test-dotnet"
version = "1.0.0"
revision = "1"
homepage = "https://example.com"
spec = "test.sss"

[source]
type = "tarball"
url = "https://example.com/test.tar.gz"
hash = "abc123"

[build]
distribution = "noble"
arch = "amd64"
workdir = "/tmp/test"

[runtime]
recipe = "dotnet-backup"
packages = [
    { name = "pkg-a", hash = "aaa", url = "http://a" },
    { name = "pkg-b", hash = "bbb", url = "http://b" },
]

[tools]
pkg_builder = "0.3.1"
sbuild = "0.85.6"
lintian = "2.116.3"
piuparts = "1.1.7"
autopkgtest = "5.28"
"#,
        );

        let config = PkgConfig::load(dir.path()).unwrap();
        let rt = config.runtime.as_ref().unwrap();
        assert_eq!(rt.recipe, "dotnet-backup");
        let pkgs = rt.vars.get("packages").unwrap().as_array().unwrap();
        assert_eq!(pkgs.len(), 2);
    }
}
