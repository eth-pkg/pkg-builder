use crate::build_pipeline::{BuildContext, BuildError, BuildStep};
use log::info;
use std::{path::PathBuf, process::Command};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CreateEmptyTarError {
    #[error("Failed to create virtual package tarball")]
    TarballCreationFailed,
}
#[derive(Default)]
pub struct CreateEmptyTar {
    tarball_path: PathBuf,
    build_artifacts_dir: PathBuf,
}

impl From<BuildContext> for CreateEmptyTar {
    fn from(context: BuildContext) -> Self {
        CreateEmptyTar {
            build_artifacts_dir: context.build_artifacts_dir.clone(),
            tarball_path: context.tarball_path.clone(),
        }
    }
}

impl BuildStep for CreateEmptyTar {
    fn step(&self) -> Result<(), BuildError> {
        info!("Creating empty .tar.gz for virtual package");
        let output = Command::new("tar")
            .args([
                "czvf",
                &self.tarball_path.display().to_string(),
                "--files-from",
                "/dev/null",
            ])
            .current_dir(&self.build_artifacts_dir)
            .output()?;

        if !output.status.success() {
            return Err(CreateEmptyTarError::TarballCreationFailed.into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    impl BuildContext {
        pub fn new() -> Self {
            Self::default()
        }
    }
    #[test]
    fn test_download_source_virtual_package() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        let build_artifacts_dir = temp_dir.path().to_path_buf();
        let tarball_name = "test_package.tar.gz";
        let tarball_path = temp_dir.path().join(tarball_name);
        let tarball_path_str = temp_dir.path().join(tarball_name);

        let mut context = BuildContext::new();
        context.tarball_path = tarball_path_str;
        context.build_artifacts_dir = build_artifacts_dir;
        let step = CreateEmptyTar::from(context);

        let result = step.step();

        assert!(result.is_ok());
        assert!(tarball_path.exists());
    }
}
