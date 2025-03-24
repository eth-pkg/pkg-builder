use std::{env, fs, path::PathBuf};
use thiserror::Error;

use log::info;
use types::{config::ConfigError, defaults::WORKDIR_ROOT};

use crate::{
    build_pipeline::{BuildContext, BuildError},
    pkg_config::{PackageType, PkgConfig},
    pkg_config_verify::PkgVerifyConfig,
    sbuild::{Sbuild, SbuildError},
    sbuild_pipelines::{SbuildGitPipeline, SbuildSourcePipeline, SbuildVirtualPipeline},
    validation::ValidationError,
};

pub struct SbuildPackager {
    config: PkgConfig,
    config_root: PathBuf,
}
#[derive(Debug, Error)]
pub enum PackageError {
    #[error(transparent)]
    BuildError(#[from] BuildError),
    #[error(transparent)]
    SbuildError(#[from] SbuildError),
    #[error(transparent)]
    ValidationError(#[from] ValidationError),
    #[error(transparent)]
    ConfigError(#[from] ConfigError),
}

impl SbuildPackager {
    pub fn new(config: PkgConfig, config_root: PathBuf) -> Self {
        // Update the config workdir
        let mut updated_config = config.clone();
        let mut default_work_dir = PathBuf::from(WORKDIR_ROOT);
        default_work_dir.push(config.build_env.codename.as_ref());
        let mut workdir = config.build_env.workdir;
        if workdir.as_os_str().is_empty() {
            workdir = default_work_dir;
        }
        let workdir = expand_path(&workdir, None);
        updated_config.build_env.workdir = workdir;

        // Update the spec file path
        let config_root_path = PathBuf::from(&config_root);
        let spec_file = updated_config.package_fields.spec_file.clone();
        let spec_file_canonical = config_root_path.join(spec_file);
        updated_config.package_fields.spec_file = spec_file_canonical.to_str().unwrap().to_string();

        // Build the complete context based on the package type
        SbuildPackager {
            config: updated_config,
            config_root,
        }
    }

    pub fn run_package(&self) -> Result<(), PackageError> {
        let context = build_context(&self.config, &self.config_root.display().to_string());
        info!("Using build context: {:#?}", context);
        match &self.config.package_type {
            PackageType::Default(_) => {
                let sbuild_setup = SbuildSourcePipeline::new(context.clone());
                sbuild_setup.execute()?;
            }
            PackageType::Git(_) => {
                let sbuild_setup = SbuildGitPipeline::new(context.clone());
                sbuild_setup.execute()?;
            }
            PackageType::Virtual => {
                let sbuild_setup = SbuildVirtualPipeline::new(context.clone());
                sbuild_setup.execute()?;
            }
        };
        let sbuild = self.get_build_env().unwrap();
        sbuild.package()?;
        Ok(())
    }

    pub fn run_env_clean(&self) -> Result<(), PackageError> {
        let sbuild = self.get_build_env().unwrap();
        sbuild.clean()?;
        Ok(())
    }

    pub fn run_env_create(&self) -> Result<(), PackageError> {
        let sbuild = self.get_build_env().unwrap();
        sbuild.create()?;
        Ok(())
    }

    pub fn verify(
        &self,
        verify_config: PkgVerifyConfig,
        no_package: bool,
    ) -> Result<(), PackageError> {
        let sbuild = self.get_build_env().unwrap();
        if !no_package {
            self.run_package()?;
        }
        sbuild.verify(verify_config)?;
        Ok(())
    }

    fn get_build_env(&self) -> Result<Sbuild, PackageError> {
        let context = build_context(&self.config, &self.config_root.display().to_string());
        let backend_build_env = Sbuild::new(self.config.clone(), context.build_files_dir.clone());
        Ok(backend_build_env)
    }

    pub fn run_lintian(&self) -> Result<(), PackageError> {
        let sbuild = self.get_build_env().unwrap();
        sbuild.run_lintian()?;
        Ok(())
    }

    pub fn run_autopkgtests(&self) -> Result<(), PackageError> {
        let sbuild = self.get_build_env().unwrap();
        sbuild.run_autopkgtests()?;
        Ok(())
    }

    pub fn run_piuparts(&self) -> Result<(), PackageError> {
        let sbuild = self.get_build_env().unwrap();
        sbuild.run_piuparts()?;
        Ok(())
    }
}

pub fn build_context(config: &PkgConfig, config_root: &str) -> BuildContext {
    let package_fields = &config.package_fields;
    let config_root_path = PathBuf::from(config_root);
    let source_to_patch_from_path = config_root_path.join("src").to_str().unwrap().to_string();

    let mut workdir = config.build_env.workdir.clone();
    let mut default_work_dir = PathBuf::from(WORKDIR_ROOT);
    default_work_dir.push(&config.build_env.codename.as_ref());
    if workdir.as_os_str().is_empty() {
        workdir = default_work_dir;
    }
    let workdir = expand_path(&workdir, None);

    let build_artifacts_dir = get_build_artifacts_dir(
        &package_fields.package_name,
        &workdir.display().to_string(),
        &package_fields.version_number.as_str(),
        &package_fields.revision_number,
    );

    let debian_orig_tarball_path = get_tarball_path(
        &package_fields.package_name,
        &package_fields.version_number.as_str(),
        &build_artifacts_dir,
    );

    let build_files_dir = get_build_files_dir(
        &package_fields.package_name,
        &package_fields.version_number.as_str(),
        &build_artifacts_dir,
    );

    let mut context = BuildContext {
        build_artifacts_dir,
        build_files_dir,
        debcrafter_version: config.build_env.debcrafter_version.as_str().to_string(),
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
            context.tarball_url =
                get_tarball_url(&default_config.tarball_url.as_str(), config_root);
            if let Some(hash) = &default_config.tarball_hash {
                context.tarball_hash = hash.clone();
            }
        }
        PackageType::Git(git_config) => {
            context.git_tag = git_config.git_tag.clone();
            context.git_url = git_config.git_url.as_str().to_string();
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
        expand_path(
            &PathBuf::from(tarball_url),
            Some(&PathBuf::from(config_root)),
        )
        .display()
        .to_string()
    }
}

pub fn expand_path(dir: &PathBuf, dir_to_expand: Option<&PathBuf>) -> PathBuf {
    if dir.to_string_lossy().starts_with('~') {
        let dir_str = dir.to_string_lossy();
        PathBuf::from(shellexpand::tilde(&dir_str).to_string())
    } else if dir.is_absolute() {
        dir.clone()
    } else {
        let parent_dir = match dir_to_expand {
            None => env::current_dir().unwrap(),
            Some(path) => path.clone(),
        };

        let path = parent_dir.join(dir);
        fs::canonicalize(path.clone()).unwrap_or(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_path_expands_tilde_correctly() {
        let tilde = PathBuf::from("~");
        let result = expand_path(&tilde, None);
        assert_ne!(result, tilde);
        assert!(!result.display().to_string().contains('~'));
    }

    #[test]
    fn expand_path_handles_absolute_paths() {
        let absolute_path = PathBuf::from("/absolute/path");
        let result = expand_path(&absolute_path, None);
        assert_eq!(result, absolute_path);
    }

    #[test]
    fn expand_path_expands_relative_paths_with_parent() {
        let file = PathBuf::from("somefile");
        let mut tmp = PathBuf::from("/tmp");
        let result = expand_path(&file, Some(&tmp));
        tmp.push(file);
        assert_eq!(result, tmp);
    }

    #[test]
    fn expand_path_expands_relative_paths_without_parent() {
        let file = PathBuf::from("somefile");

        let result = expand_path(&file, None);
        assert!(result.display().to_string().starts_with('/'));
    }
}
