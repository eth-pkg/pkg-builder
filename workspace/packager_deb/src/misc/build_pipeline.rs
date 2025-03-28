use std::path::PathBuf;

use debian::debcrafter::DebcrafterCmdError;
use thiserror::Error;

use crate::{
    configs::pkg_config::SubModule,
    steps::{create_empty_tar::CreateEmptyTarError, dowload_git::DownloadGitError},
};

#[derive(Debug, Default, Clone)]
pub struct BuildContext {
    pub tarball_url: String,
    pub tarball_hash: String,
    pub tarball_path: PathBuf,
    pub build_files_dir: PathBuf,
    pub debcrafter_version: String,
    pub homepage: String,
    pub build_artifacts_dir: PathBuf,
    pub spec_file: PathBuf,
    pub src_dir: PathBuf,
    // only for git package
    pub package_name: String,
    pub git_tag: String,
    pub git_url: String,
    pub submodules: Vec<SubModule>,
}

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    #[error("Download failed")]
    DownloadFailed,

    #[error("File copy failed: {0}")]
    FileCopyFailed(String),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("Failed to open tarball: {0}")]
    TarballOpenError(String),

    #[error("Failed to read tarball: {0}")]
    TarballReadError(String),

    #[error("Checksum verification failed: hashes do not match")]
    HashMismatchError,

    #[error("Extraction error: {0}")]
    ExtractionError(String),

    #[error(transparent)]
    DebcrafterError(#[from] DebcrafterCmdError),

    #[error("Failed to copy src directory: {0}")]
    CopyDirectory(String),

    #[error("Failed to get debian/rules permission")]
    RulesPermissionGet,

    #[error("Failed to set debian/rules permission")]
    RulesPermissionSet,

    #[error("Home directory not found")]
    HomeDirNotFound,

    #[error("Failed to create ~/.sbuildrc: {0}")]
    FileCreationError(String),

    #[error("Failed to write to ~/.sbuildrc: {0}")]
    FileWriteError(String),

    #[error(transparent)]
    DownloadGitStepError(#[from] DownloadGitError),

    #[error(transparent)]
    CreateEmptyTarStepError(#[from] CreateEmptyTarError),
}

pub trait BuildStep {
    fn step(&self) -> Result<(), BuildError>;
}

#[derive(Default)]
pub struct BuildPipeline {
    handlers: Vec<Box<dyn BuildStep>>,
}

impl BuildPipeline {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_step<T: BuildStep + 'static>(&mut self, handler: T) -> &mut Self {
        self.handlers.push(Box::new(handler));
        self
    }

    pub fn execute(&self) -> Result<(), BuildError> {
        for handler in &self.handlers {
            handler.step()?;
        }
        Ok(())
    }
}
