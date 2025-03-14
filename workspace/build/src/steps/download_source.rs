use crate::build_pipeline::{BuildContext, BuildError, BuildStep};
use log::info;
use std::{fs, process::Command};

#[derive(Default)]
pub struct DownloadSource {}

impl DownloadSource {
    pub fn new() -> Self {
        Self::default()
    }
}

impl BuildStep for DownloadSource {
    fn step(&self, context: &mut BuildContext) -> Result<(), BuildError> {
        info!("Downloading source {}", context.tarball_path);
        let is_web = context.tarball_url.starts_with("http");

        if is_web {
            info!(
                "Downloading tar: {} to location: {}",
                context.tarball_url, context.tarball_path
            );
            let status = Command::new("wget")
                .arg("-q")
                .arg("-O")
                .arg(&context.tarball_path)
                .arg(&context.tarball_url)
                .status()
                .map_err(|e| BuildError::CommandFailed(e.to_string()))?;

            if !status.success() {
                return Err(BuildError::DownloadFailed);
            }
        } else {
            info!(
                "Copying tar: {} to location: {}",
                context.tarball_url, context.tarball_path
            );
            fs::copy(&context.tarball_url, &context.tarball_path)
                .map_err(|e| BuildError::FileCopyFailed(e.to_string()))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    use tempfile::tempdir;

    use httpmock::prelude::*;

    fn setup_mock_server() -> MockServer {
        // Start the mock server
        let server = MockServer::start();

        // Mock the endpoint to serve the tarball file
        server.mock(|when, then| {
            when.method(GET).path("/test_package.tar.gz");
            then.status(200)
                .header("Content-Type", "application/octet-stream")
                .body_from_file("tests/misc/test_package.tar.gz");
        });

        server
    }
    #[test]
    fn test_download_source_non_virtual_package() {
        let server = setup_mock_server();

        let temp_dir = tempdir().expect("Failed to create temporary directory");

        let tarball_name = "test_package.tar.gz";
        let tarball_path = temp_dir.path().join(tarball_name);
        let tarball_url = format!("{}/{}", server.base_url(), tarball_name);

        let handler = DownloadSource::new();
        let mut context = BuildContext::default();
        context.tarball_path = tarball_path.to_string_lossy().to_string();
        context.tarball_url = tarball_url;

        let result = handler.step(&mut context);

        assert!(result.is_ok());
        assert!(tarball_path.exists());
    }

    #[test]
    fn test_handle_local_copy_success() {
        // Create a temporary directory for testing
        let dir = tempdir().unwrap();
        let source_path = dir.path().join("source.tar.gz");
        let dest_path = dir.path().join("dest.tar.gz");

        // Create a dummy source file
        {
            let mut file = File::create(&source_path).unwrap();
            writeln!(file, "test content").unwrap();
        }

        let handler = DownloadSource::new();

        let mut context = BuildContext::default();
        context.tarball_path = dest_path.to_string_lossy().to_string();
        context.tarball_url = source_path.to_string_lossy().to_string();
        let result = handler.step(&mut context);
        assert!(result.is_ok());

        // Verify the file was copied
        assert!(Path::new(&dest_path).exists());
    }

    #[test]
    fn test_handle_local_copy_failure() {
        // Create a temporary directory for testing
        let dir = tempdir().unwrap();
        let source_path = dir.path().join("nonexistent_source.tar.gz");
        let dest_path = dir.path().join("dest.tar.gz");

        // No source file exists

        let handler = DownloadSource::new();

        let mut context = BuildContext::default();
        context.tarball_path = dest_path.to_string_lossy().to_string();
        context.tarball_url = source_path.to_string_lossy().to_string();
        let result = handler.step(&mut context);
        assert!(result.is_err());

        match result {
            Err(BuildError::FileCopyFailed(_)) => {}
            _ => panic!("Expected FileCopyFailed error"),
        }
    }
}
