use std::{path::PathBuf, process::Command};

use debian::{execute::Execute, lintian::Lintian};
use log::{info, warn};
use types::{distribution::Distribution, version::Version};

use crate::sbuild::SbuildError;

use super::tool_runner::{BuildTool, ToolRunner};

pub struct LintianTool {
    version: Version,
    changes_file: PathBuf,
    codename: Distribution,
}

impl LintianTool {
    pub fn new(version: Version, changes_file: PathBuf, codename: Distribution) -> Self {
        LintianTool {
            version,
            changes_file,
            codename,
        }
    }
}

impl BuildTool for LintianTool {
    fn name(&self) -> &str {
        "lintian"
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
        let actual_version = Version::try_from(stdout_str)?;

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
        // Configure Lintian-specific options
        Ok(())
    }
    fn execute(&self) -> Result<(), SbuildError> {
        Lintian::new()
            .suppress_tag("bad-distribution-in-changes-file")
            .info()
            .extended_info()
            .changes_file(&self.changes_file)
            .tag_display_limit(0)
            .fail_on_warning()
            .fail_on_error()
            .suppress_tag("debug-file-with-no-debug-symbols")
            .with_codename(&self.codename)
            .execute()?;
        Ok(())
    }
}
