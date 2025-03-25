use std::path::PathBuf;

use debian::{execute::Execute, lintian::Lintian};
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
        LintianTool{
            version,
            changes_file,
            codename
        }
    }
}

impl BuildTool for LintianTool {
    fn name(&self) -> &str {
        "lintian"
    }
    fn version(&self) -> &Version {
        &self.version
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
