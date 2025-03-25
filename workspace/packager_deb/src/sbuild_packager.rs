use std::path::PathBuf;
use thiserror::Error;

use types::{config::ConfigError, defaults::WORKDIR_ROOT};

use crate::{
    build_pipeline::BuildError,
    pkg_config::PkgConfig,
    pkg_config_verify::PkgVerifyConfig,
    sbuild::{Sbuild, SbuildError},
    sbuild_args::expand_path,
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
        let backend_build_env = Sbuild::new(self.config.clone(), self.config_root.clone());
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
