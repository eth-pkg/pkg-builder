use std::path::PathBuf;

use serde::Deserialize;

/// Package metadata fields from the [package_fields] section.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PackageFields {
    pub spec_file: PathBuf,
    pub package_name: String,
    pub version_number: String,
    pub revision_number: String,
    pub homepage: String,
}
