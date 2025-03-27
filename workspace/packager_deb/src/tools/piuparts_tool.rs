use std::{path::PathBuf, process::Command};

use debian::{execute::Execute, piuparts::Piuparts};
use log::{info, warn};
use types::{distribution::Distribution, version::Version};

use crate::{
    configs::pkg_config::LanguageEnv, misc::distribution::DistributionTrait, sbuild::SbuildError,
};

use super::tool_runner::{BuildTool, ToolRunner};

pub struct PiupartsTool {
    version: Version,
    codename: Distribution,
    deb_dir: PathBuf,
    language_env: Option<LanguageEnv>,
    deb_name: PathBuf,
}

impl PiupartsTool {
    pub fn new(
        version: Version,
        codename: Distribution,
        deb_dir: PathBuf,
        deb_name: PathBuf,
        language_env: Option<LanguageEnv>,
    ) -> Self {
        PiupartsTool {
            version,
            codename,
            deb_dir,
            language_env,
            deb_name,
        }
    }
}

impl BuildTool for PiupartsTool {
    fn name(&self) -> &str {
        "piuparts"
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
        let actual_version = Version::try_from(stdout_str.replace("piuparts ", "").trim())?;

        match self.version.cmp(&actual_version) {
            std::cmp::Ordering::Less => warn!(
                "Using newer {} version ({}) than expected ({})",
                self.name(),
                actual_version,
                self.version
            ),
            std::cmp::Ordering::Greater => warn!(
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
        // Configure piuparts
        Ok(())
    }
    fn execute(&self) -> Result<(), SbuildError> {
        Piuparts::new()
            .distribution(&self.codename)
            .mirror(&self.codename.get_repo_url())
            .bindmount_dev()
            .keyring(&self.codename.get_keyring())
            .verbose()
            .with_dotnet_env(
                matches!(self.language_env, Some(LanguageEnv::Dotnet(_))),
                &self.codename,
            )
            .deb_file(&self.deb_name)
            .deb_path(&self.deb_dir)
            .execute()?;
        Ok(())
    }
}
