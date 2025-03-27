use log::info;
// use types::version::Version;

use crate::sbuild::SbuildError;

pub trait BuildTool {
    fn name(&self) -> &str;
    fn check_tool_version(&self) -> Result<(), SbuildError>;
    fn configure(&mut self, runner: &mut ToolRunner) -> Result<(), SbuildError>;
    fn execute(&self) -> Result<(), SbuildError>;
}

pub struct ToolRunner {}

impl ToolRunner {
    pub fn new() -> Self {
        ToolRunner {}
    }

    pub fn run_tool<T: BuildTool>(&mut self, mut tool: T) -> Result<(), SbuildError> {
        info!("Running {}...", tool.name());
        tool.check_tool_version()?;
        tool.configure(self)?;
        tool.execute()?;
        Ok(())
    }
}
