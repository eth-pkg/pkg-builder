use core::fmt;

use serde::Deserialize;
use crate::v1::distribution::debian::bookworm_config_builder::{BookwormPackagerConfig, BookwormPackagerConfigBuilder};

use super::distribution::{
    debian::bookworm::{BookwormPackager},
    ubuntu::jammy_jellyfish::{
        JammJellyfishPackagerConfigBuilder, JammyJellyfishPackager, JammyJellyfishPackagerConfig,
    },
};

pub trait PackagerConfig {}

pub trait Packager {
    type Config: PackagerConfig;
    fn new(config: Self::Config) -> Self;
    fn package(&self) -> Result<(), String>;

    fn get_build_env(&self) -> Result<Box<dyn BackendBuildEnv>, String>;
}
pub enum Distribution {
    Bookworm(BookwormPackagerConfig),
    JammyJellyfish(JammyJellyfishPackagerConfig),
}

pub struct DistributionPackager {
    config: DistributionPackagerConfig,
}

#[derive(Debug, Deserialize)]
pub struct DistributionPackagerConfig {
    codename: Option<String>,
    arch: Option<String>,
    package_name: Option<String>,
    version_number: Option<String>,
    tarball_url: Option<String>,
    git_source: Option<String>,
    package_is_virtual: bool,
    package_is_git: bool,
    lang_env: Option<String>,
    debcrafter_version: Option<String>,
    spec_file: Option<String>
}

pub struct BuildConfig {
    codename: String,
    arch: String,
    lang_env: Option<LanguageEnv>,
    package_dir: String,
}

impl BuildConfig {
    pub fn new(codename: &str, arch: &str, lang_env: Option<LanguageEnv>, package_dir: &String) -> Self {
        return BuildConfig {
            codename: codename.to_string(),
            arch: arch.to_string(),
            lang_env,
            package_dir: package_dir.to_string()
        };
    }
    pub fn codename(&self) -> &String {
        &self.codename
    }
    pub fn arch(&self) -> &String {
        &self.arch
    }
    pub fn lang_env(&self) -> &Option<LanguageEnv> {
        &self.lang_env
    }
    pub fn package_dir(&self) -> &String {
        &self.package_dir
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

impl fmt::Display for LanguageEnv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LanguageEnv::Rust => write!(f, "rust"),
            LanguageEnv::Go => write!(f, "go"),
            LanguageEnv::JavaScript => write!(f, "javascript"),
            LanguageEnv::Java => write!(f, "java"),
            LanguageEnv::CSharp => write!(f, "csharp"),
            LanguageEnv::TypeScript => write!(f, "typescript"),
            LanguageEnv::Zig => write!(f, "zig"),
        }
    }
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

#[derive(Debug)]
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
        let config = match self.config.codename.clone().unwrap_or_default().as_str() {
            "bookworm" | "debian 12" => BookwormPackagerConfigBuilder::new()
                .arch(self.config.arch.clone())
                .package_name(self.config.package_name.clone())
                .version_number(self.config.version_number.clone())
                .tarball_url(self.config.tarball_url.clone())
                .git_source(self.config.git_source.clone())
                .package_is_virtual(self.config.package_is_virtual)
                .package_is_git(self.config.package_is_git)
                .lang_env(self.config.lang_env.clone())
                .debcrafter_version(self.config.debcrafter_version.clone())
                .spec_file(self.config.spec_file.clone())
                .config()
                .map(|config| Distribution::Bookworm(config))
                .map_err(|err| PackagerError::MissingConfigFields(err.to_string())),
            "jammy jellyfish" | "ubuntu 22.04" => JammJellyfishPackagerConfigBuilder::new()
                .arch(self.config.arch.clone())
                .package_name(self.config.package_name.clone())
                .version_number(self.config.version_number.clone())
                .tarball_url(self.config.tarball_url.clone())
                .git_source(self.config.git_source.clone())
                .package_is_virtual(self.config.package_is_virtual)
                .package_is_git(self.config.package_is_git)
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
    pub fn package(&self) -> Result<(), PackagerError> {
        let distribution = self.map_config()?;

        return match distribution {
            Distribution::Bookworm(config) => {
                let packager = BookwormPackager::new(config);
                packager
                    .package()
                    .map_err(|err| PackagerError::PackagingError(err.to_string()))
            }
            Distribution::JammyJellyfish(config) => {
                let packager = JammyJellyfishPackager::new(config);
                packager
                    .package()
                    .map_err(|err| PackagerError::PackagingError(err.to_string()))
            }
        };
    }
}
