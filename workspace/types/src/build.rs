use crate::{pkg_config::PkgConfig, pkg_config_verify::PkgVerifyConfig};
use eyre::Result;

pub trait Packager {
    type BuildEnv: BackendBuildEnv;
    fn new(config: PkgConfig, config_root: String) -> Self;
    fn package(&self) -> Result<()>;
    fn get_build_env(&self) -> Result<Self::BuildEnv>;
}

pub trait BackendBuildEnv {
    fn clean(&self) -> Result<()>;
    fn create(&self) -> Result<()>;
    fn package(&self) -> Result<()>;
    fn verify(&self, verify_config: PkgVerifyConfig) -> Result<()>;
    fn run_lintian(&self) -> Result<()>;
    fn run_piuparts(&self) -> Result<()>;
    fn run_autopkgtests(&self) -> Result<()>;
}