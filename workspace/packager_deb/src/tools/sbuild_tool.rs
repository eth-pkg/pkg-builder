use std::{path::PathBuf, process::Command};

use debian::{execute::Execute, sbuild::SbuildBuilder};
use log::{error, info, warn};
use types::{distribution::Distribution, version::Version};

use crate::{
    configs::{pkg_config::PackageType, sbuild_version::SbuildVersion},
    misc::{
        build_pipeline::BuildContext,
        sbuild_pipelines::{SbuildGitPipeline, SbuildSourcePipeline, SbuildVirtualPipeline},
    },
    sbuild::SbuildError,
};

use super::tool_runner::{BuildTool, ToolRunner};

pub struct SbuildTool {
    version: SbuildVersion,
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
        version: SbuildVersion,
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
    fn check_tool_version(&self) -> Result<(), SbuildError> {
        let output = Command::new(self.name()).arg("--version").output()?;
        if !output.status.success() {
            return Err(SbuildError::GenericError(format!(
                "Failed to check {} version",
                self.name()
            )));
        }
        let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
        let actual_version = SbuildVersion::try_from(stdout_str)?;

        match self.version.cmp(&actual_version) {
            std::cmp::Ordering::Less => warn!(
                "Using newer {} version ({}) than expected ({})",
                self.name(),
                actual_version,
                self.version
            ),
            std::cmp::Ordering::Greater => error!(
                "Using older {} version ({}) than expected ({})",
                self.name(),
                actual_version,
                self.version
            ),
            std::cmp::Ordering::Equal => info!("{} versions match ({})", self.name(), self.version),
        }
        Ok(())
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
