use log::info;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DebcrafterCmdError {
    #[error("Command not found: {0}")]
    CommandNotFound(#[from] io::Error),

    #[error("Failed to execute command: {0}")]
    CommandFailed(CommandError),

    #[error("File not found: {0}")]
    FileNotFound(String),
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("{0}")]
    StringError(String),
    #[error("{0}")]
    IOError(#[from] io::Error),
}

impl PartialEq for CommandError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::StringError(a), Self::StringError(b)) => a == b,
            (Self::IOError(_), Self::IOError(_)) => false, // IO errors aren't comparable
            _ => false,
        }
    }
}

impl From<String> for CommandError {
    fn from(err: String) -> Self {
        CommandError::StringError(err)
    }
}

pub struct DebcrafterCmd {
    version: String,
}

impl DebcrafterCmd {
    /// Creates a new DebcrafterCmd with the specified version
    pub fn new(version: &str) -> Self {
        Self {
            version: version.to_string(),
        }
    }

    /// Checks if dpkg-parsechangelog is installed on the system
    pub fn check_if_dpkg_parsechangelog_installed(&self) -> Result<(), DebcrafterCmdError> {
        let mut cmd = Command::new("which");
        cmd.arg("dpkg-parsechangelog");

        self.handle_command_execution(
            &mut cmd,
            "dpkg-parsechangelog is not installed, please install it.".to_string(),
        )
    }

    /// Checks if the specified version of debcrafter is installed
    pub fn check_if_installed(&self) -> Result<(), DebcrafterCmdError> {
        let mut cmd = Command::new("which");
        cmd.arg(format!("debcrafter_{}", self.version));

        self.handle_command_execution(
            &mut cmd,
            format!("debcrafter_{} is not installed", self.version),
        )
    }

    /// Creates a debian directory using the specified specification file
    pub fn create_debian_dir(
        &self,
        specification_file: &str,
        target_dir: &str,
    ) -> Result<(), DebcrafterCmdError> {
        let debcrafter_dir =
            tempdir().map_err(|e| DebcrafterCmdError::CommandFailed(e.to_string().into()))?;

        let spec_file_path = fs::canonicalize(PathBuf::from(specification_file)).map_err(|_| {
            DebcrafterCmdError::FileNotFound(format!(
                "{} spec_file doesn't exist",
                specification_file
            ))
        })?;

        if !spec_file_path.exists() {
            return Err(DebcrafterCmdError::FileNotFound(format!(
                "{} spec_file doesn't exist",
                specification_file
            )));
        }

        let spec_dir = spec_file_path.parent().ok_or_else(|| {
            DebcrafterCmdError::CommandFailed("Invalid specification file path".to_string().into())
        })?;

        let spec_file_name = spec_file_path.file_name().ok_or_else(|| {
            DebcrafterCmdError::CommandFailed("Invalid specification file name".to_string().into())
        })?;

        info!(
            "Spec directory: {:?}",
            spec_dir.to_str().unwrap_or_default()
        );
        info!("Spec file: {:?}", spec_file_name);
        info!("Debcrafter directory: {:?}", debcrafter_dir.path());

        let mut cmd = Command::new(format!("debcrafter_{}", self.version));
        cmd.arg(spec_file_name)
            .current_dir(spec_dir)
            .arg(debcrafter_dir.path());

        self.handle_command_execution(&mut cmd, "Debcrafter execution failed".to_string())?;

        if let Some(first_directory) = self.get_first_directory(debcrafter_dir.path()) {
            let tmp_debian_dir = first_directory.join("debian");
            let dest_dir = Path::new(target_dir).join("debian");
            self.copy_dir_contents_recursive(&tmp_debian_dir, &dest_dir)
                .map_err(|err| DebcrafterCmdError::CommandFailed(err.to_string().into()))?;
        } else {
            return Err(DebcrafterCmdError::CommandFailed(
                "Unable to create debian dir: no output directory found"
                    .to_string()
                    .into(),
            ));
        }

        Ok(())
    }

