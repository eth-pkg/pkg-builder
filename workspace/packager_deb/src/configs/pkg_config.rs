use std::path::PathBuf;

use serde::Deserialize;
use types::{config::Architecture, defaults::WORKDIR_ROOT, distribution::Distribution, url::Url, version::Version};


use crate::misc::utils::expand_path;

use super::{autopkgtest_version::AutopkgtestVersion, sbuild_version::SbuildVersion};

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct RustConfig {
    pub rust_version: Version,
    pub rust_binary_url: Url,
    pub rust_binary_gpg_asc: String,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct GoConfig {
    pub go_version: Version,
    pub go_binary_url: Url,
    pub go_binary_checksum: String,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct JavascriptConfig {
    pub node_version: Version,
    pub node_binary_url: Url,
    pub node_binary_checksum: String,
    pub yarn_version: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct GradleConfig {
    pub gradle_version: String,
    pub gradle_binary_url: Url,
    pub gradle_binary_checksum: String,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct JavaConfig {
    pub is_oracle: bool,
    pub jdk_version: String,
    pub jdk_binary_url: Url,
    pub jdk_binary_checksum: String,
    pub gradle: Option<GradleConfig>,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct DotnetPackage {
    pub name: String,
    pub hash: String,
    pub url: Url,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
pub struct DotnetConfig {
    pub use_backup_version: bool,
    pub dotnet_packages: Vec<DotnetPackage>,
    pub deps: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct NimConfig {
    pub nim_version: Version,
    pub nim_binary_url: String,
    pub nim_version_checksum: String,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
#[serde(tag = "language_env", rename_all = "lowercase")]
pub enum LanguageEnv {
    Rust(RustConfig),
    Go(GoConfig),
    JavaScript(JavascriptConfig),
    Java(JavaConfig),
    Dotnet(DotnetConfig),
    TypeScript(JavascriptConfig),
    Nim(NimConfig),
    #[default]
    C,
    Python,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct DefaultPackageTypeConfig {
    pub tarball_url: String,
    pub tarball_hash: Option<String>,
    pub language_env: LanguageEnv,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct SubModule {
    pub commit: String,
    pub path: String,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct GitPackageTypeConfig {
    pub git_tag: String,
    pub git_url: Url,
    pub submodules: Vec<SubModule>,
    pub language_env: LanguageEnv,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
#[serde(tag = "package_type", rename_all = "lowercase")]
pub enum PackageType {
    Default(DefaultPackageTypeConfig),
    Git(GitPackageTypeConfig),
    #[default]
    Virtual,
}

impl PackageType {
    pub fn get_language_env(&self) -> Option<&LanguageEnv> {
        match self {
            PackageType::Default(config) => Some(&config.language_env),
            PackageType::Git(config) => Some(&config.language_env),
            PackageType::Virtual => None,
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct PackageFields {
    pub spec_file: PathBuf,
    pub package_name: String,
    pub version_number: Version,
    pub revision_number: String,
    pub homepage: String,
}



#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct BuildEnv {
    pub codename: Distribution,
    pub arch: Architecture,
    pub pkg_builder_version: Version,
    pub debcrafter_version: String,
    pub sbuild_cache_dir: Option<PathBuf>,
    pub docker: Option<bool>,
    pub run_lintian: Option<bool>,
    pub run_piuparts: Option<bool>,
    pub run_autopkgtest: Option<bool>,
    pub lintian_version: Version,
    pub piuparts_version: Version,
    pub autopkgtest_version: AutopkgtestVersion,
    pub sbuild_version: SbuildVersion,
    pub workdir: PathBuf,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct PkgConfig {
    pub package_fields: PackageFields,
    pub package_type: PackageType,
    pub build_env: BuildEnv,
    pub _config_root: Option<PathBuf>,
}

impl PkgConfig {
    pub fn resolve_paths(mut self, config_root: PathBuf) -> Self {
        self._config_root = Some(config_root.clone());
        // Set workdir to default if empty
        let mut default_work_dir = PathBuf::from(WORKDIR_ROOT);
        default_work_dir.push(self.build_env.codename.as_ref());

        if self.build_env.workdir.as_os_str().is_empty() {
            self.build_env.workdir = default_work_dir;
        }

        // Expand workdir path
        self.build_env.workdir = expand_path(&self.build_env.workdir, None);

        // Update spec file path to canonical form
        self.package_fields.spec_file = config_root.join(&self.package_fields.spec_file);

        self
    }
}
