// ubuntu 22.04 LTS

use crate::v1::build::sbuild::Sbuild;
use crate::v1::packager::{Packager};
use crate::v1::pkg_config::{PkgConfig};
use eyre::{Result};

#[allow(dead_code)]
pub struct JammyJellyfishPackager {
    config: PkgConfig,
    config_root: String,
}



#[allow(dead_code)]
impl Packager for JammyJellyfishPackager {
    type BuildEnv = Sbuild;

    fn new(config: PkgConfig, config_root: String) -> Self {
         JammyJellyfishPackager { config, config_root }
    }
    fn package(&self) -> Result<()> {
        todo!()
    }

    fn get_build_env(&self) -> Result<Self::BuildEnv> {
        todo!()
    }
}
