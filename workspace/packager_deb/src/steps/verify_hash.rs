use crate::build_pipeline::{BuildContext, BuildError, BuildStep};
use log::info;
use sha2::{Digest, Sha256, Sha512};
use std::fs;
use std::io::Read;

#[derive(Default)]
pub struct VerifyHash {
    tarball_path: String,
    tarball_hash: String,
}

impl From<BuildContext> for VerifyHash {
    fn from(context: BuildContext) -> Self {
        VerifyHash {
            tarball_path: context.tarball_path.clone(),
            tarball_hash: context.tarball_hash.clone(),
        }
    }
}

impl VerifyHash {
    fn calculate_sha256<R: Read>(mut reader: R) -> Result<String, BuildError> {
        let mut hasher = Sha256::new();
        std::io::copy(&mut reader, &mut hasher)?;
        let digest_bytes = hasher.finalize();
        let hex_digest = digest_bytes
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        Ok(hex_digest)
    }

    fn calculate_sha512<R: Read>(mut reader: R) -> Result<String, BuildError> {
        let mut hasher = Sha512::new();
        std::io::copy(&mut reader, &mut hasher)?;
        let digest_bytes = hasher.finalize();
        let hex_digest = digest_bytes
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        Ok(hex_digest)
    }

    fn verify_tarball_checksum(
        &self,
        tarball_path: &str,
        expected_checksum: &str,
    ) -> Result<bool, BuildError> {
        let mut file = fs::File::open(tarball_path)
            .map_err(|err| BuildError::TarballOpenError(err.to_string()))?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|err| BuildError::TarballReadError(err.to_string()))?;

        // Try SHA-512 first
        let actual_sha512 = Self::calculate_sha512(&*buffer).unwrap_or_default();
        info!("sha512 hash {}", &actual_sha512);
        if actual_sha512 == expected_checksum {
            return Ok(true);
        }

        // If SHA-512 doesn't match, try SHA-256
        let actual_sha256 = Self::calculate_sha256(&*buffer).unwrap_or_default();
        info!("sha256 hash {}", &actual_sha256);
        if actual_sha256 == expected_checksum {
            return Ok(true);
        }

        Err(BuildError::HashMismatchError)
    }
}

impl BuildStep for VerifyHash {
    fn step(&self) -> Result<(), BuildError> {
        match self.verify_tarball_checksum(&self.tarball_path, &self.tarball_hash) {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    fn create_temp_file(content: &[u8]) -> Result<(tempfile::TempDir, String), std::io::Error> {
        let dir = tempdir()?;
        let file_path = dir.path().join("test_tarball.tar.gz");
        let mut file = File::create(&file_path)?;
        file.write_all(content)?;
        Ok((dir, file_path.to_string_lossy().to_string()))
    }

    fn read_tarball(tarball_path: &str) -> Result<Vec<u8>, std::io::Error> {
        let mut file = std::fs::File::open(tarball_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    #[test]
    fn test_calculate_sha256() {
        let data = b"abc";
        let expected = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

        let result = VerifyHash::calculate_sha256(&data[..]).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_calculate_sha512() {
        let data = b"abc";
        let expected = "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f";

        let result = VerifyHash::calculate_sha512(&data[..]).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_verify_tarball_checksum_sha256_success() {
        let content = b"test content";
        let (_dir, file_path) = create_temp_file(content).unwrap();
        let buffer = read_tarball(&file_path).unwrap();

        let expected_checksum = VerifyHash::calculate_sha256(&*buffer).unwrap();

        let handler = VerifyHash::from(BuildContext::new());

        let result = handler.verify_tarball_checksum(&file_path, &expected_checksum);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_verify_tarball_checksum_sha512_success() {
        let content = b"test content";
        let (_dir, file_path) = create_temp_file(content).unwrap();
        let buffer = read_tarball(&file_path).unwrap();

        let expected_checksum = VerifyHash::calculate_sha512(&*buffer).unwrap();

        let handler = VerifyHash::from(BuildContext::new());

        let result = handler.verify_tarball_checksum(&file_path, &expected_checksum);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_verify_tarball_checksum_mismatch() {
        let content = b"test content";
        let (_dir, file_path) = create_temp_file(content).unwrap();

        let incorrect_checksum = "0000000000000000000000000000000000000000000000000000000000000000";

        let handler = VerifyHash::from(BuildContext::new());

        let result = handler.verify_tarball_checksum(&file_path, &incorrect_checksum);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BuildError::HashMismatchError));
    }

    #[test]
    fn test_handle_with_checksum() {
        let content = b"test content";
        let (_dir, file_path) = create_temp_file(content).unwrap();
        let buffer = read_tarball(&file_path).unwrap();

        let expected_checksum = VerifyHash::calculate_sha256(&*buffer).unwrap();

        let mut context = BuildContext::default();
        context.tarball_hash = expected_checksum;
        context.tarball_path = file_path;
        let handler: VerifyHash = VerifyHash::from(context);

        let result = handler.step();
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_without_checksum() {
        let mut context: BuildContext = BuildContext::default();
        context.tarball_path = "some/path".to_string();
        let handler: VerifyHash = VerifyHash::from(context);

        let result = handler.step();
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_hash_valid_checksum_512() {
        let tarball_path = "tests/misc/test_package.tar.gz";
        let expected_checksum = "abd0b8e99f983926dbf60bdcbaef13f83ec7b31d56e68f6252ed05981b237c837044ce768038fc34b71f925e2fb19b7dee451897db512bb4a99e0e1bc96d8ab3";
        let mut context: BuildContext = BuildContext::default();
        context.tarball_hash = expected_checksum.to_string();
        context.tarball_path = tarball_path.to_string();
        let handler: VerifyHash = VerifyHash::from(context);

        let result = handler.step();

        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_hash_invalid_checksum_512() {
        let tarball_path = "tests/misc/test_package.tar.gz";
        let expected_checksum = "abd0b8e99f983926dbf60bdcbaef13f83ec7b31d56e68f6252ed05981b237c837044ce768038fc34b71f925e2fb19b7dee451897db512bb4a99e0e1bc96d8ab2";

        let mut context: BuildContext = BuildContext::default();
        context.tarball_hash = expected_checksum.to_string();
        context.tarball_path = tarball_path.to_string();

        let handler = VerifyHash::from(context);
        let result = handler.step();

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            "Checksum verification failed: hashes do not match"
        );
    }

    #[test]
    fn test_verify_hash_valid_checksum_256() {
        let tarball_path = "tests/misc/test_package.tar.gz";
        let expected_checksum = "b610e83c026d4c465636779240b6ed40a076593a61df5f6b9f9f59f1a929478d";

        let mut context: BuildContext = BuildContext::default();
        context.tarball_hash = expected_checksum.to_string();
        context.tarball_path = tarball_path.to_string();
        let handler = VerifyHash::from(context);

        let result = handler.step();

        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_hash_invalid_checksum_256() {
        let tarball_path = "tests/misc/test_package.tar.gz";
        let expected_checksum = "b610e83c026d4c465636779240b6ed40a076593a61df5f6b9f9f59f1a929478_";

        let mut context: BuildContext = BuildContext::default();
        context.tarball_hash = expected_checksum.to_string();
        context.tarball_path = tarball_path.to_string();
        let handler = VerifyHash::from(context);

        let result = handler.step();

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            "Checksum verification failed: hashes do not match"
        );
    }
}
