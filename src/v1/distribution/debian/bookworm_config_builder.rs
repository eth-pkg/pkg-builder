use crate::v1::pkg_config::{PkgConfig, PackageFields, PackageType, BuildEnv};
use eyre::{Result};
use std::path::PathBuf;
use std::{env, fs};
use crate::v1::packager::PackagerConfig;

#[derive(Clone)]
pub struct DerivedFields {
    pub spec_file: String,
    pub workdir: String,
    pub build_artifacts_dir: String,
    pub tarball_path: String,
    pub build_files_dir: String,
    pub src_dir: String,
}
#[derive(Clone)]
pub struct BookwormPackagerConfig {
    pub read_config: PkgConfig,
    pub derived_fields: DerivedFields
}
impl PackagerConfig for BookwormPackagerConfig{}
pub struct BookwormPackagerConfigBuilder {
    config: PkgConfig,
    workdir: Option<String>,
    config_root: String,
}

impl BookwormPackagerConfigBuilder {
    pub fn new() -> Self {
        BookwormPackagerConfigBuilder{
            workdir: None,
            config_root: "".to_string(),
            config: PkgConfig{
                package_fields: PackageFields::default(),
                package_type: PackageType::Virtual,
                build_env: BuildEnv::default(),
                cli_options: None,
                verify: None,
            }
        }
    }
    pub fn config(mut self, pkg_config: PkgConfig) -> Self {
        self.config = pkg_config;
        self
    }

    pub fn config_root(mut self, config_root: String) -> Self {
        self.config_root = config_root;
        self
    }

    pub fn build(self) -> Result<BookwormPackagerConfig> {
        let package_fields = self.config.package_fields.clone();
        let spec_file = package_fields.spec_file;
        let config_root_path = PathBuf::from(&self.config_root);
        let spec_file_canonical = config_root_path.join(spec_file);
        let spec_file = spec_file_canonical.to_str().unwrap().to_string();
        let src_dir = config_root_path.join("src").to_str().unwrap().to_string();

        let workdir = self
            .workdir
            .ok_or("~/.pkg-builder/packages/bookworm")
            .unwrap();
        let workdir = expand_path(&workdir);
        let build_artifacts_dir = get_build_artifacts_dir(&package_fields.package_name, &workdir);
        let tarball_path = get_tarball_path(&package_fields.package_name, &package_fields.version_number, &build_artifacts_dir);
        let build_files_dir = get_build_files_dir(&package_fields.package_name, &package_fields.version_number, &build_artifacts_dir);
        let derived_fields = DerivedFields{
            spec_file,
            workdir,
            build_artifacts_dir,
            tarball_path,
            build_files_dir,
            src_dir
        };
        let config = BookwormPackagerConfig {
            read_config: self.config.clone(),
            derived_fields
        };

        Ok(config)
    }
}

pub fn get_build_artifacts_dir(package_name: &str, work_dir: &str) -> String {
    let build_artifacts_dir = format!("{}/{}", work_dir, &package_name);
    build_artifacts_dir
}
pub fn get_tarball_path(package_name: &str, version_number: &str, build_artifacts_dir: &str) -> String {
    let tarball_path = format!(
        "{}/{}_{}.orig.tar.gz",
        &build_artifacts_dir, &package_name, &version_number
    );
    tarball_path
}
pub fn get_build_files_dir(package_name: &str, version_number: &str, build_artifacts_dir: &str) -> String {
    let build_files_dir = format!(
        "{}/{}-{}",
        build_artifacts_dir, &package_name, &version_number
    );
    build_files_dir
}

pub fn expand_path(dir: &str) -> String {
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
