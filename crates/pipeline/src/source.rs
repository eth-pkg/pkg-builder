use std::fs;

use config::source::SourceKind;
use log::info;

use crate::context::Workspace;
use crate::PipelineError;

/// Acquire the source for the package (download, clone, or create empty tar).
pub fn acquire_source(ws: &Workspace) -> Result<(), PipelineError> {
    match &ws.config.source {
        SourceKind::Tarball { url, hash, .. } => {
            let is_web = url.starts_with("http");
            if is_web {
                info!("Downloading tar: {} to {:?}", url, ws.tarball_path);
                tool::archive::wget(url, &ws.tarball_path)?;
            } else {
                info!("Copying tar: {} to {:?}", url, ws.tarball_path);
                fs::copy(url, &ws.tarball_path).map_err(|e| PipelineError::Phase {
                    phase: "source",
                    message: format!("File copy failed: {}", e),
                })?;
            }

            if let Some(h) = hash {
                if !h.is_empty() {
                    verify_tarball_hash(&ws.tarball_path, h)?;
                }
            }
        }
        SourceKind::Git {
            url,
            tag,
            submodules,
            ..
        } => {
            tool::archive::clone_and_archive(
                url,
                tag,
                submodules,
                &ws.tarball_path,
                &ws.config.package.package_name,
                &ws.build_artifacts_dir,
            )?;
        }
        SourceKind::Virtual => {
            tool::archive::create_empty_tar(&ws.tarball_path, &ws.build_artifacts_dir)?;
        }
    }
    Ok(())
}

fn verify_tarball_hash(
    tarball_path: &std::path::Path,
    expected: &str,
) -> Result<(), PipelineError> {
    use sha2::{Digest, Sha256, Sha512};
    use std::io::Read;

    let mut file =
        fs::File::open(tarball_path).map_err(|e| PipelineError::Phase {
            phase: "verify_hash",
            message: format!("Failed to open tarball: {}", e),
        })?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(|e| PipelineError::Phase {
        phase: "verify_hash",
        message: format!("Failed to read tarball: {}", e),
    })?;

    // Try SHA-512 first
    let mut hasher = Sha512::new();
    hasher.update(&buffer);
    let sha512 = hex_encode(&hasher.finalize());
    info!("sha512 hash {}", &sha512);
    if sha512 == expected {
        return Ok(());
    }

    // Try SHA-256
    let mut hasher = Sha256::new();
    hasher.update(&buffer);
    let sha256 = hex_encode(&hasher.finalize());
    info!("sha256 hash {}", &sha256);
    if sha256 == expected {
        return Ok(());
    }

    Err(PipelineError::Phase {
        phase: "verify_hash",
        message: "Checksum verification failed: hashes do not match".to_string(),
    })
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_hex_encode() {
        assert_eq!(hex_encode(&[0xab, 0xcd, 0xef]), "abcdef");
        assert_eq!(hex_encode(&[0x00, 0xff]), "00ff");
        assert_eq!(hex_encode(&[]), "");
    }

    #[test]
    fn test_verify_tarball_hash_sha256() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.tar.gz");
        std::fs::write(&file_path, b"abc").unwrap();

        // SHA-256 of "abc"
        let sha256 = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        assert!(verify_tarball_hash(&file_path, sha256).is_ok());
    }

    #[test]
    fn test_verify_tarball_hash_sha512() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.tar.gz");
        std::fs::write(&file_path, b"abc").unwrap();

        // SHA-512 of "abc"
        let sha512 = "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f";
        assert!(verify_tarball_hash(&file_path, sha512).is_ok());
    }

    #[test]
    fn test_verify_tarball_hash_mismatch() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.tar.gz");
        std::fs::write(&file_path, b"abc").unwrap();

        let result = verify_tarball_hash(&file_path, "wrong_hash");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_tarball_hash_file_not_found() {
        let result = verify_tarball_hash(
            std::path::Path::new("/nonexistent/file.tar.gz"),
            "abc",
        );
        assert!(result.is_err());
    }
}
