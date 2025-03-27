use std::{path::PathBuf, process::Command};

use debian::{execute::Execute, sbuild::SbuildBuilder};
use log::{error, info, warn};
use types::distribution::Distribution;

use crate::{
    configs::{pkg_config::PackageType, sbuild_version::SbuildVersion},
    misc::{
        build_pipeline::BuildContext,
        sbuild_pipelines::{SbuildGitPipeline, SbuildSourcePipeline, SbuildVirtualPipeline},
    },
    sbuild::SbuildError,
};

use super::tool_runner::{BuildTool, ToolRunner};

pub struct SbuildToolArgs {
    pub(crate) version: SbuildVersion,
    pub(crate) codename: Distribution,
    pub(crate) cache_file: PathBuf,
    pub(crate) build_chroot_setup_commands: Vec<String>,
    pub(crate) run_lintian: bool,
    pub(crate) build_files_dir: PathBuf,
    pub(crate) package_type: PackageType,
    pub(crate) context: BuildContext,
}
pub struct SbuildTool {
    args: SbuildToolArgs,
}

impl SbuildTool {
    pub fn new(args: SbuildToolArgs) -> Self {
        SbuildTool { args }
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

        match self.args.version.cmp(&actual_version) {
            std::cmp::Ordering::Less => warn!(
                "Using newer {} version ({}) than expected ({})",
                self.name(),
                actual_version,
                self.args.version
            ),
            std::cmp::Ordering::Greater => error!(
                "Using older {} version ({}) than expected ({})",
                self.name(),
                actual_version,
                self.args.version
            ),
            std::cmp::Ordering::Equal => info!("{} versions match ({})", self.name(), self.args.version),
        }
        Ok(())
    }
    fn configure(&mut self, _runner: &mut ToolRunner) -> Result<(), SbuildError> {
        info!("Using build context: {:#?}", self.args.context);
        match &self.args.package_type {
            PackageType::Default(_) => {
                let sbuild_setup = SbuildSourcePipeline::new(self.args.context.clone());
                sbuild_setup.execute()?;
            }
            PackageType::Git(_) => {
                let sbuild_setup = SbuildGitPipeline::new(self.args.context.clone());
                sbuild_setup.execute()?;
            }
            PackageType::Virtual => {
                let sbuild_setup = SbuildVirtualPipeline::new(self.args.context.clone());
                sbuild_setup.execute()?;
            }
        };
        Ok(())
    }
    fn execute(&self) -> Result<(), SbuildError> {
        let builder = SbuildBuilder::new()
            .distribution(&self.args.codename)
            .build_arch_all()
            .build_source()
            .cache_file(self.args.cache_file.clone())
            .verbose()
            .chroot_mode_unshare()
            .setup_commands(&self.args.build_chroot_setup_commands)
            .no_run_piuparts()
            .no_apt_upgrades()
            .run_lintian(self.args.run_lintian)
            .no_run_autopkgtest()
            .working_dir(&self.args.build_files_dir);

        builder.execute()?;

        Ok(())
    }
}
