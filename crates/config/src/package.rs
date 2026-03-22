use std::path::PathBuf;

use serde::Deserialize;

/// Package metadata fields from the [package] section.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PackageFields {
    pub name: String,
    pub version: String,
    pub revision: String,
    pub homepage: String,
    pub spec: PathBuf,
}
