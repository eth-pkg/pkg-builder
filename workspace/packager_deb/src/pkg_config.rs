use std::path::PathBuf;

use serde::Deserialize;
use types::{distribution::Distribution, url::Url, version::Version};

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
    pub gradle_version: Version,
    pub gradle_binary_url: Url,
    pub gradle_binary_checksum: String,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct JavaConfig {
    pub is_oracle: bool,
    pub jdk_version: Version,
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
    pub tarball_url: Url,
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
    pub spec_file: String,
    pub package_name: String,
    pub version_number: Version,
    pub revision_number: String,
    pub homepage: String,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct BuildEnv {
    pub codename: Distribution,
    pub arch: String,
    pub pkg_builder_version: Version,
    pub debcrafter_version: Version,
    pub sbuild_cache_dir: Option<PathBuf>,
    pub docker: Option<bool>,
    pub run_lintian: Option<bool>,
    pub run_piuparts: Option<bool>,
    pub run_autopkgtest: Option<bool>,
    pub lintian_version: Version,
    pub piuparts_version: Version,
    pub autopkgtest_version: Version,
    pub sbuild_version: Version,
    pub workdir: PathBuf,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct PkgConfig {
    pub package_fields: PackageFields,
    pub package_type: PackageType,
    pub build_env: BuildEnv,
}
