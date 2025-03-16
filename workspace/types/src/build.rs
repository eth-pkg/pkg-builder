use crate::{pkg_config::PkgConfig, pkg_config_verify::PkgVerifyConfig};

pub trait Packager {
    type Error;
    type BuildEnv: BackendBuildEnv;
    
    fn new(config: PkgConfig, config_root: String) -> Self;
    fn package(&self) -> Result<(), Self::Error>;
    fn get_build_env(&self) -> Result<Self::BuildEnv, Self::Error>;
}

pub trait BackendBuildEnv {
    type Error;
    
    fn clean(&self) -> Result<(), Self::Error>;
    fn create(&self) -> Result<(), Self::Error>;
    fn package(&self) -> Result<(), Self::Error>;
    fn verify(&self, verify_config: PkgVerifyConfig) -> Result<(), Self::Error>;
    fn run_lintian(&self) -> Result<(), Self::Error>;
    fn run_piuparts(&self) -> Result<(), Self::Error>;
    fn run_autopkgtests(&self) -> Result<(), Self::Error>;
}