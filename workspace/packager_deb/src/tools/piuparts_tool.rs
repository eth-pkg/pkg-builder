use std::path::PathBuf;

use debian::{execute::Execute, piuparts::Piuparts};
use types::{distribution::Distribution, version::Version};

use crate::{distribution::DistributionTrait, pkg_config::LanguageEnv, sbuild::SbuildError};

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
            deb_name
        }
    }
}

impl BuildTool for PiupartsTool {
    fn name(&self) -> &str {
        "lintian"
    }
    fn version(&self) -> &Version {
        &self.version
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
