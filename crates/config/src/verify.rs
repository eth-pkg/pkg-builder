use serde::Deserialize;
use sha1::{Digest, Sha1};
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

/// Verify that built package artifacts match the expected SHA1 hashes.
///
/// Returns `Ok(())` if all hashes match, or an error listing all mismatches.
pub fn verify_hashes(config: &PkgConfig) -> Result<(), Box<dyn std::error::Error>> {
    let verify_config = config
        .verify
        .as_ref()
        .ok_or("No [verify] section in pkg-builder.toml")?;

    let pkg = &config.package;
    let artifacts_dir = config.build_env.workdir.join(format!(
        "{}-{}-{}",
        pkg.name, pkg.version, pkg.revision
    ));

    verify_hashes_in_dir(verify_config, &artifacts_dir)
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

        let buffer = std::fs::read(&file_path)?;
        let mut hasher = Sha1::new();
        hasher.update(&buffer);
        let actual: String = hasher
            .finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();

        if actual != pkg_hash.hash {
            errors.push(format!(
                "SHA1 mismatch for {}: expected {}, got {}",
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
