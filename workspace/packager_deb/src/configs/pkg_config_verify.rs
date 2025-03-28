use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
pub struct PackageHash {
    pub name: String,
    pub hash: String,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
pub struct VerifyConfig {
    pub package_hash: Vec<PackageHash>,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
pub struct PkgVerifyConfig {
    pub verify: VerifyConfig,
}
