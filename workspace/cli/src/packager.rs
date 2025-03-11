use build::sbuild_packager::SbuildPackager;
use common::{build::{BackendBuildEnv, Packager}, pkg_config::PkgConfig, pkg_config_verify::PkgVerifyConfig};
use eyre::{eyre, Result};



pub struct DistributionPackager {
    config: PkgConfig,
    config_root: String,
}

impl DistributionPackager {
    pub fn new(config: PkgConfig, config_root: String) -> Self {
        Self { config, config_root }
    }

    fn with_packager<F>(&self, operation: F) -> Result<()>
    where
        F: Fn(&SbuildPackager) -> Result<()>,
    {
        let codename = self.config.build_env.codename.as_str();
        match codename {
            "bookworm" | "noble numbat" | "jammy jellyfish" => {
                let packager = SbuildPackager::new(self.config.clone(), self.config_root.clone());
                operation(&packager)
            }
            _ => Err(eyre!("Invalid codename '{}' specified", codename)),
        }
    }

    fn with_build_env<F>(&self, operation: F) -> Result<()>
    where
        F: Fn(&<SbuildPackager as Packager>::BuildEnv) -> Result<()>,
    {
        self.with_packager(|packager| {
            let build_env = packager.get_build_env()?;
            operation(&build_env)
        })
    }

    pub fn package(&self) -> Result<()> {
        self.with_packager(|packager| packager.package())
    }

    pub fn run_lintian(&self) -> Result<()> {
        self.with_build_env(|env| env.run_lintian())
    }

    pub fn run_piuparts(&self) -> Result<()> {
        self.with_build_env(|env| env.run_piuparts())
    }

    pub fn run_autopkgtests(&self) -> Result<()> {
        self.with_build_env(|env| env.run_autopkgtests())
    }

    pub fn clean_build_env(&self) -> Result<()> {
        self.with_build_env(|env| env.clean())
    }

    pub fn create_build_env(&self) -> Result<()> {
        self.with_build_env(|env| env.create())
    }

    pub fn verify(&self, verify_config: PkgVerifyConfig, package: bool) -> Result<()> {
        self.with_packager(|packager| {
            if package {
                packager.package()?;
            }
            let build_env = packager.get_build_env()?;
            build_env.verify(verify_config.clone())
        })
    }
}