    /// Recursively copies the contents of a directory to another location
    fn copy_dir_contents_recursive(&self, src_dir: &Path, dest_dir: &Path) -> io::Result<()> {
        info!(
            "Copying directory: {:?} to {:?}",
            src_dir.display(),
            dest_dir.display()
        );

        if !src_dir.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Source path is not a directory: {}", src_dir.display()),
            ));
        }

        if !dest_dir.exists() {
            fs::create_dir_all(dest_dir)?;
        }

        for entry in fs::read_dir(src_dir)? {
            let entry = entry?;
            let src_path = entry.path();
            let dest_path = dest_dir.join(entry.file_name());

            if src_path.is_dir() {
                self.copy_dir_contents_recursive(&src_path, &dest_path)?;
            } else {
                fs::copy(&src_path, &dest_path)?;
            }
        }

        Ok(())
    }

    /// Handles command execution and processes errors
    fn handle_command_execution(
        &self,
        cmd: &mut Command,
        error_message: String,
    ) -> Result<(), DebcrafterCmdError> {
        let output = cmd
            .output()
            .map_err(|e| DebcrafterCmdError::CommandNotFound(e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let detailed_error = if stderr.is_empty() {
                error_message
            } else {
                format!("{}: {}", error_message, stderr)
            };

            return Err(DebcrafterCmdError::CommandFailed(detailed_error.into()));
        }

        Ok(())
    }

    /// Gets the first directory in a given path
    fn get_first_directory(&self, dir: &Path) -> Option<PathBuf> {
        if !dir.is_dir() {
            return None;
        }

        fs::read_dir(dir)
            .ok()?
            .filter_map(Result::ok)
            .find(|entry| entry.path().is_dir())
            .map(|entry| entry.path())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_command_error_from_string() {
        let error = CommandError::from("test error".to_string());
        assert_eq!(error, CommandError::StringError("test error".to_string()));
    }

    #[test]
    fn test_command_error_from_io_error() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "io error");
        let error_kind = io_error.kind();
        let error = CommandError::from(io::Error::new(error_kind, "io error"));
        assert!(matches!(error, CommandError::IOError(_)));
    }

    #[test]
    fn test_get_first_directory_none() {
        // Create a temp file (not a directory)
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();
        let cmd = DebcrafterCmd::new("test");

        assert_eq!(cmd.get_first_directory(path), None);
    }

    #[test]
    fn test_get_first_directory_empty() {
        // Create an empty temp directory
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        let cmd = DebcrafterCmd::new("test");

        assert_eq!(cmd.get_first_directory(path), None);
    }

    #[test]
    fn test_get_first_directory_with_subdirectory() {
        // Create a temp directory with a subdirectory
        let temp_dir = TempDir::new().unwrap();
        let sub_dir_path = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir_path).unwrap();
        let cmd = DebcrafterCmd::new("test");

        let result = cmd.get_first_directory(temp_dir.path());
        assert!(result.is_some());
        assert_eq!(result.unwrap(), sub_dir_path);
    }

    #[test]
    fn test_copy_dir_contents_recursive() {
        // Create source directory structure
        let src_dir = TempDir::new().unwrap();
        let src_file1_path = src_dir.path().join("file1.txt");
        let src_subdir_path = src_dir.path().join("subdir");
        let src_file2_path = src_subdir_path.join("file2.txt");

        fs::create_dir(&src_subdir_path).unwrap();
        fs::write(&src_file1_path, b"test content 1").unwrap();
        fs::write(&src_file2_path, b"test content 2").unwrap();

        // Create destination directory
        let dest_dir = TempDir::new().unwrap();
        let cmd = DebcrafterCmd::new("test");

        // Copy the directory contents
        let result = cmd.copy_dir_contents_recursive(src_dir.path(), dest_dir.path());
        assert!(result.is_ok());

        // Verify the destination has the same structure and content
        let dest_file1_path = dest_dir.path().join("file1.txt");
        let dest_subdir_path = dest_dir.path().join("subdir");
        let dest_file2_path = dest_subdir_path.join("file2.txt");

        assert!(dest_file1_path.exists());
        assert!(dest_subdir_path.exists());
        assert!(dest_file2_path.exists());

        assert_eq!(
            fs::read_to_string(&dest_file1_path).unwrap(),
            "test content 1"
        );
        assert_eq!(
            fs::read_to_string(&dest_file2_path).unwrap(),
            "test content 2"
        );
    }

    #[test]
    fn test_copy_dir_contents_recursive_source_not_dir() {
        // Create a temp file (not a directory)
        let temp_file = NamedTempFile::new().unwrap();
        let dest_dir = TempDir::new().unwrap();
        let cmd = DebcrafterCmd::new("test");

        let result = cmd.copy_dir_contents_recursive(temp_file.path(), dest_dir.path());
        assert!(result.is_err());
    }

    // Integration tests that would run with actual commands should be marked as ignored by default

    #[test]
    #[ignore]
    fn test_check_if_dpkg_parsechangelog_installed_success() {
        // This test assumes dpkg-parsechangelog is installed
        let cmd = DebcrafterCmd::new("test");
        let result = cmd.check_if_dpkg_parsechangelog_installed();
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_check_if_installed_version_nonexistent() {
        // This test assumes a nonexistent debcrafter version
        let cmd = DebcrafterCmd::new("nonexistent_version");
        let result = cmd.check_if_installed();
        assert!(result.is_err());
    }
}
