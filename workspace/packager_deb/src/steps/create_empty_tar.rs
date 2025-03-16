use crate::build_pipeline::{BuildContext, BuildError, BuildStep};
use log::info;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CreateEmptyTarError {
    #[error("Failed to create virtual package tarball")]
    TarballCreationFailed,
}
#[derive(Default)]
pub struct CreateEmptyTar {}

impl CreateEmptyTar {
    pub fn new() -> Self {
        Self::default()
    }
}

impl BuildStep for CreateEmptyTar {
    fn step(&self, context: &mut BuildContext) -> Result<(), BuildError> {
        info!("Creating empty .tar.gz for virtual package");
        let output = Command::new("tar")
            .args(["czvf", &context.tarball_path, "--files-from", "/dev/null"])
            .current_dir(&context.build_artifacts_dir)
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

    #[test]
    fn test_download_source_virtual_package() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        let build_artifacts_dir = String::from(temp_dir.path().to_str().unwrap());
        let tarball_name = "test_package.tar.gz";
        let tarball_path = temp_dir.path().join(tarball_name);
        let tarball_path_str = String::from(temp_dir.path().join(tarball_name).to_str().unwrap());


        let step = CreateEmptyTar::new();
        let mut context = BuildContext::new();
        context.tarball_path = tarball_path_str;
        context.build_artifacts_dir = build_artifacts_dir;
        let result = step.step(&mut context);


        assert!(result.is_ok());
        assert!(tarball_path.exists());
    }
}
