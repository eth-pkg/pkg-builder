use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Field: '{0}' cannot be empty")]
    EmptyField(String),

    #[error("Validation failed: {0}")]
    Multiple(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("Package hash cannot be empty")]
    EmptyPackageHash,
}
