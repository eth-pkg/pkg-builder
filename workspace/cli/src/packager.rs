use build::sbuild_packager::{PackageError, SbuildPackager};
use build::sbuild::SbuildError;
use thiserror::Error;
use types::{
    build::{BackendBuildEnv, Packager},
    pkg_config::PkgConfig,
    pkg_config_verify::PkgVerifyConfig,
};

#[derive(Debug, Error)]
pub enum DistributionError {
    #[error("Invalid codename '{0}' specified")]
    InvalidCodename(String),

    #[error(transparent)]
    PackagerError(#[from] PackageError),

    #[error(transparent)]
    SbuildError(#[from] SbuildError),
}

pub struct DistributionPackager {
    config: PkgConfig,
    config_root: String,
}

impl DistributionPackager {
    pub fn new(config: PkgConfig, config_root: String) -> Self {
        Self {
            config,
            config_root,
        }
    }

    fn with_packager<F, R>(&self, operation: F) -> Result<R, DistributionError>
    where
        F: FnOnce(&SbuildPackager) -> Result<R, DistributionError>,
    {
        let codename = self.config.build_env.codename.as_str();
        match codename {
            "bookworm" | "noble numbat" | "jammy jellyfish" => {
                let packager = SbuildPackager::new(self.config.clone(), self.config_root.clone());
                operation(&packager)
            }
            _ => Err(DistributionError::InvalidCodename(codename.to_string())),
        }
    }

    fn with_build_env<F, R>(&self, operation: F) -> Result<R, DistributionError>
    where
        F: FnOnce(&<SbuildPackager as Packager>::BuildEnv) -> Result<R, DistributionError>,
    {
        self.with_packager(|packager| {
            let build_env = packager.get_build_env()?;
            operation(&build_env)
        })
    }

    pub fn package(&self) -> Result<(), DistributionError> {
        self.with_packager(|packager| Ok(packager.package()?))
    }

    pub fn run_lintian(&self) -> Result<(), DistributionError> {
        self.with_build_env(|env| Ok(env.run_lintian()?))
    }

    pub fn run_piuparts(&self) -> Result<(), DistributionError> {
        self.with_build_env(|env| Ok(env.run_piuparts()?))
    }

    pub fn run_autopkgtests(&self) -> Result<(), DistributionError> {
        self.with_build_env(|env| Ok(env.run_autopkgtests()?))
    }

    pub fn clean_build_env(&self) -> Result<(), DistributionError> {
        self.with_build_env(|env| Ok(env.clean()?))
    }

    pub fn create_build_env(&self) -> Result<(), DistributionError> {
        self.with_build_env(|env| Ok(env.create()?))
    }

    pub fn verify(
        &self,
        verify_config: PkgVerifyConfig,
        package: bool,
    ) -> Result<(), DistributionError> {
        self.with_packager(|packager| {
            if package {
                packager.package()?;
            }

            let build_env = packager.get_build_env()?;
            build_env.verify(verify_config.clone())?;
            Ok(())
        })
    }
}
