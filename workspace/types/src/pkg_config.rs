use serde::{Deserialize, Deserializer};
use std::str::FromStr;

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
pub struct RustConfig {
    pub rust_version: String,
    pub rust_binary_url: String,
    pub rust_binary_gpg_asc: String,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
pub struct GoConfig {
    pub go_version: String,
    pub go_binary_url: String,
    pub go_binary_checksum: String,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
pub struct JavascriptConfig {
    pub node_version: String,
    pub node_binary_url: String,
    pub node_binary_checksum: String,
    pub yarn_version: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
pub struct GradleConfig {
    pub gradle_version: String,
    pub gradle_binary_url: String,
    pub gradle_binary_checksum: String,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
pub struct JavaConfig {
    pub is_oracle: bool,
    pub jdk_version: String,
    pub jdk_binary_url: String,
    pub jdk_binary_checksum: String,
    pub gradle: Option<GradleConfig>,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
pub struct DotnetPackage {
    pub name: String,
    pub hash: String,
    pub url: String,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
pub struct DotnetConfig {
    pub use_backup_version: bool,
    pub dotnet_packages: Vec<DotnetPackage>,
    pub deps: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
pub struct NimConfig {
    pub nim_version: String,
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

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
pub struct DefaultPackageTypeConfig {
    pub tarball_url: String,
    pub tarball_hash: Option<String>,
    pub language_env: LanguageEnv,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
pub struct SubModule {
    pub commit: String,
    pub path: String,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
pub struct GitPackageTypeConfig {
    pub git_tag: String,
    pub git_url: String,
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

#[derive(Debug, Deserialize, PartialEq, Default, Clone)]
pub struct PackageFields {
    pub spec_file: String,
    pub package_name: String,
    pub version_number: String,
    pub revision_number: String,
    pub homepage: String,
}

#[derive(Debug, Deserialize, PartialEq, Default, Clone)]
pub struct BuildEnv {
    pub codename: String,
    pub arch: String,
    pub pkg_builder_version: String,
    pub debcrafter_version: String,
    pub sbuild_cache_dir: Option<String>,
    pub docker: Option<bool>,
    pub run_lintian: Option<bool>,
    pub run_piuparts: Option<bool>,
    pub run_autopkgtest: Option<bool>,
    pub lintian_version: String,
    pub piuparts_version: String,
    pub autopkgtest_version: String,
    pub sbuild_version: String,
    #[serde(deserialize_with = "deserialize_option_empty_string")]
    pub workdir: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
pub struct PkgConfig {
    pub package_fields: PackageFields,
    pub package_type: PackageType,
    pub build_env: BuildEnv,
}

pub fn deserialize_option_empty_string<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: FromStr,
    T::Err: std::fmt::Display,
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        T::from_str(&s).map(Some).map_err(serde::de::Error::custom)
    }
}
