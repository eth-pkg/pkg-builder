use crate::{ConfigError, PkgConfig};

/// Validate the entire configuration.
pub fn validate_config(config: &PkgConfig) -> Result<(), ConfigError> {
    if config.package.package_name.is_empty() {
        return Err(ConfigError::Validation(
            "package_name cannot be empty".into(),
        ));
    }

    if config.package.version_number.is_empty() {
        return Err(ConfigError::Validation(
            "version_number cannot be empty".into(),
        ));
    }

    if config.package.revision_number.is_empty() {
        return Err(ConfigError::Validation(
            "revision_number cannot be empty".into(),
        ));
    }

    Ok(())
}
