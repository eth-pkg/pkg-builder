pub mod build_env;
pub mod language;
pub mod package;
pub mod source;
pub mod validation;
pub mod verify;

use std::path::{Path, PathBuf};

use build_env::BuildEnv;
use package::PackageFields;
use source::SourceKind;
use thiserror::Error;
use validation::validate_config;

/// The main configuration struct, parsed from pkg-builder.toml.
/// Parsed once, validated once, immutable after construction.
#[derive(Debug, Clone)]
pub struct PkgConfig {
    pub package: PackageFields,
    pub source: SourceKind,
    pub build_env: BuildEnv,
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
pub const VERIFY_CONFIG_FILE_NAME: &str = "pkg-builder-verify.toml";
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

        // Resolve spec_file relative to config root
        if self.package.spec_file.is_relative() {
            self.package.spec_file = self.config_root.join(&self.package.spec_file);
        }

        // Resolve tarball URL if it's a local path
        if let SourceKind::Tarball { ref mut url, .. } = self.source {
            if !url.starts_with("http") {
                let resolved = expand_path(&PathBuf::from(&*url), Some(&self.config_root));
                *url = resolved.display().to_string();
            }
        }

        // Resolve sbuild_cache_dir
        self.build_env.sbuild_cache_dir = expand_path(&self.build_env.sbuild_cache_dir, None);
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

/// Raw TOML structure, matches the existing config format exactly.
#[derive(Debug, serde::Deserialize)]
struct RawConfig {
    package_fields: PackageFields,
    package_type: RawPackageType,
    build_env: RawBuildEnv,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "package_type", rename_all = "lowercase")]
enum RawPackageType {
    Default(RawDefaultPackage),
    Git(RawGitPackage),
    Virtual,
}

#[derive(Debug, serde::Deserialize)]
struct RawDefaultPackage {
    tarball_url: String,
    tarball_hash: Option<String>,
    language_env: language::LanguageEnv,
}

#[derive(Debug, serde::Deserialize)]
struct RawGitPackage {
    git_url: url::Url,
    git_tag: String,
    #[serde(default)]
    submodules: Vec<source::Submodule>,
    language_env: language::LanguageEnv,
}

#[derive(Debug, serde::Deserialize)]
struct RawBuildEnv {
    codename: build_env::Distribution,
    arch: build_env::Architecture,
    pkg_builder_version: String,
    #[serde(default)]
    #[allow(dead_code)]
    debcrafter_version: Option<String>,
    #[serde(default)]
    sbuild_cache_dir: Option<PathBuf>,
    #[serde(default)]
    #[allow(dead_code)]
    docker: Option<bool>,
    #[serde(default)]
    run_lintian: Option<bool>,
    #[serde(default)]
    run_piuparts: Option<bool>,
    #[serde(default)]
    run_autopkgtest: Option<bool>,
    lintian_version: String,
    piuparts_version: String,
    autopkgtest_version: String,
    sbuild_version: String,
    #[serde(default)]
    workdir: PathBuf,
    #[serde(default)]
    snapshot_date: Option<String>,
    #[serde(default)]
    snapshot_security_date: Option<String>,
}

