use serde::Deserialize;

use super::{
    debian::bookworm::{BookwormPackager, BookwormPackagerConfig, BookwormPackagerConfigBuilder},
    ubuntu::jammy_jellyfish::{
        JammJellyfishPackagerConfigBuilder, JammyJellyfishPackager, JammyJellyfishPackagerConfig,
    },
};

pub trait PackagerConfig {}

pub trait Packager {
    type Config: PackagerConfig;
    fn new(config: Self::Config) -> Self;
    fn package(&self) -> Result<bool, String>;
    fn create_build_env(&self) -> Result<Box<dyn BackendBuildEnv>, String>;
}
enum Distribution {
    Bookworm(BookwormPackagerConfig),
    JammyJellyfish(JammyJellyfishPackagerConfig),
}

pub struct DistributionPackager {
    config: DistributionPackagerConfig,
}

#[derive(Debug, Deserialize)]
pub struct DistributionPackagerConfig {
    codename: String,
    arch: String,
    package_name: String,
    version_number: String,
    tarball_url: String,
    git_source: String,
    is_virtual_package: bool,
    is_git: bool,
    lang_env: String,
}

pub struct BuildConfig {
    codename: String,
    arch: String,
    lang_env: LanguageEnv,
}

impl BuildConfig {
    pub fn new(codename: &str, arch: &str, lang_env: LanguageEnv) -> Self {
        return BuildConfig {
            codename: codename.to_string(),
            arch: arch.to_string(),
            lang_env,
        };
    }
    pub fn codename(&self) -> &String {
        &self.codename
    }
    pub fn arch(&self) -> &String {
        &self.arch
    }
    pub fn lang_env(&self) -> &LanguageEnv {
        &self.lang_env
    }
}

pub trait BackendBuildEnv {
    fn clean(&self) -> Result<(), String>;
    fn create(&self) -> Result<(), String>;
    fn build(&self) -> Result<(), String>;
}

#[derive(Debug, Copy, Clone)]
pub enum LanguageEnv {
    Rust,
    Go,
    JavaScript,
    Java,
    CSharp,
    TypeScript,
    Zig,
}

impl LanguageEnv {
    pub fn from_string(lang_env: &str) -> Option<Self> {
        match lang_env.to_lowercase().as_str() {
            "rust" => Some(LanguageEnv::Rust),
            "go" => Some(LanguageEnv::Go),
            "javascript" => Some(LanguageEnv::JavaScript),
            "java" => Some(LanguageEnv::Java),
            "csharp" => Some(LanguageEnv::CSharp),
            "typescript" => Some(LanguageEnv::TypeScript),
            "zig" => Some(LanguageEnv::Zig),
            _ => None,
        }
    }
}

pub enum PackagerError {
    InvalidCodename(String),
    MissingConfigFields(String),
    PackagingError(String),
}

impl DistributionPackager {
    pub fn new(config: DistributionPackagerConfig) -> Self {
        return DistributionPackager { config };
    }
    fn map_config(&self) -> Result<Distribution, PackagerError> {
        let config = match self.config.codename.as_str() {
            "bookworm" | "debian 12" => BookwormPackagerConfigBuilder::new()
                .arch(self.config.arch.clone())
                .package_name(self.config.package_name.clone())
                .version_number(self.config.version_number.clone())
                .tarball_url(self.config.tarball_url.clone())
                .git_source(self.config.git_source.clone())
                .is_virtual_package(self.config.is_virtual_package)
                .is_git(self.config.is_git)
                .lang_env(self.config.lang_env.clone())
                .config()
                .map(|config| Distribution::Bookworm(config))
                .map_err(|err| PackagerError::MissingConfigFields(err.to_string())),
            "jammy jellyfish" | "ubuntu 22.04" => JammJellyfishPackagerConfigBuilder::new()
                .arch(self.config.arch.clone())
                .package_name(self.config.package_name.clone())
                .version_number(self.config.version_number.clone())
                .tarball_url(self.config.tarball_url.clone())
                .git_source(self.config.git_source.clone())
                .is_virtual_package(self.config.is_virtual_package)
                .is_git(self.config.is_git)
                .lang_env(self.config.lang_env.clone())
                .config()
                .map(|config| Distribution::JammyJellyfish(config))
                .map_err(|err| PackagerError::MissingConfigFields(err.to_string())),
            invalid_codename => {
                return Err(PackagerError::InvalidCodename(format!(
                    "Invalid codename '{}' specified",
                    invalid_codename
                )));
            }
        };
        return config;
    }
    pub fn package(&self) -> Result<bool, PackagerError> {
        let packager_type = self.map_config()?;

        match packager_type {
            // Match on specific types of PackagerConfig
            Distribution::Bookworm(config) => {
                let packager = BookwormPackager::new(config);
                return packager
                    .package()
                    .map_err(|err| PackagerError::PackagingError(err.to_string()));
            }
            Distribution::JammyJellyfish(config) => {
                let packager = JammyJellyfishPackager::new(config);
                return packager
                    .package()
                    .map_err(|err| PackagerError::PackagingError(err.to_string()));
            }
        };
    }
}
