use crate::{
    build_pipeline::{BuildContext, BuildError, BuildPipeline},
    dir_setup::get_tarball_url,
};

use super::{
    create_debian_dir_handler::CreateDebianDirHandle,
    download_source_handler::DownloadSourceHandler, extract_source_handler::ExtractSourceHandler,
    package_dir_setup_handler::PackageDirSetupHandler, patch_source_handler::PatchSourceHandle,
    setup_sbuild_handler::SetupSbuildHandle, verify_hash_handler::VerifyHashHandler,
};

#[derive(Default)]
pub struct SbuildSetupDefault {
    tarball_url: String,
    config_root: String,
    tarball_hash: String,
    debian_orig_tarball_path: String,
    build_files_dir: String,
    debcrafter_version: String,
    homepage: String,
    build_artifacts_dir: String,
    spec_file: String,
}

impl SbuildSetupDefault {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_tarball_url(mut self, tarball_url: String) -> Self {
        self.tarball_url = tarball_url;
        self
    }
    pub fn with_config_root(mut self, config_root: String) -> Self {
        self.tarball_url = config_root;
        self
    }
    pub fn with_tarball_hash(mut self, tarball_hash: String) -> Self {
        self.tarball_hash = tarball_hash;
        self
    }
    pub fn with_build_files_dir(mut self, build_files_dir: String) -> Self {
        self.build_files_dir = build_files_dir;
        self
    }
    pub fn with_debcrafter_version(mut self, debcrafter_version: String) -> Self {
        self.debcrafter_version = debcrafter_version;
        self
    }

    pub fn with_homepage(mut self, homepage: String) -> Self {
        self.homepage = homepage;
        self
    }
    pub fn with_build_artifacts_dir(mut self, build_artifacts_dir: String) -> Self {
        self.build_artifacts_dir = build_artifacts_dir;
        self
    }
    pub fn with_spec_file(mut self, spec_file: String) -> Self {
        self.spec_file = spec_file;
        self
    }

    pub fn execute(self) -> Result<(), BuildError> {
        let mut pipeline = BuildPipeline::new();

        let package_dir_handle = PackageDirSetupHandler::new();
        let tarball_path = get_tarball_url(&self.tarball_url, &self.config_root);

        let download_source_handle =
            DownloadSourceHandler::new(tarball_path.clone(), self.tarball_url.clone());
        let verify_hash_handle = VerifyHashHandler::new()
            .with_tarball_path(tarball_path.clone())
            .with_expected_checksum(Some(self.tarball_hash.clone()));
        let extract_source_handle = ExtractSourceHandler::new()
            .with_build_files_dir(self.debian_orig_tarball_path.clone())
            .with_tarball_path(tarball_path.clone());
        let create_deb_dir: CreateDebianDirHandle = CreateDebianDirHandle::new()
            .with_build_files_dir(self.build_files_dir.clone())
            .with_debcrafter_version(self.debcrafter_version.clone())
            .with_spec_file(self.spec_file.clone());
        let patch_source_handle = PatchSourceHandle::new()
            .with_build_files_dir(self.build_files_dir.clone())
            .with_homepage(self.homepage.clone())
            .with_src_dir(self.spec_file.clone());
        let setup_sbuild_handle = SetupSbuildHandle::new();
        pipeline
            .add_step(package_dir_handle)
            .add_step(download_source_handle)
            .add_step(verify_hash_handle)
            .add_step(extract_source_handle)
            .add_step(create_deb_dir)
            .add_step(patch_source_handle)
            .add_step(setup_sbuild_handle);

        let context = &mut BuildContext::new();
        context.build_artifacts_dir = self.build_artifacts_dir.clone();
        pipeline.execute(context)?;
        Ok(())
    }
}
