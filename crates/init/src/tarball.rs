use log::warn;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::Path;

use crate::http_client;

/// Result of downloading and hashing a tarball.
pub struct TarballResult {
    /// The computed sha256 hash.
    pub hash: String,
    /// The local path where the tarball was saved (if downloaded from a URL).
    pub local_path: Option<std::path::PathBuf>,
}

/// Download a tarball from `url`, save it to `output_dir`, and compute its sha256 hash.
///
/// If `url` is a local file path, hashes it in place without downloading.
/// If `upstream_hash` is provided, verifies the computed hash matches.
pub fn download_and_hash_tarball(
    url: &str,
    output_dir: &Path,
    upstream_hash: Option<&str>,
) -> Result<TarballResult, Box<dyn std::error::Error>> {
    // Local file: hash it directly
    if !url.starts_with("http://") && !url.starts_with("https://") {
        if Path::new(url).exists() {
            let hash = sha256_file(url)?;
            return Ok(TarballResult {
                hash,
                local_path: None,
            });
        }
        return Err(format!("Local file not found: {}", url).into());
    }

    let response = http_client().get(url).send()?;
    if !response.status().is_success() {
        return Err(format!("Failed to download tarball (HTTP {})", response.status()).into());
    }
    let bytes = response.bytes()?;

    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let hash = format!("{:x}", hasher.finalize());

    // Verify against upstream hash if provided
    if let Some(upstream) = upstream_hash {
        if hash != upstream {
            return Err(
                format!("Hash mismatch! Computed: {}, Expected: {}", hash, upstream).into(),
            );
        }
    } else {
        try_verify_upstream_hash(url, &hash);
    }

    // Save tarball locally
    let filename = url.rsplit('/').next().unwrap_or("source.tar.gz");
    let local_path = output_dir.join(filename);
    fs::write(&local_path, &bytes)?;

    Ok(TarballResult {
        hash,
        local_path: Some(local_path),
    })
}

/// Compute the sha256 hash of a local file.
pub fn sha256_file(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Try to verify the computed hash against upstream checksums.
///
/// Attempts `{url}.sha256`, `{url}.sha256sum`, and `SHA256SUMS` in the same directory.
/// Logs warnings on mismatch, but does not return an error.
pub fn try_verify_upstream_hash(url: &str, computed_hash: &str) {
    for suffix in &[".sha256", ".sha256sum"] {
        let hash_url = format!("{}{}", url, suffix);
        if let Ok(resp) = http_client().get(&hash_url).send() {
            if resp.status().is_success() {
                if let Ok(text) = resp.text() {
                    let upstream = text.trim().split_whitespace().next().unwrap_or("");
                    if upstream == computed_hash {
                        log::info!("Upstream hash verified via {}", hash_url);
                    } else if !upstream.is_empty() {
                        warn!(
                            "Upstream hash mismatch from {}: expected {}, got {}",
                            hash_url, upstream, computed_hash
                        );
                    }
                    return;
                }
            }
        }
    }

    if let Some(dir_url) = url
        .rsplit_once('/')
        .map(|(base, _)| format!("{}/SHA256SUMS", base))
    {
        if let Ok(resp) = http_client().get(&dir_url).send() {
            if resp.status().is_success() {
                if let Ok(text) = resp.text() {
                    let filename = url.rsplit('/').next().unwrap_or("");
                    for line in text.lines() {
                        if line.contains(filename) {
                            let hash = line.split_whitespace().next().unwrap_or("");
                            if hash == computed_hash {
                                log::info!("Upstream hash verified via SHA256SUMS");
                            } else {
                                warn!("Upstream hash mismatch from SHA256SUMS");
                            }
                            return;
                        }
                    }
                }
            }
        }
    }
}
