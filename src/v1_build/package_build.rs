// src/package_build.rs
use crate::config::PackageBuildConfig;
use crate::prepare_build::PrepareBuild;

pub struct PackageBuild {
    config: PackageBuildConfig,
}

impl PackageBuild {
    pub fn new(config: PackageBuildConfig) -> PackageBuild {
        PackageBuild { config }
    }

    pub fn prepare(&self) -> Result<bool, &'static str> {
        PrepareBuild::new().prepare()
    }
    pub fn guess(&self) -> Result<bool, &'static str> {
        Ok(true)
    }
    pub fn build_and_test(&self) -> Result<bool, &'static str> {
        Ok(true)
    }
    pub fn verify(&self) -> Result<bool, &'static str> {
        Ok(true)
    }
}
