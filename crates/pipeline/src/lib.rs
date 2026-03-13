pub mod context;
pub mod env;
pub mod prepare;
pub mod source;
pub mod test;
pub mod verify;

use std::sync::Arc;

use config::PkgConfig;
use context::Workspace;
use thiserror::Error;
use tool::ToolError;

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error(transparent)]
    Config(#[from] config::ConfigError),
    #[error(transparent)]
    Tool(#[from] ToolError),
    #[error("{phase}: {message}")]
    Phase {
        phase: &'static str,
        message: String,
    },
    #[error("Hash verification failed for {file}: expected {expected}, got {actual}")]
    HashMismatch {
        file: String,
        expected: String,
        actual: String,
    },
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Main entry point for package building operations.
pub struct PackageBuilder {
    ws: Workspace,
}

impl PackageBuilder {
    pub fn new(config: PkgConfig) -> Result<Self, PipelineError> {
        let ws = Workspace::new(Arc::new(config))?;
        Ok(Self { ws })
    }

    pub fn create_env(&self) -> Result<(), PipelineError> {
        env::create_env(&self.ws)
    }

    pub fn clean_env(&self) -> Result<(), PipelineError> {
        env::clean_env(&self.ws)
    }

    pub fn run_lintian(&self) -> Result<(), PipelineError> {
        test::run_lintian(&self.ws)
    }

    pub fn run_piuparts(&self) -> Result<(), PipelineError> {
        test::run_piuparts(&self.ws)
    }

    pub fn run_autopkgtest(&self) -> Result<(), PipelineError> {
        test::run_autopkgtest(&self.ws)
    }

    pub fn verify(
        &self,
        verify_config: config::verify::VerifyConfig,
    ) -> Result<(), PipelineError> {
        verify::verify_package(&self.ws, verify_config)
    }
}
