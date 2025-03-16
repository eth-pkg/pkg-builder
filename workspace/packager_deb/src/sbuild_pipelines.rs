use crate::{
    build_pipeline::{BuildContext, BuildError, BuildPipeline},
    steps::{
        create_debian_dir::CreateDebianDir, create_empty_tar::CreateEmptyTar,
        dowload_git::DownloadGit, download_source::DownloadSource, extract_source::ExtractSource,
        package_dir_setup::PackageDirSetup, patch_source::PatchSource, setup_sbuild::SetupSbuild,
        verify_hash::VerifyHash,
    },
};

#[derive(Default)]
pub struct SbuildSourcePipeline {
    context: BuildContext,
}

impl SbuildSourcePipeline {
    pub fn new(context: BuildContext) -> Self {
        SbuildSourcePipeline { context }
    }

    pub fn execute(self) -> Result<(), BuildError> {
        let mut pipeline = BuildPipeline::new();
        let package_dir_handle = PackageDirSetup::from(self.context.clone());
        let download_source_handle = DownloadSource::from(self.context.clone());
        let verify_hash_handle = VerifyHash::from(self.context.clone());
        let extract_source_handle = ExtractSource::from(self.context.clone());
        let create_deb_dir: CreateDebianDir = CreateDebianDir::from(self.context.clone());
        let patch_source_handle = PatchSource::from(self.context.clone());
        let setup_sbuild_handle = SetupSbuild::from(self.context.clone());

        pipeline
            .add_step(package_dir_handle)
            .add_step(download_source_handle)
            .add_step(verify_hash_handle)
            .add_step(extract_source_handle)
            .add_step(create_deb_dir)
            .add_step(patch_source_handle)
            .add_step(setup_sbuild_handle);

        pipeline.execute()?;
        Ok(())
    }
}

#[derive(Default)]
pub struct SbuildGitPipeline {
    context: BuildContext,
}

impl SbuildGitPipeline {
    pub fn new(context: BuildContext) -> Self {
        SbuildGitPipeline { context }
    }

    pub fn execute(self) -> Result<(), BuildError> {
        let mut pipeline = BuildPipeline::new();
        let package_dir_handle = PackageDirSetup::from(self.context.clone());
        let download_git_step = DownloadGit::from(self.context.clone());
        let extract_source_handle = ExtractSource::from(self.context.clone());
        let create_deb_dir: CreateDebianDir = CreateDebianDir::from(self.context.clone());
        let patch_source_handle = PatchSource::from(self.context.clone());
        let setup_sbuild_handle = SetupSbuild::from(self.context.clone());

        pipeline
            .add_step(package_dir_handle)
            .add_step(download_git_step)
            .add_step(extract_source_handle)
            .add_step(create_deb_dir)
            .add_step(patch_source_handle)
            .add_step(setup_sbuild_handle);

        pipeline.execute()?;
        Ok(())
    }
}

#[derive(Default)]
pub struct SbuildVirtualPipeline {
    context: BuildContext,
}

impl SbuildVirtualPipeline {
    pub fn new(context: BuildContext) -> Self {
        SbuildVirtualPipeline { context }
    }

    pub fn execute(self) -> Result<(), BuildError> {
        let mut pipeline = BuildPipeline::new();
        let package_dir_handle = PackageDirSetup::from(self.context.clone());
        let empty_tar_handle = CreateEmptyTar::from(self.context.clone());
        let extract_source_handle = ExtractSource::from(self.context.clone());
        let create_deb_dir: CreateDebianDir = CreateDebianDir::from(self.context.clone());
        let patch_source_handle = PatchSource::from(self.context.clone());
        let setup_sbuild_handle = SetupSbuild::from(self.context.clone());

        pipeline
            .add_step(package_dir_handle)
            .add_step(empty_tar_handle)
            .add_step(extract_source_handle)
            .add_step(create_deb_dir)
            .add_step(patch_source_handle)
            .add_step(setup_sbuild_handle);

        pipeline.execute()?;
        Ok(())
    }
}
