use core::fmt;

use crate::v1::distribution::debian::bookworm_config_builder::{
    BookwormPackagerConfig, BookwormPackagerConfigBuilder,
};
use crate::v1::cli_config::{CliConfig};

use super::distribution::{
    debian::bookworm::BookwormPackager,
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
    config: CliConfig,
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
pub enum Error {
    InvalidCodename(String),
    MissingConfigFields(String),
    PackagingError(String),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidCodename(inner) => {
                // You can customize how you want to display this error variant
                write!(f, "Invalid codename: {}", inner)
            }
            Error::MissingConfigFields(inner) => {
                // You can customize how you want to display this error variant
                write!(f, "Missing fields: {}", inner)
            }
            Error::PackagingError(inner) => {
                // You can customize how you want to display this error variant
                write!(f, "Failed to package: {}", inner)
            }
        }
    }
}

impl std::error::Error for Error {}


impl DistributionPackager {
    pub fn new(config: CliConfig) -> Self {
         DistributionPackager { config }
    }
    fn map_config(&self) -> Result<Distribution, Error> {
        let build_env = self.config.build_env();
        let package_fields = self.config.package_fields();
        let config = match build_env.codename().clone().unwrap_or_default().as_str() {
            "bookworm" | "debian 12" => BookwormPackagerConfigBuilder::default()
                .arch(build_env.arch().clone())
                .package_name(package_fields.package_name().clone())
                .version_number(package_fields.version_number().clone())
                .tarball_url(package_fields.tarball_url().clone())
                .git_source(package_fields.git_source().clone())
                .package_type(package_fields.package_type().clone())
                .lang_env(package_fields.lang_env().clone())
                .debcrafter_version(build_env.debcrafter_version().clone())
                .spec_file(package_fields.spec_file().clone())
                .homepage(package_fields.homepage().clone())
                .config()
                .map(Distribution::Bookworm)
                .map_err(|err| Error::MissingConfigFields(err.to_string())),
            "jammy jellyfish" | "ubuntu 22.04" => JammJellyfishPackagerConfigBuilder::new()
                .arch(build_env.arch().clone())
                .package_name(package_fields.package_name().clone())
                .version_number(package_fields.version_number().clone())
                .tarball_url(package_fields.tarball_url().clone())
                .git_source(package_fields.git_source().clone())
               // .package_type(package_fields.package_type().clone())
                .lang_env(package_fields.lang_env().clone())
                .config()
                .map(Distribution::JammyJellyfish)
                .map_err(|err| Error::MissingConfigFields(err.to_string())),
            invalid_codename => {
                return Err(Error::InvalidCodename(format!(
                    "Invalid codename '{}' specified",
                    invalid_codename
                )));
            }
        };
        config
    }
    pub fn package(&self) -> Result<(), Error> {
        let distribution = self.map_config()?;

        match distribution {
            Distribution::Bookworm(config) => {
                let packager = BookwormPackager::new(config);
                packager
                    .package()
                    .map_err(|err| Error::PackagingError(err.to_string()))?;
            }
            Distribution::JammyJellyfish(config) => {
                let packager = JammyJellyfishPackager::new(config);
                packager
                    .package()
                    .map_err(|err| Error::PackagingError(err.to_string()))?;
            }
        };
        Ok(())
    }
}
