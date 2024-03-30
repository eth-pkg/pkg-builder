use std::path::PathBuf;
use crate::v1::packager::{LanguageEnv, PackagerConfig};

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
    homepage: String
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
    package_type: Option<String>,
    lang_env: Option<LanguageEnv>,
    debcrafter_version: Option<String>,
    spec_file: Option<String>,
    homepage: Option<String>,
    config_root: String,
}

impl BookwormPackagerConfigBuilder {
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

    pub fn package_type(mut self, package_type: Option<String>) -> Self {
        self.package_type = package_type;
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

    pub fn config_root(mut self, config_root: String) -> Self {
        self.config_root = config_root;
        self
    }

    pub fn config(self) -> Result<BookwormPackagerConfig, String> {
        let package_type = self
            .package_type
            .ok_or_else(|| "Missing package_type field".to_string())?;

        if package_type != "virtual" && package_type != "default" && package_type != "from_git" {
            return Err("Invalid combination package_is_virtual package_is_git!".to_string());
        }
        let package_type = if package_type == "virtual" {
            PackageType::Virtual
        } else if package_type == "from_git" {
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
        } else {
            let lang_env = self
                .lang_env
                .ok_or_else(|| "Missing lang_env field".to_string())?;
            let tarball_url = self
                .tarball_url
                .ok_or_else(|| "Missing tarball_url field".to_string())?;
            PackageType::Default {
                lang_env,
                tarball_url,
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

        let package_fields = PackageFields {
            package_name,
            version_number,
            revision_number: "".to_string(),
            spec_file,
            homepage,
        };
        let build_env = BookwormBuildEnv {
            codename: "bookworm".to_string(),
            arch,
            pkg_builder_version: "".to_string(),
            debcrafter_version,
            run_lintian: false,
            run_piuparts: false,
            run_autopkgtest: false,
            workdir: "/tmp/pkg-builder".to_string(),
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
    pub fn config_root(&self) -> &PackageType {
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
        println!("build_artifacts_dir {}", build_artifacts_dir);
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
        println!("build_files_dir {}", build_files_dir);
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
    pub fn run_lintian(&self) -> bool {
        self.run_lintian
    }
    pub fn run_piuparts(&self) -> bool {
        self.run_piuparts
    }
    pub fn run_autopkgtest(&self) -> bool {
        self.run_autopkgtest
    }

    pub fn workdir(&self) -> &String {
        &self.workdir
    }
}
