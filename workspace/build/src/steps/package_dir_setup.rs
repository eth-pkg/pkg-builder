use crate::build_pipeline::{BuildContext, BuildError, BuildStep};
use log::info;
use std::{fs, path::Path};

#[derive(Default)]
pub struct PackageDirSetup {}

impl PackageDirSetup {
    pub fn new() -> Self {
        Self::default()
    }
}

impl BuildStep for PackageDirSetup {
    fn step(&self, context: &mut BuildContext) -> Result<(), BuildError> {
        let debian_artifacts_dir = &context.debian_artifacts_dir;

        if Path::new(debian_artifacts_dir).exists() {
            info!("Removing previous package folder {}", debian_artifacts_dir);

            fs::remove_dir_all(debian_artifacts_dir)?;
        }

        info!("Creating package folder {}", debian_artifacts_dir);

        fs::create_dir_all(debian_artifacts_dir)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_pipeline::{BuildContext, BuildError};
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn setup_test_context() -> (BuildContext, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let artifacts_dir = temp_dir.path().join("artifacts");

        let mut context = BuildContext::new();
        context.debian_artifacts_dir = artifacts_dir.to_string_lossy().to_string();

        (context, temp_dir)
    }

    #[test]
    fn test_handle_creates_new_directory() {
        let (mut context, _temp_dir) = setup_test_context();
        let handler = PackageDirSetup::new();

        let result = handler.step(&mut context);

        assert!(result.is_ok());
        assert!(Path::new(&context.debian_artifacts_dir).exists());
        assert!(Path::new(&context.debian_artifacts_dir).is_dir());
    }

    #[test]
    fn test_handle_removes_existing_directory() {
        let (mut context, _temp_dir) = setup_test_context();

        fs::create_dir_all(&context.debian_artifacts_dir).expect("Failed to create test directory");

        let test_file = PathBuf::from(&context.debian_artifacts_dir).join("test_file.txt");
        fs::write(&test_file, "test content").expect("Failed to write test file");

        assert!(test_file.exists());

        let handler = PackageDirSetup::new();
        let result = handler.step(&mut context);

        assert!(result.is_ok());
        assert!(Path::new(&context.debian_artifacts_dir).exists());
        assert!(Path::new(&context.debian_artifacts_dir).is_dir());

        assert!(!test_file.exists());
    }

    #[test]
    fn test_handle_with_permission_error() {
        // This test would ideally test the error case when directory operations fail
        // But it's hard to simulate in a portable way, so we're just checking the error mapping
        // In a real test environment, you might use mock_fs or similar libraries

        let (mut context, _temp_dir) = setup_test_context();

        context.debian_artifacts_dir = "/root/forbidden_dir".to_string();

        let handler = PackageDirSetup::new();
        let result = handler.step(&mut context);

        assert!(result.is_err());

        if let Err(BuildError::IoError(err)) = result {
            assert_eq!(err.to_string(), "Permission denied (os error 13)");
        } else {
            panic!("Expected IoError, got a different error or success");
        }
    }

}
