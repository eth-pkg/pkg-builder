use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use crate::PkgConfig;

/// Hash entry for a package file.
#[derive(Debug, Clone, Deserialize)]
pub struct PackageHash {
    pub name: String,
    pub hash: String,
}

/// Verification configuration, containing expected hashes.
#[derive(Debug, Clone, Deserialize)]
pub struct VerifyConfig {
    pub package_hash: Vec<PackageHash>,
}

/// Verify that built package artifacts match the expected SHA-256 hashes.
///
/// Returns `Ok(())` if all hashes match, or an error listing all mismatches.
pub fn verify_hashes(config: &PkgConfig) -> Result<(), Box<dyn std::error::Error>> {
    let verify_config = config
        .verify
        .as_ref()
        .ok_or("No [verify] section in pkg-builder.toml")?;

    let pkg = &config.package;
    let artifacts_dir = config
        .build_env
        .workdir
        .join(format!("{}-{}-{}", pkg.name, pkg.version, pkg.revision));

    verify_hashes_in_dir(verify_config, &artifacts_dir)
}

/// Compute SHA-256 hash of a file using streaming reads.
fn sha256_file(path: &Path) -> Result<String, std::io::Error> {
    let file = File::open(path)?;
    let mut reader = BufReader::with_capacity(8192, file);
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect())
}

/// Verify package hashes against files in a specific directory.
pub fn verify_hashes_in_dir(
    verify_config: &VerifyConfig,
    artifacts_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut errors = Vec::new();

    for pkg_hash in &verify_config.package_hash {
        let file_path = artifacts_dir.join(&pkg_hash.name);

        if !file_path.exists() {
            errors.push(format!("Verification file missing: {}", pkg_hash.name));
            continue;
        }

        let actual = sha256_file(&file_path)?;

        if actual != pkg_hash.hash {
            errors.push(format!(
                "SHA-256 mismatch for {}: expected {}, got {}",
                pkg_hash.name, pkg_hash.hash, actual
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; ").into())
    }
}
