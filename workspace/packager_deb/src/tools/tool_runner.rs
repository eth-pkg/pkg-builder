use log::info;
use types::version::Version;

use crate::{sbuild::SbuildError, utils::check_tool_version};

pub trait BuildTool {
    fn name(&self) -> &str;
    fn version(&self) -> &Version;
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
        check_tool_version(tool.name(), &tool.version().clone())?;
        tool.configure(self)?;
        tool.execute()?;
        Ok(())
    }
}
