use common::{
    build::{BackendBuildEnv, Packager},
    pkg_config::{PackageType, PkgConfig},
};
use eyre::Result;

use log::info;
use std::path::PathBuf;

use crate::{
    build_pipeline::{BuildContext, BuildPipeline},
    dir_setup::{
        create_debian_dir, create_empty_tar, download_git, expand_path, extract_source,
        get_build_artifacts_dir, get_build_files_dir, get_tarball_path, get_tarball_url,
        patch_source, setup_sbuild,
    },
    handlers::{
        download_source_handler::DownloadSourceHandler,
        package_dir_setup_handler::PackageDirSetupHandler, verify_hash_handler::VerifyHashHandler,
    },
    sbuild::Sbuild,
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
        let mut pipeline = BuildPipeline::new();
        let pre_build: Result<()> = match &self.config.package_type {
            PackageType::Default(config) => {
                let package_dir_handle = PackageDirSetupHandler::new();
                let tarball_path = get_tarball_url(&config.tarball_url, &self.config_root);

                let download_source_handle =
                    DownloadSourceHandler::new(tarball_path.clone(), config.tarball_url.clone());
                let verify_hash_handle = VerifyHashHandler::new()
                    .with_tarball_path(tarball_path)
                    .with_expected_checksum(config.tarball_hash.clone());
                // let extract_source_handle = ExtractSourceHandler::new();
                // let create_debian_dir = CreateDebianDirHandle::new();
                // let patch_source_handle = VerifyHashHandler::new();
                // let setup_sbuild_handle = VerifyHashHandler::new();
                pipeline
                    .add_step(package_dir_handle)
                    .add_step(download_source_handle)
                    .add_step(verify_hash_handle);
                // .add_handler(extract_source_handle)
                // .add_handler(create_debian_dir)
                // .add_handler(patch_source_handle)
                // .add_handler(setup_sbuild_handle);

                let context = &mut BuildContext::new();
                context.build_artifacts_dir = self.debian_artifacts_dir.clone();
                pipeline.execute(context)?;

                extract_source(&self.debian_orig_tarball_path, &self.build_files_dir)?;
                create_debian_dir(
                    &self.build_files_dir.clone(),
                    &self.config.build_env.debcrafter_version,
                    &self.config.package_fields.spec_file,
                )?;
                patch_source(
                    &self.build_files_dir.clone(),
                    &self.config.package_fields.homepage,
                    &self.source_to_patch_from_path,
                )?;
                setup_sbuild()?;
                Ok(())
            }
            PackageType::Git(config) => {
                let package_dir_handle = PackageDirSetupHandler::new();
                pipeline.add_step(package_dir_handle);

                let context = &mut BuildContext::new();
                context.build_artifacts_dir = self.debian_artifacts_dir.clone();
                pipeline.execute(context)?;

                download_git(
                    &self.debian_artifacts_dir,
                    &self.debian_orig_tarball_path,
                    &self.config.package_fields.package_name,
                    &config.git_url,
                    &config.git_tag,
                    &config.submodules,
                )?;
                extract_source(&self.debian_orig_tarball_path, &self.build_files_dir)?;
                create_debian_dir(
                    &self.build_files_dir.clone(),
                    &self.config.build_env.debcrafter_version,
                    &self.config.package_fields.spec_file,
                )?;
                patch_source(
                    &self.build_files_dir.clone(),
                    &self.config.package_fields.homepage,
                    &self.source_to_patch_from_path,
                )?;
                setup_sbuild()?;
                Ok(())
            }
            PackageType::Virtual => {
                info!("creating virtual package");
                let package_dir_handle = PackageDirSetupHandler::new();
                pipeline.add_step(package_dir_handle);

                let context = &mut BuildContext::new();
                context.build_artifacts_dir = self.debian_artifacts_dir.clone();
                pipeline.execute(context)?;

                create_empty_tar(&self.debian_artifacts_dir, &self.debian_orig_tarball_path)?;
                extract_source(&self.debian_orig_tarball_path, &self.build_files_dir)?;
                create_debian_dir(
                    &self.build_files_dir.clone(),
                    &self.config.build_env.debcrafter_version,
                    &self.config.package_fields.spec_file,
                )?;
                patch_source(
                    &self.build_files_dir.clone(),
                    &self.config.package_fields.homepage,
                    &self.source_to_patch_from_path,
                )?;
                setup_sbuild()?;
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
