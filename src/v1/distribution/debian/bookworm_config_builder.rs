use crate::v1::cli_config::{CliBuildEnv, CliPackageFields};
use crate::v1::packager::{LanguageEnv, PackagerConfig};
use log::info;
use std::path::PathBuf;
use std::{env, fs, io};

#[derive(Clone)]
pub struct BookwormPackagerConfig {
    package_fields: PackageFields,
    build_env: BookwormBuildEnv,
    package_type: PackageType,
}

impl PackagerConfig for BookwormPackagerConfig {}

#[derive(Clone)]
pub struct PackageFields {
    package_name: String,
    version_number: String,
    revision_number: String,
    spec_file: String,
    homepage: String,
    src_dir: String,
}
#[derive(Clone)]
pub enum PackageType {
    Default {
        tarball_url: String,
        lang_env: LanguageEnv,
    },
    Git {
        git_source: String,
        lang_env: LanguageEnv,
    },
    Virtual,
}

impl PackageType {
    // This should be only used for constructing for Builder
    fn from_string(package_type: &str) -> Option<Self> {
        match package_type.to_lowercase().as_str() {
            "virtual" => Some(PackageType::Virtual),
            "default" => Some(PackageType::Default {
                tarball_url: "".to_string(),
                lang_env: LanguageEnv::Go,
            }),
            "from_git" => Some(PackageType::Git {
                git_source: "".to_string(),
                lang_env: LanguageEnv::Go,
            }),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct BookwormBuildEnv {
    codename: String,
    arch: String,
    pkg_builder_version: String,
    debcrafter_version: String,
    run_lintian: bool,
    run_piuparts: bool,
    run_autopkgtest: bool,
    workdir: String,
}
#[derive(Default)]
pub struct BookwormPackagerConfigBuilder {
    arch: Option<String>,
    package_name: Option<String>,
    version_number: Option<String>,
    tarball_url: Option<String>,
    git_source: Option<String>,
    package_type: Option<PackageType>,
    lang_env: Option<LanguageEnv>,
    debcrafter_version: Option<String>,
    spec_file: Option<String>,
    homepage: Option<String>,
    config_root: String,
    pkg_builder_version: Option<String>,
    run_piuparts: bool,
    run_lintian: bool,
    run_autopkgtests: bool,
    workdir: Option<String>,
}

impl BookwormPackagerConfigBuilder {
    pub fn build_env(mut self, build_env: &CliBuildEnv) -> Self {
        self.arch = build_env.arch().clone();
        self.pkg_builder_version = build_env.pkg_builder_version().clone();
        self.debcrafter_version = build_env.debcrafter_version().clone();
        self.run_lintian = build_env.run_lintian();
        self.run_autopkgtests = build_env.run_autopkgtest();
        self.run_piuparts = build_env.run_piuparts();
        self.lang_env = LanguageEnv::from_string(&build_env.lang_env().clone().unwrap_or_default());
        self.workdir = build_env.workdir().clone();
        self
    }
    pub fn package_fields(mut self, package_fields: &CliPackageFields) -> Self {
        self.package_name = package_fields.package_name().clone();
        self.version_number = package_fields.version_number().clone();
        self.tarball_url = package_fields.tarball_url().clone();
        self.git_source = package_fields.git_source().clone();
        self.package_type =
            PackageType::from_string(&package_fields.package_type().clone().unwrap_or_default());
        self.spec_file = package_fields.spec_file().clone();
        self.homepage = package_fields.homepage().clone();
        self
    }

    pub fn config_root(mut self, config_root: String) -> Self {
        self.config_root = config_root;
        self
    }

    pub fn config(self) -> Result<BookwormPackagerConfig, String> {
        let package_type = self
            .package_type
            .ok_or_else(|| "Missing package_type field or invalid argument".to_string())?;

        let package_type = match package_type {
            PackageType::Virtual => package_type,
            PackageType::Git { .. } => {
                let lang_env = self
                    .lang_env
                    .ok_or_else(|| "Missing lang_env field".to_string())?;
                let git_source = self
                    .git_source
                    .ok_or_else(|| "Missing git_source field".to_string())?;
                PackageType::Git {
                    lang_env,
                    git_source,
                }
            }
            PackageType::Default { .. } => {
                let lang_env = self
                    .lang_env
                    .ok_or_else(|| "Missing lang_env field".to_string())?;
                let tarball_url = self
                    .tarball_url
                    .ok_or_else(|| "Missing tarball_url field".to_string())?;
                let is_web = tarball_url.starts_with("http");
                let tarball_url = match is_web {
                    true => tarball_url,
                    false => {
                        let config_root_path = PathBuf::from(&self.config_root);
                        let tarball_url = config_root_path.join(tarball_url);
                        tarball_url.to_str().unwrap().to_string()
                    }
                };
                PackageType::Default {
                    lang_env,
                    tarball_url,
                }
            }
        };
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

        let config_root_path = PathBuf::from(&self.config_root);
        let spec_file_canonical = config_root_path.join(spec_file);
        let spec_file = spec_file_canonical.to_str().unwrap().to_string();
        let src_dir = config_root_path.join("src").to_str().unwrap().to_string();
        let workdir = self
            .workdir
            .ok_or("~/.pkg-builder/packages/bookworm")
            .unwrap();
        let workdir = expand_path(&workdir);

        let package_fields = PackageFields {
            package_name,
            version_number,
            revision_number: "".to_string(),
            spec_file,
            homepage,
            src_dir,
        };
        let build_env = BookwormBuildEnv {
            codename: "bookworm".to_string(),
            arch,
            pkg_builder_version: "".to_string(),
            debcrafter_version,
            run_lintian: self.run_lintian,
            run_piuparts: self.run_piuparts,
            run_autopkgtest: self.run_autopkgtests,
            workdir,
        };
        let config = BookwormPackagerConfig {
            package_fields,
            build_env,
            package_type,
        };
        Ok(config)
    }
}
impl BookwormPackagerConfig {
    pub fn package_fields(&self) -> &PackageFields {
        &self.package_fields
    }
    pub fn build_env(&self) -> &BookwormBuildEnv {
        &self.build_env
    }
    pub fn package_type(&self) -> &PackageType {
        &self.package_type
    }
    pub fn lang_env(&self) -> Option<LanguageEnv> {
        match self.package_type {
            PackageType::Default { lang_env, .. } => Some(lang_env),
            PackageType::Git { lang_env, .. } => Some(lang_env),
            PackageType::Virtual => None,
        }
    }
    pub fn build_artifacts_dir(&self) -> String {
        let package_name = self.package_fields().package_name();
        let build_artifacts_dir = format!("{}/{}", self.build_env().workdir(), &package_name);
        build_artifacts_dir
    }
    pub fn tarball_path(&self) -> String {
        let package_name = self.package_fields().package_name();
        let version_number = self.package_fields().version_number();
        let build_artifacts_dir = self.build_artifacts_dir();
        let tarball_path = format!(
            "{}/{}_{}.orig.tar.gz",
            &build_artifacts_dir, &package_name, &version_number
        );
        tarball_path
    }
    pub fn build_files_dir(&self) -> String {
        let package_name = self.package_fields().package_name();
        let version_number = self.package_fields().version_number();
        let build_artifacts_dir = self.build_artifacts_dir();
        let build_files_dir = format!(
            "{}/{}-{}",
            build_artifacts_dir, &package_name, &version_number
        );
        build_files_dir
    }
}

impl PackageFields {
    pub fn package_name(&self) -> &String {
        &self.package_name
    }
    pub fn version_number(&self) -> &String {
        &self.version_number
    }

    pub fn revision_number(&self) -> &String {
        &self.revision_number
    }
    pub fn spec_file(&self) -> &String {
        &self.spec_file
    }
    pub fn homepage(&self) -> &String {
        &self.homepage
    }
    pub fn src_dir(&self) -> &String {
        &self.src_dir
    }
}
impl BookwormBuildEnv {
    pub fn codename(&self) -> &String {
        &self.codename
    }
    pub fn arch(&self) -> &String {
        &self.arch
    }

    pub fn pkg_builder_version(&self) -> &String {
        &self.pkg_builder_version
    }
    pub fn debcrafter_version(&self) -> &String {
        &self.debcrafter_version
    }
    pub fn run_lintian(&self) -> &bool {
        &self.run_lintian
    }
    pub fn run_piuparts(&self) -> &bool {
        &self.run_piuparts
    }
    pub fn run_autopkgtest(&self) -> &bool {
        &self.run_autopkgtest
    }

    pub fn workdir(&self) -> &String {
        &self.workdir
    }
}

fn expand_path(dir: &str) -> String {
    if dir.starts_with('~') {
        let expanded_path = shellexpand::tilde(dir).to_string();
        expanded_path
    } else {
        let parent_dir = env::current_dir().unwrap();
        let dir = parent_dir.join(dir);
        let path = fs::canonicalize(dir.clone()).unwrap();
        let path = path.to_str().unwrap().to_string();
        path
    }
}
