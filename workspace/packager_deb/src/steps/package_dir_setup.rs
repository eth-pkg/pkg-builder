use crate::misc::build_pipeline::{BuildContext, BuildError, BuildStep};
use log::info;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct PackageDirSetup {
    build_artifacts_dir: PathBuf,
}

impl From<BuildContext> for PackageDirSetup {
    fn from(context: BuildContext) -> Self {
        PackageDirSetup {
            build_artifacts_dir: context.build_artifacts_dir.clone(),
        }
    }
}

impl BuildStep for PackageDirSetup {
    fn step(&self) -> Result<(), BuildError> {
        let build_artifacts_dir = &self.build_artifacts_dir;

        if Path::new(build_artifacts_dir).exists() {
            info!("Removing previous package folder {:?}", build_artifacts_dir);

            fs::remove_dir_all(build_artifacts_dir)?;
        }

        info!("Creating package folder {:?}", build_artifacts_dir);

        fs::create_dir_all(build_artifacts_dir)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::misc::build_pipeline::{BuildContext, BuildError};
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn setup_test_context() -> (BuildContext, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let artifacts_dir = temp_dir.path().join("artifacts");

        let mut context = BuildContext::new();
        context.build_artifacts_dir = artifacts_dir;

        (context, temp_dir)
    }

    #[test]
    fn test_handle_creates_new_directory() {
        let (context, _temp_dir) = setup_test_context();
        let handler = PackageDirSetup::from(context);

        let result = handler.step();

        assert!(result.is_ok());
        assert!(Path::new(&handler.build_artifacts_dir).exists());
        assert!(Path::new(&handler.build_artifacts_dir).is_dir());
    }

    #[test]
    fn test_handle_removes_existing_directory() {
        let (context, _temp_dir) = setup_test_context();

        fs::create_dir_all(&context.build_artifacts_dir).expect("Failed to create test directory");

        let test_file = PathBuf::from(&context.build_artifacts_dir).join("test_file.txt");
        fs::write(&test_file, "test content").expect("Failed to write test file");

        assert!(test_file.exists());

        let handler = PackageDirSetup::from(context);
        let result = handler.step();

        assert!(result.is_ok());
        assert!(Path::new(&handler.build_artifacts_dir).exists());
        assert!(Path::new(&handler.build_artifacts_dir).is_dir());

        assert!(!test_file.exists());
    }

    #[test]
    fn test_handle_with_permission_error() {
        // This test would ideally test the error case when directory operations fail
        // But it's hard to simulate in a portable way, so we're just checking the error mapping
        // In a real test environment, you might use mock_fs or similar libraries

        let (mut context, _temp_dir) = setup_test_context();

        context.build_artifacts_dir = "/root/forbidden_dir".into();

        let handler = PackageDirSetup::from(context);
        let result = handler.step();

        assert!(result.is_err());

        if let Err(BuildError::IoError(err)) = result {
            assert_eq!(err.to_string(), "Permission denied (os error 13)");
        } else {
            panic!("Expected IoError, got a different error or success");
        }
    }
}
