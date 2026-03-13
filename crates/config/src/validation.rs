use crate::build_env::normalize_snapshot_date;
use crate::{ConfigError, PkgConfig};

/// Validate the entire configuration.
pub fn validate_and_normalize(config: &mut PkgConfig) -> Result<(), ConfigError> {
    if config.package.name.is_empty() {
        return Err(ConfigError::Validation(
            "package name cannot be empty".into(),
        ));
    }

    if config.package.version.is_empty() {
        return Err(ConfigError::Validation(
            "package version cannot be empty".into(),
        ));
    }

    if config.package.revision.is_empty() {
        return Err(ConfigError::Validation(
            "package revision cannot be empty".into(),
        ));
    }

    validate_source_fields(config)?;
    validate_snapshot_config(config)?;

    Ok(())
}

/// Characters that are unsafe in shell contexts (command injection).
const SHELL_UNSAFE: &[char] = &[
    ';', '&', '|', '$', '`', '(', ')', '{', '}', '<', '>', '!', '#', '\'', '"', '\\', '\n', '\r',
    '\0',
];

/// Validate that a string contains no shell metacharacters.
fn validate_safe_shell_value(field: &str, value: &str) -> Result<(), ConfigError> {
    if let Some(c) = value.chars().find(|c| SHELL_UNSAFE.contains(c)) {
        return Err(ConfigError::Validation(format!(
            "{} contains unsafe character '{}' — only alphanumeric, dots, hyphens, underscores, slashes, colons, and @ are allowed",
            field, c
        )));
    }
    Ok(())
}

/// Validate that a URL contains no shell metacharacters (allows :// and query strings).
fn validate_safe_url(field: &str, value: &str) -> Result<(), ConfigError> {
    // URLs may contain ? = & % + but not shell-dangerous chars like ; | ` $ ( ) { } etc.
    const URL_UNSAFE: &[char] = &[
        ';', '|', '`', '(', ')', '{', '}', '<', '>', '!', '\'', '"', '\\', '\n', '\r', '\0',
    ];
    if let Some(c) = value.chars().find(|c| URL_UNSAFE.contains(c)) {
        return Err(ConfigError::Validation(format!(
            "{} contains unsafe character '{}' — URLs must not contain shell metacharacters",
            field, c
        )));
    }
    Ok(())
}

/// Validate source-related fields that flow into shell commands in generated Makefiles.
fn validate_source_fields(config: &PkgConfig) -> Result<(), ConfigError> {
    use crate::source::SourceKind;

    // Package fields
    validate_safe_shell_value("package.name", &config.package.name)?;
    validate_safe_shell_value("package.version", &config.package.version)?;
    validate_safe_shell_value("package.revision", &config.package.revision)?;
    validate_safe_url("package.homepage", &config.package.homepage)?;

    match &config.source {
        SourceKind::Tarball { url, hash } => {
            validate_safe_url("source.url", url)?;
            if let Some(ref h) = hash {
                validate_safe_shell_value("source.hash", h)?;
            }
        }
        SourceKind::Git {
            url,
            tag,
            submodules,
        } => {
            validate_safe_url("source.url", url)?;
            validate_safe_shell_value("source.tag", tag)?;
            for (i, sub) in submodules.iter().enumerate() {
                validate_safe_shell_value(&format!("source.submodules[{}].path", i), &sub.path)?;
                if sub.path.contains("..") {
                    return Err(ConfigError::Validation(format!(
                        "source.submodules[{}].path must not contain '..'",
                        i
                    )));
                }
                validate_safe_shell_value(
                    &format!("source.submodules[{}].commit", i),
                    &sub.commit,
                )?;
            }
        }
        SourceKind::Virtual => {}
    }

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
        let normalized = normalize_snapshot_date(date).map_err(|e| ConfigError::Validation(e))?;
        config.build_env.snapshot_date = Some(normalized);
    }
    if let Some(ref date) = config.build_env.snapshot_security_date {
        let normalized = normalize_snapshot_date(date).map_err(|e| ConfigError::Validation(e))?;
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
                spec: "test.sss".into(),
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                revision: "1".to_string(),
                homepage: "https://example.com".to_string(),
            },
            source: SourceKind::Virtual,
            build_env: BuildEnv {
                distribution,
                arch: Architecture::Amd64,
                pkg_builder_version: "0.3.1".to_string(),
                chroot_dir: PathBuf::from("/tmp/cache"),
                workdir: PathBuf::from("/tmp/work"),
                testing: TestingConfig {
                    run_lintian: false,
                    run_piuparts: false,
                    run_autopkgtest: false,
                },
                tool_versions: ToolVersions {
                    debcrafter: "8189263".to_string(),
                    sbuild: "0.85.6".to_string(),
                    lintian: "2.116.3".to_string(),
                    piuparts: "1.1.7".to_string(),
                    autopkgtest: "5.28".to_string(),
                },
                snapshot_date: snapshot_date.map(String::from),
                snapshot_security_date: snapshot_security_date.map(String::from),
            },
            runtime: None,
            verify: None,
            config_root: PathBuf::from("/test"),
        }
    }

    #[test]
    fn test_no_snapshot_passes() {
        let mut cfg = test_config_with_snapshot(Distribution::bookworm(), None, None);
        assert!(validate_and_normalize(&mut cfg).is_ok());
    }

    #[test]
    fn test_debian_snapshot_passes() {
        let mut cfg =
            test_config_with_snapshot(Distribution::bookworm(), Some("20250101T000000Z"), None);
        assert!(validate_and_normalize(&mut cfg).is_ok());
    }

    #[test]
    fn test_debian_snapshot_short_date_normalized() {
        let mut cfg = test_config_with_snapshot(Distribution::bookworm(), Some("20250101"), None);
        assert!(validate_and_normalize(&mut cfg).is_ok());
        assert_eq!(
            cfg.build_env.snapshot_date.as_deref(),
            Some("20250101T000000Z")
        );
    }

    #[test]
    fn test_ubuntu_snapshot_rejected() {
        let mut cfg =
            test_config_with_snapshot(Distribution::noble(), Some("20250101T000000Z"), None);
        let err = validate_and_normalize(&mut cfg).unwrap_err();
        assert!(err.to_string().contains("Ubuntu"));
    }

    #[test]
    fn test_security_without_main_rejected() {
        let mut cfg =
            test_config_with_snapshot(Distribution::bookworm(), None, Some("20250101T000000Z"));
        let err = validate_and_normalize(&mut cfg).unwrap_err();
        assert!(err.to_string().contains("snapshot_security_date requires"));
    }

    #[test]
    fn test_invalid_date_rejected() {
        let mut cfg = test_config_with_snapshot(Distribution::bookworm(), Some("2025-01-01"), None);
        let err = validate_and_normalize(&mut cfg).unwrap_err();
        assert!(err.to_string().contains("Invalid snapshot date"));
    }

    #[test]
    fn test_both_snapshot_dates_pass() {
        let mut cfg = test_config_with_snapshot(
            Distribution::bookworm(),
            Some("20250101T000000Z"),
            Some("20250115T000000Z"),
        );
        assert!(validate_and_normalize(&mut cfg).is_ok());
    }
}
