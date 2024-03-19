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
}
enum PackagerType {
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
}
pub enum PackagerError {
    InvalidCodename(String),
    MissingConfigFields(String),
    PackagingError(String)
}

impl DistributionPackager {
    pub fn new(config: DistributionPackagerConfig) -> Self {
        return DistributionPackager { config };
    }
    fn map_config(&self) -> Result<PackagerType, PackagerError> {
        let config = match self.config.codename.as_str() {
            "bookworm" | "debian 12" => BookwormPackagerConfigBuilder::new()
                .arch(self.config.arch.clone())
                .package_name(self.config.package_name.clone())
                .version_number(self.config.version_number.clone())
                .tarball_url(self.config.tarball_url.clone())
                .git_source(self.config.git_source.clone())
                .is_virtual_package(self.config.is_virtual_package)
                .is_git(self.config.is_git)
                .config()
                .map(|config| PackagerType::Bookworm(config))
                .map_err(|err| PackagerError::MissingConfigFields(err.to_string())),
            "jammy jellyfish" | "ubuntu 22.04" => JammJellyfishPackagerConfigBuilder::new()
                .arch(self.config.arch.clone())
                .package_name(self.config.package_name.clone())
                .version_number(self.config.version_number.clone())
                .tarball_url(self.config.tarball_url.clone())
                .git_source(self.config.git_source.clone())
                .is_virtual_package(self.config.is_virtual_package)
                .is_git(self.config.is_git)
                .config()
                .map(|config| PackagerType::JammyJellyfish(config))
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
            PackagerType::Bookworm(config) => {
                let packager = BookwormPackager::new(config);
                return packager
                    .package()
                    .map_err(|err| PackagerError::PackagingError(err.to_string()));
            }
            PackagerType::JammyJellyfish(config) => {
                let packager = JammyJellyfishPackager::new(config);
                return packager
                    .package()
                    .map_err(|err| PackagerError::PackagingError(err.to_string()));
            }
        };
    }
}
