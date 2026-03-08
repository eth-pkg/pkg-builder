pub mod archive;
pub mod autopkgtest;
pub mod command;
pub mod lintian;
pub mod piuparts;
pub mod sbuild;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("{tool} failed with exit code {code}")]
    ExitCode { tool: String, code: i32 },
    #[error("{tool} not found: {source}")]
    NotFound { tool: String, source: std::io::Error },
    #[error("{tool}: {message}")]
    Other { tool: String, message: String },
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
