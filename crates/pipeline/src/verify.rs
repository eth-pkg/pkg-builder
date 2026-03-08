use log::info;
use sha1::{Digest, Sha1};

use config::verify::PkgVerifyConfig;

use crate::context::Workspace;
use crate::PipelineError;

/// Verify package hashes against a verify config. Optionally rebuild first.
pub fn verify_package(
    ws: &Workspace,
    verify_config: PkgVerifyConfig,
    skip_build: bool,
) -> Result<(), PipelineError> {
    if !skip_build {
        crate::build::build_package(ws)?;
    }

    let output_dir = ws
        .build_files_dir
        .parent()
        .ok_or(PipelineError::Phase {
            phase: "verify",
            message: "Invalid build files dir".to_string(),
        })?;

    let mut errors = Vec::new();

    for pkg_hash in &verify_config.verify.package_hash {
        let file_path = output_dir.join(&pkg_hash.name);

        if !file_path.exists() {
            errors.push(format!("Verification file missing: {}", pkg_hash.name));
            continue;
        }

        let buffer = std::fs::read(&file_path).map_err(|_| PipelineError::Phase {
            phase: "verify",
            message: format!("Failed to read file: {}", pkg_hash.name),
        })?;

        let actual_sha1 = calculate_sha1(&buffer);

        if actual_sha1 != pkg_hash.hash {
            errors.push(format!(
                "SHA1 mismatch for {}: expected {}, got {}",
                pkg_hash.name, pkg_hash.hash, actual_sha1
            ));
        }
    }

    if errors.is_empty() {
        info!("Verification successful!");
        Ok(())
    } else {
        Err(PipelineError::Phase {
            phase: "verify",
            message: errors.join("; "),
        })
    }
}

fn calculate_sha1(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect()
}
