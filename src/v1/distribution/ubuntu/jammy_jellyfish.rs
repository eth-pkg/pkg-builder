// ubuntu 22.04 LTS

use thiserror::Error;
use crate::v1::build::sbuild::Sbuild;
use crate::v1::packager::{LanguageEnv, Packager, PackagerConfig};
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
    lang_env: LanguageEnv,
}

impl PackagerConfig for JammyJellyfishPackagerConfig {}

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
impl JammJellyfishPackagerConfigBuilder {

    pub fn arch(mut self, arch: Option<String>) -> Self {
        self.arch = arch;
        self
    }

    pub fn package_name(mut self, package_name: Option<String>) -> Self {
        self.package_name = package_name;
        self
    }

    pub fn version_number(mut self, version_number: Option<String>) -> Self {
        self.version_number = version_number;
        self
    }

    pub fn tarball_url(mut self, tarball_url: Option<String>) -> Self {
        self.tarball_url = tarball_url;
        self
    }

    pub fn git_source(mut self, git_source: Option<String>) -> Self {
        self.git_source = git_source;
        self
    }

    pub fn package_is_virtual(mut self, package_is_virtual: bool) -> Self {
        self.package_is_virtual = package_is_virtual;
        self
    }

    pub fn package_is_git(mut self, package_is_git: bool) -> Self {
        self.package_is_git = package_is_git;
        self
    }

    pub fn lang_env(mut self, lang_env: Option<String>) -> Self {
        self.lang_env = LanguageEnv::from_string(&lang_env.unwrap_or_default());
        self
    }

    pub fn config(self) -> Result<JammyJellyfishPackagerConfig, String> {
        let arch = self.arch.ok_or_else(|| "Missing arch field".to_string())?;
        let package_name = self
            .package_name
            .ok_or_else(|| "Missing package_name field".to_string())?;
        let version_number = self
            .version_number
            .ok_or_else(|| "Missing version_number field".to_string())?;
        let tarball_url = self
            .tarball_url
            .ok_or_else(|| "Missing tarball_url field".to_string())?;
        let git_source = self
            .git_source
            .ok_or_else(|| "Missing git_source field".to_string())?;
        let lang_env = self
            .lang_env
            .ok_or_else(|| "Missing lang_env field".to_string())?;

        Ok(JammyJellyfishPackagerConfig {
            arch,
            package_name,
            version_number,
            tarball_url,
            git_source,
            package_is_virtual: self.package_is_virtual,
            package_is_git: self.package_is_git,
            lang_env,
        })
    }
}
#[derive(Debug, Error)]
pub enum Error {}

impl Packager for JammyJellyfishPackager {
    type Error = Error;
    type Config = JammyJellyfishPackagerConfig;
    type BuildEnv = Sbuild;

    fn new(config: Self::Config) -> Self {
         JammyJellyfishPackager { config }
    }
    fn package(&self) -> Result<(), self::Error> {
        todo!()
    }

    fn get_build_env(&self) -> Result<Self::BuildEnv, Self::Error> {
        todo!()
    }
}
