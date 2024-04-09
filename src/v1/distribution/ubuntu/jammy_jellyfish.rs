// ubuntu 22.04 LTS

use crate::v1::build::sbuild::Sbuild;
use crate::v1::packager::{Packager, PackagerConfig};
use crate::v1::pkg_config::LanguageEnv;
use eyre::{Result};

#[allow(dead_code)]
pub struct JammyJellyfishPackager {
    config: JammyJellyfishPackagerConfig,
}
#[allow(dead_code)]
pub struct JammyJellyfishPackagerConfig {
    arch: String,
    package_name: String,
    version_number: String,
    tarball_url: String,
    git_source: String,
    package_is_virtual: bool,
    package_is_git: bool,
    lang_env: crate::v1::pkg_config::LanguageEnv,
}
#[allow(dead_code)]
impl PackagerConfig for JammyJellyfishPackagerConfig {}

#[allow(dead_code)]
#[derive(Default)]
pub struct JammJellyfishPackagerConfigBuilder {
    arch: Option<String>,
    package_name: Option<String>,
    version_number: Option<String>,
    tarball_url: Option<String>,
    git_source: Option<String>,
    package_is_virtual: bool,
    package_is_git: bool,
    lang_env: Option<LanguageEnv>,
}

#[allow(dead_code)]
impl Packager for JammyJellyfishPackager {
    type Config = JammyJellyfishPackagerConfig;
    type BuildEnv = Sbuild;

    fn new(config: Self::Config) -> Self {
         JammyJellyfishPackager { config }
    }
    fn package(&self) -> Result<()> {
        todo!()
    }

    fn get_build_env(&self) -> Result<Self::BuildEnv> {
        todo!()
    }
}
