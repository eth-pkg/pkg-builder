use std::path::PathBuf;

use debian::{execute::Execute, sbuild::SbuildBuilder};
use log::info;
use types::{distribution::Distribution, version::Version};

use crate::{
    configs::pkg_config::PackageType,
    misc::build_pipeline::BuildContext,
    misc::sbuild_pipelines::{SbuildGitPipeline, SbuildSourcePipeline, SbuildVirtualPipeline},
    sbuild::SbuildError,
};

use super::tool_runner::{BuildTool, ToolRunner};

pub struct SbuildTool {
    version: Version,
    codename: Distribution,
    cache_file: PathBuf,
    build_chroot_setup_commands: Vec<String>,
    run_lintian: bool,
    build_files_dir: PathBuf,
    package_type: PackageType,
    context: BuildContext,
}

impl SbuildTool {
    pub fn new(
        version: Version,
        codename: Distribution,
        cache_file: PathBuf,
        build_chroot_setup_commands: Vec<String>,
        run_lintian: bool,
        build_files_dir: PathBuf,
        package_type: PackageType,
        context: BuildContext,
    ) -> Self {
        SbuildTool {
            version,
            codename,
            cache_file,
            build_chroot_setup_commands,
            run_lintian,
            build_files_dir,
            package_type,
            context,
        }
    }
}

impl BuildTool for SbuildTool {
    fn name(&self) -> &str {
        "sbuild"
    }
    fn version(&self) -> &Version {
        &self.version
    }
    fn configure(&mut self, _runner: &mut ToolRunner) -> Result<(), SbuildError> {
        info!("Using build context: {:#?}", self.context);
        match &self.package_type {
            PackageType::Default(_) => {
                let sbuild_setup = SbuildSourcePipeline::new(self.context.clone());
                sbuild_setup.execute()?;
            }
            PackageType::Git(_) => {
                let sbuild_setup = SbuildGitPipeline::new(self.context.clone());
                sbuild_setup.execute()?;
            }
            PackageType::Virtual => {
                let sbuild_setup = SbuildVirtualPipeline::new(self.context.clone());
                sbuild_setup.execute()?;
            }
        };
        Ok(())
    }
    fn execute(&self) -> Result<(), SbuildError> {
        let builder = SbuildBuilder::new()
            .distribution(&self.codename)
            .build_arch_all()
            .build_source()
            .cache_file(self.cache_file.clone())
            .verbose()
            .chroot_mode_unshare()
            .setup_commands(&self.build_chroot_setup_commands)
            .no_run_piuparts()
            .no_apt_upgrades()
            .run_lintian(self.run_lintian)
            .no_run_autopkgtest()
            .working_dir(&self.build_files_dir);

        builder.execute()?;

        Ok(())
    }
}
