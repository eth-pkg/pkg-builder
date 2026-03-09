use crate::build_env::normalize_snapshot_date;
use crate::{ConfigError, PkgConfig};

/// Validate the entire configuration.
pub fn validate_config(config: &mut PkgConfig) -> Result<(), ConfigError> {
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

    validate_snapshot_config(config)?;

    Ok(())
}

fn validate_snapshot_config(config: &mut PkgConfig) -> Result<(), ConfigError> {
    let has_snapshot = config.build_env.snapshot_date.is_some();
    let has_security = config.build_env.snapshot_security_date.is_some();

    if !has_snapshot && !has_security {
        return Ok(());
    }

    // snapshot_security_date requires snapshot_date
    if has_security && !has_snapshot {
        return Err(ConfigError::Validation(
            "snapshot_security_date requires snapshot_date to be set".into(),
        ));
    }

    // Snapshots are Debian-only
    if config.build_env.distribution.is_ubuntu() {
        return Err(ConfigError::Validation(
            "snapshot_date is only supported for Debian distributions (Ubuntu has no public snapshot service)".into(),
        ));
    }

    // Validate and normalize dates
    if let Some(ref date) = config.build_env.snapshot_date {
        let normalized =
            normalize_snapshot_date(date).map_err(|e| ConfigError::Validation(e))?;
        config.build_env.snapshot_date = Some(normalized);
    }
    if let Some(ref date) = config.build_env.snapshot_security_date {
        let normalized =
            normalize_snapshot_date(date).map_err(|e| ConfigError::Validation(e))?;
        config.build_env.snapshot_security_date = Some(normalized);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_env::*;
    use crate::package::PackageFields;
    use crate::source::SourceKind;
    use std::path::PathBuf;

    fn test_config_with_snapshot(
        distribution: Distribution,
        snapshot_date: Option<&str>,
        snapshot_security_date: Option<&str>,
    ) -> PkgConfig {
        PkgConfig {
            package: PackageFields {
                spec_file: "test.sss".into(),
                package_name: "test".to_string(),
                version_number: "1.0.0".to_string(),
                revision_number: "1".to_string(),
                homepage: "https://example.com".to_string(),
            },
            source: SourceKind::Virtual,
            build_env: BuildEnv {
                distribution,
                arch: Architecture::Amd64,
                pkg_builder_version: "0.3.1".to_string(),
                sbuild_cache_dir: PathBuf::from("/tmp/cache"),
                workdir: PathBuf::from("/tmp/work"),
                testing: TestingConfig {
                    run_lintian: false,
                    run_piuparts: false,
                    run_autopkgtest: false,
                },
                tool_versions: ToolVersions {
                    sbuild: "0.85.6".to_string(),
                    lintian: "2.116.3".to_string(),
                    piuparts: "1.1.7".to_string(),
                    autopkgtest: "5.28".to_string(),
                },
                snapshot_date: snapshot_date.map(String::from),
                snapshot_security_date: snapshot_security_date.map(String::from),
            },
            config_root: PathBuf::from("/test"),
        }
    }

    #[test]
    fn test_no_snapshot_passes() {
        let mut cfg = test_config_with_snapshot(Distribution::bookworm(), None, None);
        assert!(validate_config(&mut cfg).is_ok());
    }

    #[test]
    fn test_debian_snapshot_passes() {
        let mut cfg = test_config_with_snapshot(
            Distribution::bookworm(),
            Some("20250101T000000Z"),
            None,
        );
        assert!(validate_config(&mut cfg).is_ok());
    }

    #[test]
    fn test_debian_snapshot_short_date_normalized() {
        let mut cfg =
            test_config_with_snapshot(Distribution::bookworm(), Some("20250101"), None);
        assert!(validate_config(&mut cfg).is_ok());
        assert_eq!(
            cfg.build_env.snapshot_date.as_deref(),
            Some("20250101T000000Z")
        );
    }

    #[test]
    fn test_ubuntu_snapshot_rejected() {
        let mut cfg = test_config_with_snapshot(
            Distribution::noble(),
            Some("20250101T000000Z"),
            None,
        );
        let err = validate_config(&mut cfg).unwrap_err();
        assert!(err.to_string().contains("Ubuntu"));
    }

    #[test]
    fn test_security_without_main_rejected() {
        let mut cfg = test_config_with_snapshot(
            Distribution::bookworm(),
            None,
            Some("20250101T000000Z"),
        );
        let err = validate_config(&mut cfg).unwrap_err();
        assert!(err.to_string().contains("snapshot_security_date requires"));
    }

    #[test]
    fn test_invalid_date_rejected() {
        let mut cfg = test_config_with_snapshot(
            Distribution::bookworm(),
            Some("2025-01-01"),
            None,
        );
        let err = validate_config(&mut cfg).unwrap_err();
        assert!(err.to_string().contains("Invalid snapshot date"));
    }

    #[test]
    fn test_both_snapshot_dates_pass() {
        let mut cfg = test_config_with_snapshot(
            Distribution::bookworm(),
            Some("20250101T000000Z"),
            Some("20250115T000000Z"),
        );
        assert!(validate_config(&mut cfg).is_ok());
    }
}
