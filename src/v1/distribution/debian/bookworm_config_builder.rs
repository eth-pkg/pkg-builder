use crate::v1::packager::LanguageEnv;

pub struct NormalPackageConfig {
    pub arch: String,
    pub package_name: String,
    pub version_number: String,
    pub tarball_url: String,
    pub lang_env: LanguageEnv,
    pub debcrafter_version: String,
    pub spec_file: String,
    pub homepage: String,
}

pub struct GitPackageConfig {
    pub arch: String,
    pub package_name: String,
    pub version_number: String,
    pub git_source: String,
    pub lang_env: LanguageEnv,
    pub debcrafter_version: String,
    pub spec_file: String,
    pub homepage: String,
}

pub struct VirtualPackageConfig {
    pub arch: String,
    pub package_name: String,
    pub version_number: String,
    pub debcrafter_version: String,
    pub spec_file: String,
    pub homepage: String,
}

pub enum BookwormPackagerConfig {
    InvalidCombination,
    NormalPackage(NormalPackageConfig),
    GitPackage(GitPackageConfig),
    VirtualPackage(VirtualPackageConfig),
}

pub struct BookwormPackagerConfigBuilder {
    arch: Option<String>,
    package_name: Option<String>,
    version_number: Option<String>,
    tarball_url: Option<String>,
    git_source: Option<String>,
    package_is_virtual: bool,
    package_is_git: bool,
    lang_env: Option<LanguageEnv>,
    debcrafter_version: Option<String>,
    spec_file: Option<String>,
    homepage: Option<String>,
}

impl BookwormPackagerConfigBuilder {
    pub fn new() -> Self {
        BookwormPackagerConfigBuilder {
            arch: None,
            package_name: None,
            version_number: None,
            tarball_url: None,
            git_source: None,
            package_is_virtual: false,
            package_is_git: false,
            lang_env: None,
            debcrafter_version: None,
            spec_file: None,
            homepage: None,
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

    pub fn debcrafter_version(mut self, debcrafter_version: Option<String>) -> Self {
        self.debcrafter_version = debcrafter_version;
        self
    }

    pub fn spec_file(mut self, spec_file: Option<String>) -> Self {
        self.spec_file = spec_file;
        self
    }

    pub fn homepage(mut self, homepage: Option<String>) -> Self {
        self.homepage = homepage;
        self
    }

    pub fn config(self) -> Result<BookwormPackagerConfig, String> {
        if self.package_is_virtual && self.package_is_git {
            return Ok(BookwormPackagerConfig::InvalidCombination);
        }
        let arch = self.arch.ok_or_else(|| "Missing arch field".to_string())?;
        let package_name = self
            .package_name
            .ok_or_else(|| "Missing package_name field".to_string())?;
        let version_number = self
            .version_number
            .ok_or_else(|| "Missing version_number field".to_string())?;

        let debcrafter_version = self
            .debcrafter_version
            .ok_or_else(|| "Missing debcrafter_version field".to_string())?;
        let spec_file = self
            .spec_file
            .ok_or_else(|| "Missing spec_file field".to_string())?;
        let homepage = self
            .homepage
            .ok_or_else(|| "Missing homepage field".to_string())?;
        if self.package_is_virtual {
            let config = VirtualPackageConfig {
                arch,
                package_name,
                version_number,
                debcrafter_version,
                spec_file,
                homepage,
            };
            return Ok(BookwormPackagerConfig::VirtualPackage(config));
        }
        let lang_env = self
            .lang_env
            .ok_or_else(|| "Missing lang_env field".to_string())?;
        if self.package_is_git {
            let git_source = self
                .git_source
                .ok_or_else(|| "Missing git_source field".to_string())?;
            let config = GitPackageConfig {
                arch,
                package_name,
                version_number,
                lang_env,
                debcrafter_version,
                git_source,
                spec_file,
                homepage,
            };
            return Ok(BookwormPackagerConfig::GitPackage(config));
        } else {
            let tarball_url = self
                .tarball_url
                .ok_or_else(|| "Missing tarball_url field".to_string())?;
            let config = NormalPackageConfig {
                arch,
                package_name,
                version_number,
                tarball_url,
                lang_env,
                debcrafter_version,
                spec_file,
                homepage,
            };
            Ok(BookwormPackagerConfig::NormalPackage(config))
        }
    }
}
