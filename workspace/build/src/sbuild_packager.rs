use eyre::Result;
use types::{
    build::{BackendBuildEnv, Packager},
    pkg_config::{PackageType, PkgConfig},
};
use std::{env, fs, path::PathBuf};


use log::info;

use crate::{
    build_pipeline::BuildContext,
    sbuild::Sbuild,
    steps::sbuild_setup::{SbuildSetupDefault, SbuildSetupGit, SbuildSetupVirtual},
};

pub struct SbuildPackager {
    config: PkgConfig,
    source_to_patch_from_path: String,
    debian_artifacts_dir: String,
    debian_orig_tarball_path: String,
    build_files_dir: String,
    config_root: String,
}

impl Packager for SbuildPackager {
    type BuildEnv = Sbuild;

    fn new(config: PkgConfig, config_root: String) -> Self {
        let package_fields = config.package_fields.clone();
        let config_root_path = PathBuf::from(&config_root);
        let source_to_patch_from_path = config_root_path.join("src").to_str().unwrap().to_string();
        let workdir = config.build_env.workdir.clone().unwrap_or(format!(
            "~/.pkg-builder/packages/{}",
            config.build_env.codename
        ));
        let workdir = expand_path(&workdir, None);
        let debian_artifacts_dir = get_build_artifacts_dir(
            &package_fields.package_name,
            &workdir,
            &package_fields.version_number,
            &package_fields.revision_number,
        );
        let debian_orig_tarball_path = get_tarball_path(
            &package_fields.package_name,
            &package_fields.version_number,
            &debian_artifacts_dir,
        );
        let build_files_dir = get_build_files_dir(
            &package_fields.package_name,
            &package_fields.version_number,
            &debian_artifacts_dir,
        );
        let mut updated_config = SbuildPackager {
            config,
            source_to_patch_from_path,
            build_files_dir,
            debian_artifacts_dir,
            debian_orig_tarball_path,
            config_root,
        };
        updated_config.config.build_env.workdir = Some(workdir);
        let spec_file = package_fields.spec_file;
        let spec_file_canonical = config_root_path.join(spec_file);
        updated_config.config.package_fields.spec_file =
            spec_file_canonical.to_str().unwrap().to_string();
        updated_config
    }

    fn package(&self) -> Result<()> {
        let pre_build: Result<()> = match &self.config.package_type {
            PackageType::Default(config) => {
                let tarball_url = get_tarball_url(&config.tarball_url, &self.config_root);
                let context = BuildContext {
                    build_artifacts_dir: self.debian_artifacts_dir.clone(),
                    build_files_dir: self.build_files_dir.clone(),
                    debcrafter_version: self.config.build_env.debcrafter_version.clone(),
                    homepage: self.config.package_fields.homepage.clone(),
                    spec_file: self.config.package_fields.spec_file.clone(),
                    tarball_hash: config.tarball_hash.clone().unwrap().clone(),
                    tarball_url: tarball_url.clone(),
                    src_dir: self.source_to_patch_from_path.clone(),
                    tarball_path: self.debian_orig_tarball_path.clone(),
                    package_name: self.config.package_fields.package_name.clone(),
                    git_tag: "".into(),
                    git_url: "".into(),
                    submodules: vec![],
                };
                info!("Using build context: {:#?}", context);
                let sbuild_setup = SbuildSetupDefault::new(context);
                sbuild_setup.execute()?;
                Ok(())
            }
            PackageType::Git(config) => {
                let context = BuildContext {
                    build_artifacts_dir: self.debian_artifacts_dir.clone(),
                    build_files_dir: self.build_files_dir.clone(),
                    debcrafter_version: self.config.build_env.debcrafter_version.clone(),
                    homepage: self.config.package_fields.homepage.clone(),
                    spec_file: self.config.package_fields.spec_file.clone(),
                    tarball_hash: "".into(),
                    tarball_url: "".into(),
                    src_dir: self.source_to_patch_from_path.clone(),
                    tarball_path: self.debian_orig_tarball_path.clone(),
                    package_name: self.config.package_fields.package_name.clone(),
                    git_tag: config.git_tag.clone(),
                    git_url: config.git_url.clone(),
                    submodules: config.submodules.clone(),
                };
                info!("Using build context: {:#?}", context);
                let sbuild_setup = SbuildSetupGit::new(context);
                sbuild_setup.execute()?;
                Ok(())
            }
            PackageType::Virtual => {
                let context = BuildContext {
                    build_artifacts_dir: self.debian_artifacts_dir.clone(),
                    build_files_dir: self.build_files_dir.clone(),
                    debcrafter_version: self.config.build_env.debcrafter_version.clone(),
                    homepage: self.config.package_fields.homepage.clone(),
                    spec_file: self.config.package_fields.spec_file.clone(),
                    tarball_hash: "".into(),
                    tarball_url: "".into(),
                    git_tag: "".into(),
                    git_url: "".into(),
                    submodules: vec![],
                    src_dir: self.source_to_patch_from_path.clone(),
                    tarball_path: self.debian_orig_tarball_path.clone(),
                    package_name: self.config.package_fields.package_name.clone(),
                };
                info!("Using build context: {:#?}", context);
                let sbuild_setup = SbuildSetupVirtual::new(context);
                sbuild_setup.execute()?;
                Ok(())
            }
        };
        pre_build?;
        let build_env = self.get_build_env().unwrap();
        build_env.package()?;
        Ok(())
    }

