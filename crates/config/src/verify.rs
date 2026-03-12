use serde::Deserialize;

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
