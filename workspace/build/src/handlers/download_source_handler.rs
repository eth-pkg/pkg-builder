use crate::build_pipeline::{BuildContext, BuildError, BuildHandler};
use log::info;
use std::{fs, process::Command};

#[derive(Default)]
pub struct DownloadSourceHandler {
    pub tarball_path: Option<String>, // tarball to download to
    pub tarball_url: Option<String>,  // tarball from download
}

impl DownloadSourceHandler {
    pub fn new(tarball_path: String, tarball_url: String) -> Self {
        DownloadSourceHandler {
            tarball_path: Some(tarball_path),
            tarball_url: Some(tarball_url),
        }
    }
}

impl BuildHandler for DownloadSourceHandler {
    fn handle(&self, _context: &mut BuildContext) -> Result<(), BuildError> {
        let tarball_path = self.tarball_path.clone().ok_or(BuildError::MissingPath)?;
        let tarball_url = self.tarball_url.clone().ok_or(BuildError::MissingUrl)?;

        info!("Downloading source {}", tarball_path);
        let is_web = tarball_url.starts_with("http");

        if is_web {
            info!(
                "Downloading tar: {} to location: {}",
                tarball_url, tarball_path
            );
            let status = Command::new("wget")
                .arg("-q")
                .arg("-O")
                .arg(&tarball_path)
                .arg(&tarball_url)
                .status()
                .map_err(|e| BuildError::CommandFailed(e.to_string()))?;

            if !status.success() {
                return Err(BuildError::DownloadFailed);
            }
        } else {
            info!("Copying tar: {} to location: {}", tarball_url, tarball_path);
            fs::copy(&tarball_url, &tarball_path)
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

        let handler = DownloadSourceHandler::new(
            tarball_path.to_string_lossy().to_string(),
            tarball_url,
        );
        let mut context = BuildContext::default(); 

        let result = handler.handle(&mut context);

        assert!(result.is_ok());
        assert!(tarball_path.exists());
    }

    #[test]
    fn test_new_handler() {
        let handler = DownloadSourceHandler::new(
            "path/to/tarball.tar.gz".to_string(),
            "http://example.com/tarball.tar.gz".to_string(),
        );

        assert_eq!(
            handler.tarball_path,
            Some("path/to/tarball.tar.gz".to_string())
        );
        assert_eq!(
            handler.tarball_url,
            Some("http://example.com/tarball.tar.gz".to_string())
        );
    }

    #[test]
    fn test_default_handler() {
        let handler = DownloadSourceHandler::default();

        assert_eq!(handler.tarball_path, None);
        assert_eq!(handler.tarball_url, None);
    }

    #[test]
    fn test_handle_missing_path() {
        let handler = DownloadSourceHandler {
            tarball_path: None,
            tarball_url: Some("http://example.com/tarball.tar.gz".to_string()),
        };

        let mut context = BuildContext::default(); // You'd need to implement this

        let result = handler.handle(&mut context);
        assert!(result.is_err());

        match result {
            Err(BuildError::MissingPath) => {}
            _ => panic!("Expected MissingPath error"),
        }
    }

    #[test]
    fn test_handle_missing_url() {
        let handler = DownloadSourceHandler {
            tarball_path: Some("path/to/tarball.tar.gz".to_string()),
            tarball_url: None,
        };

        let mut context = BuildContext::default();

        let result = handler.handle(&mut context);
        assert!(result.is_err());

        match result {
            Err(BuildError::MissingUrl) => {}
            _ => panic!("Expected MissingUrl error"),
        }
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

        let handler = DownloadSourceHandler::new(
            dest_path.to_string_lossy().to_string(),
            source_path.to_string_lossy().to_string(),
        );

        let mut context = BuildContext::default();

        let result = handler.handle(&mut context);
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

        let handler = DownloadSourceHandler::new(
            dest_path.to_string_lossy().to_string(),
            source_path.to_string_lossy().to_string(),
        );

        let mut context = BuildContext::default();

        let result = handler.handle(&mut context);
        assert!(result.is_err());

        match result {
            Err(BuildError::FileCopyFailed(_)) => {}
            _ => panic!("Expected FileCopyFailed error"),
        }
    }
}