    fn get_build_env(&self) -> Result<Self::BuildEnv> {
        let backend_build_env = Sbuild::new(self.config.clone(), self.build_files_dir.clone());
        Ok(backend_build_env)
    }
}

pub fn get_build_artifacts_dir(
    package_name: &str,
    work_dir: &str,
    version_number: &str,
    revision_number: &str,
) -> String {
    let build_artifacts_dir = format!(
        "{}/{}-{}-{}",
        work_dir, &package_name, version_number, revision_number
    );
    build_artifacts_dir
}

pub fn get_tarball_path(
    package_name: &str,
    version_number: &str,
    build_artifacts_dir: &str,
) -> String {
    let tarball_path = format!(
        "{}/{}_{}.orig.tar.gz",
        &build_artifacts_dir, &package_name, &version_number
    );
    tarball_path
}

pub fn get_build_files_dir(
    package_name: &str,
    version_number: &str,
    build_artifacts_dir: &str,
) -> String {
    let build_files_dir = format!(
        "{}/{}-{}",
        build_artifacts_dir, &package_name, &version_number
    );
    build_files_dir
}

pub fn get_tarball_url(tarball_url: &str, config_root: &str) -> String {
    if tarball_url.starts_with("http") {
        tarball_url.to_string()
    } else {
        expand_path(tarball_url, Some(config_root))
    }
}

pub fn expand_path(dir: &str, dir_to_expand: Option<&str>) -> String {
    if dir.starts_with('~') {
        let expanded_path = shellexpand::tilde(dir).to_string();
        expanded_path
    } else if dir.starts_with('/') {
        dir.to_string()
    } else {
        let parent_dir = match dir_to_expand {
            None => env::current_dir().unwrap(),
            Some(path) => PathBuf::from(path),
        };
        let dir = parent_dir.join(dir);
        let path = fs::canonicalize(dir.clone()).unwrap_or(dir);
        let path = path.to_str().unwrap().to_string();
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_path_expands_tilde_correctly() {
        let result = expand_path("~", None);
        assert_ne!(result, "~");
        assert!(!result.contains('~'));
    }

    #[test]
    fn expand_path_handles_absolute_paths() {
        let result = expand_path("/absolute/path", None);
        assert_eq!(result, "/absolute/path");
    }

    #[test]
    fn expand_path_expands_relative_paths_with_parent() {
        let result = expand_path("somefile", Some("/tmp"));
        assert_eq!(result, "/tmp/somefile");
    }

    #[test]
    fn expand_path_expands_relative_paths_without_parent() {
        let result = expand_path("somefile", None);
        assert!(result.starts_with('/'));
    }
}
