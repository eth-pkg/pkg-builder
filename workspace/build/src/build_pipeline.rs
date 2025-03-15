use thiserror::Error;

use crate::debcrafter_cmd::DebcrafterCmdError;

#[derive(Debug, Default)]
pub struct BuildContext {
    pub tarball_url: String,
    pub config_root: String,
    pub tarball_hash: String,
    pub debian_orig_tarball_path: String,
    pub build_files_dir: String,
    pub debcrafter_version: String,
    pub homepage: String,
    pub build_artifacts_dir: String,
    pub debian_artifacts_dir: String,
    pub spec_file: String,
    pub tarball_path: String,
    pub src_dir: String,

}

impl BuildContext {
    pub fn new() -> Self {
        Self::default()
    }
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
}



pub trait BuildStep {
    fn step(&self, context: &mut BuildContext) -> Result<(), BuildError>;
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
    
    pub fn execute(&self, context: &mut BuildContext) -> Result<(), BuildError> {
        for handler in &self.handlers {
            handler.step(context)?;
        }
        Ok(())
    }
}

