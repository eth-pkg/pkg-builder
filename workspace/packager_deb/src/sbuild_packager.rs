use std::{env, fs, path::PathBuf};
use thiserror::Error;
use types::{
    build::{BackendBuildEnv, Packager},
    pkg_config::{PackageType, PkgConfig},
};

use log::info;

use crate::{
    build_pipeline::{BuildContext, BuildError},
    sbuild::{Sbuild, SbuildError},
    steps::sbuild_setup::{SbuildSetupDefault, SbuildSetupGit, SbuildSetupVirtual},
};

pub struct SbuildPackager {
    config: PkgConfig,
    context: BuildContext,
}
#[derive(Debug, Error)]
pub enum PackageError {
    #[error(transparent)]
    BuildError(#[from] BuildError),
    #[error(transparent)]
    SbuildError(#[from] SbuildError),
}

impl Packager for SbuildPackager {
    type BuildEnv = Sbuild;
    type Error = PackageError;

    fn new(config: PkgConfig, config_root: String) -> Self {
        // Update the config workdir
        let mut updated_config = config.clone();
        let workdir = config.build_env.workdir.clone().unwrap_or(format!(
            "~/.pkg-builder/packages/{}",
            config.build_env.codename
        ));
        let workdir = expand_path(&workdir, None);
        updated_config.build_env.workdir = Some(workdir);

        // Update the spec file path
        let config_root_path = PathBuf::from(&config_root);
        let spec_file = updated_config.package_fields.spec_file.clone();
        let spec_file_canonical = config_root_path.join(spec_file);
        updated_config.package_fields.spec_file = spec_file_canonical.to_str().unwrap().to_string();

        // Build the complete context based on the package type
        let context = build_context(&updated_config, &config_root);
        SbuildPackager {
            config: updated_config,
            context,
        }
    }

    fn package(&self) -> Result<(), PackageError> {
        match &self.config.package_type {
            PackageType::Default(_) => {
                info!("Using build context: {:#?}", self.context);
                let sbuild_setup = SbuildSetupDefault::new(self.context.clone());
                sbuild_setup.execute()?;
            }
            PackageType::Git(_) => {
                info!("Using build context: {:#?}", self.context);
                let sbuild_setup = SbuildSetupGit::new(self.context.clone());
                sbuild_setup.execute()?;
            }
            PackageType::Virtual => {
                info!("Using build context: {:#?}", self.context);
                let sbuild_setup = SbuildSetupVirtual::new(self.context.clone());
                sbuild_setup.execute()?;
            }
        };
        let sbuild = self.get_build_env().unwrap();
        sbuild.package()?;
        Ok(())
    }

    fn get_build_env(&self) -> Result<Self::BuildEnv, PackageError> {
        let backend_build_env =
            Sbuild::new(self.config.clone(), self.context.build_files_dir.clone());
        Ok(backend_build_env)
    }
}

pub fn build_context(config: &PkgConfig, config_root: &str) -> BuildContext {
    let package_fields = &config.package_fields;
    let config_root_path = PathBuf::from(config_root);
    let source_to_patch_from_path = config_root_path.join("src").to_str().unwrap().to_string();

    let workdir = config.build_env.workdir.clone().unwrap_or(format!(
        "~/.pkg-builder/packages/{}",
        config.build_env.codename
    ));
    let workdir = expand_path(&workdir, None);

    let build_artifacts_dir = get_build_artifacts_dir(
        &package_fields.package_name,
        &workdir,
        &package_fields.version_number,
        &package_fields.revision_number,
    );

    let debian_orig_tarball_path = get_tarball_path(
        &package_fields.package_name,
        &package_fields.version_number,
        &build_artifacts_dir,
    );

    let build_files_dir = get_build_files_dir(
        &package_fields.package_name,
        &package_fields.version_number,
        &build_artifacts_dir,
    );

    let mut context = BuildContext {
        build_artifacts_dir,
        build_files_dir,
        debcrafter_version: config.build_env.debcrafter_version.clone(),
        homepage: package_fields.homepage.clone(),
        spec_file: package_fields.spec_file.clone(),
        tarball_hash: String::new(),
        tarball_url: String::new(),
        src_dir: source_to_patch_from_path,
        tarball_path: debian_orig_tarball_path,
        package_name: package_fields.package_name.clone(),
        git_tag: String::new(),
        git_url: String::new(),
        submodules: vec![],
    };

    match &config.package_type {
        PackageType::Default(default_config) => {
            context.tarball_url = get_tarball_url(&default_config.tarball_url, config_root);
            if let Some(hash) = &default_config.tarball_hash {
                context.tarball_hash = hash.clone();
            }
        }
        PackageType::Git(git_config) => {
            context.git_tag = git_config.git_tag.clone();
            context.git_url = git_config.git_url.clone();
            context.submodules = git_config.submodules.clone();
        }
        PackageType::Virtual => {
            // Virtual packages already have the correct default values
        }
    }

    context
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