impl RawConfig {
    fn into_pkg_config(self, config_root: PathBuf) -> Result<PkgConfig, ConfigError> {
        let source = match self.package_type {
            RawPackageType::Default(d) => SourceKind::Tarball {
                url: d.tarball_url,
                hash: d.tarball_hash,
                language: d.language_env,
            },
            RawPackageType::Git(g) => SourceKind::Git {
                url: g.git_url.to_string(),
                tag: g.git_tag,
                submodules: g.submodules,
                language: g.language_env,
            },
            RawPackageType::Virtual => SourceKind::Virtual,
        };

        let build_env = BuildEnv {
            distribution: self.build_env.codename,
            arch: self.build_env.arch,
            pkg_builder_version: self.build_env.pkg_builder_version,
            sbuild_cache_dir: self
                .build_env
                .sbuild_cache_dir
                .unwrap_or_else(|| PathBuf::from("~/.cache/sbuild")),
            workdir: self.build_env.workdir,
            testing: build_env::TestingConfig {
                run_lintian: self.build_env.run_lintian.unwrap_or(false),
                run_piuparts: self.build_env.run_piuparts.unwrap_or(false),
                run_autopkgtest: self.build_env.run_autopkgtest.unwrap_or(false),
            },
            tool_versions: build_env::ToolVersions {
                sbuild: self.build_env.sbuild_version,
                lintian: self.build_env.lintian_version,
                piuparts: self.build_env.piuparts_version,
                autopkgtest: self.build_env.autopkgtest_version,
            },
            snapshot_date: self.build_env.snapshot_date,
            snapshot_security_date: self.build_env.snapshot_security_date,
        };

        Ok(PkgConfig {
            package: self.package_fields,
            source,
            build_env,
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
[package_fields]
spec_file = "hello.sss"
package_name = "hello"
version_number = "1.0.0"
revision_number = "1"
homepage = "https://example.com"

[package_type]
package_type = "default"
tarball_url = "https://example.com/hello-1.0.0.tar.gz"
tarball_hash = "abc123"

[package_type.language_env]
language_env = "c"

[build_env]
codename = "bookworm"
arch = "amd64"
pkg_builder_version = "0.3.1"
debcrafter_version = "8189263"
lintian_version = "2.116.3"
piuparts_version = "1.1.7"
autopkgtest_version = "5.28"
sbuild_version = "0.85.6"
workdir = "/tmp/test"
"#
    }

    #[test]
    fn test_load_default_package() {
        let dir = tempdir().unwrap();
        write_config(dir.path(), base_default_config());

        let config = PkgConfig::load(dir.path()).unwrap();
        assert_eq!(config.package.package_name, "hello");
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
[package_fields]
spec_file = "test.sss"
package_name = "test-virtual"
version_number = "1.0.0"
revision_number = "1"
homepage = "https://example.com"

[package_type]
package_type = "virtual"

[build_env]
codename = "bookworm"
arch = "amd64"
pkg_builder_version = "0.3.1"
debcrafter_version = "8189263"
lintian_version = "2.116.3"
piuparts_version = "1.1.7"
autopkgtest_version = "5.28"
sbuild_version = "0.85.6"
workdir = "/tmp/test"
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
[package_fields]
spec_file = "test.sss"
package_name = "test-git"
version_number = "1.0.0"
revision_number = "1"
homepage = "https://example.com"

[package_type]
package_type = "git"
git_url = "https://github.com/example/repo.git"
git_tag = "v1.0.0"
submodules = []

[package_type.language_env]
language_env = "go"
go_version = "1.22.2"
go_binary_url = "https://go.dev/dl/go1.22.2.linux-amd64.tar.gz"
go_binary_checksum = "abc123"

[build_env]
codename = "noble"
arch = "amd64"
pkg_builder_version = "0.3.1"
debcrafter_version = "8189263"
lintian_version = "2.116.3"
piuparts_version = "1.1.7"
autopkgtest_version = "5.28"
sbuild_version = "0.85.6"
workdir = "/tmp/test"
"#,
        );

        let config = PkgConfig::load(dir.path()).unwrap();
        assert!(matches!(config.source, SourceKind::Git { .. }));
    }

    #[test]
    fn test_load_by_file_path() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("custom-name.toml");
        fs::write(&file_path, base_default_config()).unwrap();

        let config = PkgConfig::load(&file_path).unwrap();
        assert_eq!(config.package.package_name, "hello");
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
[package_fields]
spec_file = "test.sss"
package_name = "test"
version_number = "1.0.0"
revision_number = "1"
homepage = "https://example.com"

[package_type]
package_type = "virtual"

[build_env]
codename = "bookworm"
arch = "amd64"
pkg_builder_version = "0.3.1"
debcrafter_version = "8189263"
lintian_version = "2.116.3"
piuparts_version = "1.1.7"
autopkgtest_version = "5.28"
sbuild_version = "0.85.6"
workdir = "/tmp/test"
"#,
        );

        let config = PkgConfig::load(dir.path()).unwrap();
        let cache_str = config.build_env.sbuild_cache_dir.display().to_string();
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
[package_fields]
spec_file = "test.sss"
package_name = "test"
version_number = "1.0.0"
revision_number = "1"
homepage = "https://example.com"

[package_type]
package_type = "virtual"

[build_env]
codename = "bookworm"
arch = "amd64"
pkg_builder_version = "0.3.1"
debcrafter_version = "8189263"
lintian_version = "2.116.3"
piuparts_version = "1.1.7"
autopkgtest_version = "5.28"
sbuild_version = "0.85.6"
workdir = "/tmp/test"
sbuild_cache_dir = "/custom/cache"
"#,
        );

        let config = PkgConfig::load(dir.path()).unwrap();
        assert_eq!(
            config.build_env.sbuild_cache_dir,
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
[package_fields]
spec_file = "test.sss"
package_name = "test"
version_number = "1.0.0"
revision_number = "1"
homepage = "https://example.com"

[package_type]
package_type = "default"
tarball_url = "test.tar.gz"

[package_type.language_env]
language_env = "c"

[build_env]
codename = "bookworm"
arch = "amd64"
pkg_builder_version = "0.3.1"
debcrafter_version = "8189263"
lintian_version = "2.116.3"
piuparts_version = "1.1.7"
autopkgtest_version = "5.28"
sbuild_version = "0.85.6"
workdir = "/tmp/test"
"#,
        );

        let config = PkgConfig::load(dir.path()).unwrap();
        match &config.source {
            SourceKind::Tarball { url, .. } => {
                // Local path should be resolved relative to config root
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
[package_fields]
spec_file = "test.sss"
package_name = "test"
version_number = "1.0.0"
revision_number = "1"
homepage = "https://example.com"

[package_type]
package_type = "virtual"

[build_env]
codename = "bookworm"
arch = "amd64"
pkg_builder_version = "0.3.1"
debcrafter_version = "8189263"
lintian_version = "2.116.3"
piuparts_version = "1.1.7"
autopkgtest_version = "5.28"
sbuild_version = "0.85.6"
workdir = "/tmp/test"
"#,
        );

        let config = PkgConfig::load(dir.path()).unwrap();
        assert!(!config.build_env.testing.run_lintian);
        assert!(!config.build_env.testing.run_piuparts);
        assert!(!config.build_env.testing.run_autopkgtest);
    }

    #[test]
    fn test_spec_file_resolved_relative_to_config_root() {
        let dir = tempdir().unwrap();
        write_config(dir.path(), base_default_config());

        let config = PkgConfig::load(dir.path()).unwrap();
        assert!(
            config.package.spec_file.is_absolute(),
            "spec_file should be absolute: {:?}",
            config.package.spec_file
        );
        assert!(config
            .package
            .spec_file
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
}
