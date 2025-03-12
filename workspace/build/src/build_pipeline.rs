use thiserror::Error;

use crate::debcrafter_cmd::DebcrafterCmdError;

#[derive(Debug, Default)]
pub struct BuildContext {
    pub build_artifacts_dir: String, // package_directory we download and extract

}

impl BuildContext {
    pub fn new() -> Self {
        Self::default()
    }
}


#[derive(Error, Debug)]
pub enum BuildError {
    #[error("Missing tarball path")]
    MissingPath,
    
    #[error("Missing tarball URL")]
    MissingUrl,
    
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



pub trait BuildHandler {
    fn handle(&self, context: &mut BuildContext) -> Result<(), BuildError>;
}

#[derive(Default)]
pub struct BuildPipeline {
    handlers: Vec<Box<dyn BuildHandler>>,
}

impl BuildPipeline {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn add_step<T: BuildHandler + 'static>(&mut self, handler: T) -> &mut Self {
        self.handlers.push(Box::new(handler));
        self
    }
    
    pub fn execute(&self, context: &mut BuildContext) -> Result<(), BuildError> {
        for handler in &self.handlers {
            handler.handle(context)?;
        }
        Ok(())
    }
}

