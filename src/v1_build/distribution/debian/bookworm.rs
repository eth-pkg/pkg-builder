use crate::v1_build::distribution::packager::{Packager, PackagerConfig};

pub struct BookwormPackager {
    config: BookwormPackagerConfig,
}
pub struct BookwormPackagerConfig {
    arch: String,
    package_name: String,
    version_number: String,
    tarball_url: String,
    git_source: String,
    is_virtual_package: bool,
    is_git: bool,
}

impl PackagerConfig for BookwormPackagerConfig {}
pub struct BookwormPackagerConfigBuilder {
    arch: Option<String>,
    package_name: Option<String>,
    version_number: Option<String>,
    tarball_url: Option<String>,
    git_source: Option<String>,
    is_virtual_package: bool,
    is_git: bool,
}
impl BookwormPackagerConfigBuilder {
    pub fn new() -> Self {
        BookwormPackagerConfigBuilder {
            arch: None,
            package_name: None,
            version_number: None,
            tarball_url: None,
            git_source: None,
            is_virtual_package: false,
            is_git: false,
        }
    }

    pub fn arch(mut self, arch: String) -> Self {
        self.arch = Some(arch);
        self
    }

    pub fn package_name(mut self, package_name: String) -> Self {
        self.package_name = Some(package_name);
        self
    }

    pub fn version_number(mut self, version_number: String) -> Self {
        self.version_number = Some(version_number);
        self
    }

    pub fn tarball_url(mut self, tarball_url: String) -> Self {
        self.tarball_url = Some(tarball_url);
        self
    }

    pub fn git_source(mut self, git_source: String) -> Self {
        self.git_source = Some(git_source);
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

    pub fn config(self) -> Result<BookwormPackagerConfig, String> {
        let arch = self.arch.ok_or_else(|| "Missing arch field".to_string())?;
        let package_name = self.package_name.ok_or_else(|| "Missing package_name field".to_string())?;
        let version_number = self.version_number.ok_or_else(|| "Missing version_number field".to_string())?;
        let tarball_url = self.tarball_url.ok_or_else(|| "Missing tarball_url field".to_string())?;
        let git_source = self.git_source.ok_or_else(|| "Missing git_source field".to_string())?;
        
        Ok(BookwormPackagerConfig {
            arch,
            package_name,
            version_number,
            tarball_url,
            git_source,
            is_virtual_package: self.is_virtual_package,
            is_git: self.is_git,
        })
    }
}

impl Packager for BookwormPackager {
    type Config = BookwormPackagerConfig;

    fn new(config: Self::Config) -> Self {
        return BookwormPackager { config };
    }

    fn package(&self) -> Result<bool, String> {
        todo!()
    }
}
