use crate::{pkg_config::PkgConfig, pkg_config_verify::PkgVerifyConfig};

pub trait Packager {
    type Error;

    fn new(config: PkgConfig, config_root: String) -> Self;
    fn clean(&self) -> Result<(), Self::Error>;
    fn create(&self) -> Result<(), Self::Error>;
    fn package(&self) -> Result<(), Self::Error>;
    fn verify(&self, verify_config: PkgVerifyConfig, no_package: bool) -> Result<(), Self::Error>;
}
