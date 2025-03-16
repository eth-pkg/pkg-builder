use crate::build_pipeline::{BuildContext, BuildError, BuildPipeline};

use super::{
    create_debian_dir::CreateDebianDir, download_source::DownloadSource,
    extract_source::ExtractSource, package_dir_setup::PackageDirSetup, patch_source::PatchSource,
    setup_sbuild::SetupSbuild, verify_hash::VerifyHash,
};

#[derive(Default)]
pub struct SbuildSetupDefault {
    context: BuildContext,
}

impl SbuildSetupDefault {
    pub fn new(context: BuildContext) -> Self {
        SbuildSetupDefault { context }
    }

    pub fn execute(self) -> Result<(), BuildError> {
        let mut pipeline = BuildPipeline::new();

        let package_dir_handle = PackageDirSetup::new();
        let download_source_handle = DownloadSource::new();
        let verify_hash_handle = VerifyHash::new();
        let extract_source_handle = ExtractSource::new();
        let create_deb_dir: CreateDebianDir = CreateDebianDir::new();
        let patch_source_handle = PatchSource::new();
        let setup_sbuild_handle = SetupSbuild::new();
        pipeline
            .add_step(package_dir_handle)
            .add_step(download_source_handle)
            .add_step(verify_hash_handle)
            .add_step(extract_source_handle)
            .add_step(create_deb_dir)
            .add_step(patch_source_handle)
            .add_step(setup_sbuild_handle);

        pipeline.execute(&mut self.context.clone())?;
        Ok(())
    }
}

#[derive(Default)]
pub struct SbuildSetupGit {
    context: BuildContext,
}

impl SbuildSetupGit {
    pub fn new(context: BuildContext) -> Self {
        SbuildSetupGit { context }
    }

    pub fn execute(self) -> Result<(), BuildError> {
        Ok(())
    }
}

// #[derive(Default)]
// pub struct SbuildSetupVirtual {
//     context: BuildContext,
// }

// impl SbuildSetupVirtual {
//     pub fn new(context: BuildContext) -> Self {
//         SbuildSetupVirtual { context }
//     }

//     pub fn execute(self) -> Result<(), BuildError> {
     
//         Ok(())
//     }
// }
