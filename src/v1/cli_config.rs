use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
pub struct CliConfig {
    package_fields: CliPackageFields,
    build_env: CliBuildEnv,
    cli_options: CliOptionsConfig,
    verify: CliVerifyConfig,
}
impl CliConfig {
    pub fn package_fields(&self) -> &CliPackageFields {
        &self.package_fields
    }
    pub fn build_env(&self) -> &CliBuildEnv {
        &self.build_env
    }

    pub fn cli_options(&self) -> &CliOptionsConfig {
        &self.cli_options
    }
    pub fn verify(&self) -> &CliVerifyConfig {
        &self.verify
    }
}
#[derive(Debug)]
pub struct CliPackageFields {
    package_name: Option<String>,
    version_number: Option<String>,
    revision_number: Option<String>,
    tarball_url: Option<String>,
    git_source: Option<String>,
    package_type: Option<String>,
    spec_file: Option<String>,
    homepage: Option<String>,
}

impl CliPackageFields {
    pub fn package_name(&self) -> &Option<String> {
        &self.package_name
    }
    pub fn version_number(&self) -> &Option<String> {
        &self.version_number
    }

    pub fn revision_number(&self) -> &Option<String> {
        &self.revision_number
    }
    pub fn tarball_url(&self) -> &Option<String> {
        &self.tarball_url
    }
    pub fn git_source(&self) -> &Option<String> {
        &self.git_source
    }
    pub fn package_type(&self) -> &Option<String> {
        &self.package_type
    }
    pub fn spec_file(&self) -> &Option<String> {
        &self.spec_file
    }
    pub fn homepage(&self) -> &Option<String> {
        &self.homepage
    }
}

fn deserialize_option_string(value: Option<String>) -> Option<String> {
    match value {
        Some(s) if s.is_empty() => None,
        _ => value,
    }
}
fn deserialize_option_bool(value: Option<bool>) -> bool {
    match value {
        Some(value) => value,
        _ => false,
    }
}

impl<'de> Deserialize<'de> for CliPackageFields {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawPackageFields {
            package_name: Option<String>,
            version_number: Option<String>,
            revision_number: Option<String>,
            tarball_url: Option<String>,
            git_source: Option<String>,
            package_type: Option<String>,
            spec_file: Option<String>,
            homepage: Option<String>,
        }

        let raw_package = RawPackageFields::deserialize(deserializer)?;

        Ok(CliPackageFields {
            package_name: deserialize_option_string(raw_package.package_name),
            version_number: deserialize_option_string(raw_package.version_number),
            git_source: deserialize_option_string(raw_package.git_source),
            homepage: deserialize_option_string(raw_package.homepage),
            package_type: deserialize_option_string(raw_package.package_type),
            revision_number: deserialize_option_string(raw_package.revision_number),
            spec_file: deserialize_option_string(raw_package.spec_file),
            tarball_url: deserialize_option_string(raw_package.tarball_url),
        })
    }
}

#[derive(Debug)]
pub struct CliBuildEnv {
    codename: Option<String>,
    arch: Option<String>,
    pkg_builder_version: Option<String>,
    debcrafter_version: Option<String>,
    run_lintian: bool,
    run_piuparts: bool,
    run_autopkgtest: bool,
    workdir: Option<String>,
    lang_env: Option<String>,

}

impl CliBuildEnv {
    pub fn codename(&self) -> &Option<String> {
        &self.codename
    }
    pub fn arch(&self) -> &Option<String> {
        &self.arch
    }

    pub fn pkg_builder_version(&self) -> &Option<String> {
        &self.pkg_builder_version
    }
    pub fn debcrafter_version(&self) -> &Option<String> {
        &self.debcrafter_version
    }
    pub fn run_lintian(&self) -> bool {
        self.run_lintian
    }
    pub fn run_piuparts(&self) -> bool {
        self.run_piuparts
    }
    pub fn run_autopkgtest(&self) -> bool {
        self.run_autopkgtest
    }

    pub fn workdir(&self) -> &Option<String> {
        &self.workdir
    }
    pub fn lang_env(&self) -> &Option<String> {
        &self.lang_env
    }

}

impl<'de> Deserialize<'de> for CliBuildEnv {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawBuildEnv {
            codename: Option<String>,
            arch: Option<String>,
            pkg_builder_version: Option<String>,
            debcrafter_version: Option<String>,
            run_lintian: Option<bool>,
            run_piuparts: Option<bool>,
            run_autopkgtest: Option<bool>,
            workdir: Option<String>,
            lang_env: Option<String>,

        }

        let raw_package = RawBuildEnv::deserialize(deserializer)?;

        Ok(CliBuildEnv {
            codename: deserialize_option_string(raw_package.codename),
            arch: deserialize_option_string(raw_package.arch),
            pkg_builder_version: deserialize_option_string(raw_package.pkg_builder_version),
            debcrafter_version: deserialize_option_string(raw_package.debcrafter_version),
            run_lintian: deserialize_option_bool(raw_package.run_lintian),
            run_piuparts: deserialize_option_bool(raw_package.run_piuparts),
            run_autopkgtest: deserialize_option_bool(raw_package.run_autopkgtest),
            workdir: deserialize_option_string(raw_package.workdir),
            lang_env: deserialize_option_string(raw_package.lang_env),

        })
    }
}

#[derive(Debug)]
pub struct CliOptionsConfig {
    is_ci: bool,
    log: Option<String>,
    log_to: Option<String>,
    reuse_previous_build: bool,
}
impl<'de> Deserialize<'de> for CliOptionsConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawCliOptionsConfig {
            is_ci: Option<bool>,
            log: Option<String>,
            log_to: Option<String>,
            reuse_previous_build: Option<bool>,
        }

        let raw_package = RawCliOptionsConfig::deserialize(deserializer)?;

        Ok(CliOptionsConfig {
            is_ci: deserialize_option_bool(raw_package.is_ci),
            log: deserialize_option_string(raw_package.log),
            log_to: deserialize_option_string(raw_package.log_to),
            reuse_previous_build: deserialize_option_bool(raw_package.reuse_previous_build),
        })
    }
}
#[derive(Debug)]
pub struct CliVerifyConfig {
    tarball_hash: Option<String>,
    git_commit: Option<String>,
    bin_bash: Option<String>,
}
impl<'de> Deserialize<'de> for CliVerifyConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawVerifyConfig {
            tarball_hash: Option<String>,
            git_commit: Option<String>,
            bin_bash: Option<String>,
        }

        let raw_package = RawVerifyConfig::deserialize(deserializer)?;

        Ok(CliVerifyConfig {
            tarball_hash: deserialize_option_string(raw_package.tarball_hash),
            git_commit: deserialize_option_string(raw_package.git_commit),
            bin_bash: deserialize_option_string(raw_package.bin_bash),
        })
    }
}
