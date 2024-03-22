// ubuntu 22.04 LTS

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
    is_virtual_package: bool,
    is_git: bool,
    lang_env: LanguageEnv,
}

impl PackagerConfig for JammyJellyfishPackagerConfig {}
pub struct JammJellyfishPackagerConfigBuilder {
    arch: Option<String>,
    package_name: Option<String>,
    version_number: Option<String>,
    tarball_url: Option<String>,
    git_source: Option<String>,
    is_virtual_package: bool,
    is_git: bool,
    lang_env: Option<LanguageEnv>,
}
impl JammJellyfishPackagerConfigBuilder {
    pub fn new() -> Self {
        JammJellyfishPackagerConfigBuilder {
            arch: None,
            package_name: None,
            version_number: None,
            tarball_url: None,
            git_source: None,
            is_virtual_package: false,
            is_git: false,
            lang_env: None,
        }
    }

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

    pub fn is_virtual_package(mut self, is_virtual_package: bool) -> Self {
        self.is_virtual_package = is_virtual_package;
        self
    }

    pub fn is_git(mut self, is_git: bool) -> Self {
        self.is_git = is_git;
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
            is_virtual_package: self.is_virtual_package,
            is_git: self.is_git,
            lang_env,
        })
    }
}
impl Packager for JammyJellyfishPackager {
    type Config = JammyJellyfishPackagerConfig;

    fn new(config: Self::Config) -> Self {
        return JammyJellyfishPackager { config };
    }
    fn package(&self) -> Result<(), String> {
        todo!()
    }
}
