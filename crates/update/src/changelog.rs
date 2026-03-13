use chrono::Utc;
use std::path::Path;

/// Generate a new Debian changelog entry.
///
/// Returns the formatted entry as a string, ready to be prepended to an existing changelog.
pub fn new_changelog_entry(
    package_name: &str,
    version: &str,
    revision: &str,
    distribution: &str,
    maintainer: &str,
    message: &str,
) -> String {
    let date = Utc::now().format("%a, %d %b %Y %H:%M:%S %z").to_string();
    format!(
        "{} ({}-{}) {}; urgency=medium\n\n  * {}\n\n -- {}  {}\n",
        package_name, version, revision, distribution, message, maintainer, date
    )
}

/// Prepend a new changelog entry to an existing debian/changelog file.
///
/// If the file doesn't exist, creates it with just the new entry.
pub fn prepend_changelog_entry(
    changelog_path: &Path,
    entry: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let existing = if changelog_path.exists() {
        std::fs::read_to_string(changelog_path)?
    } else {
        String::new()
    };

    let new_content = if existing.is_empty() {
        entry.to_string()
    } else {
        format!("{}\n{}", entry, existing)
    };

    if let Some(parent) = changelog_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(changelog_path, new_content)?;
    Ok(())
}

/// Try to extract the maintainer string from a .sss file.
///
/// Looks for a line like `maintainer: "Name <email>"` or `maintainer = "Name <email>"`.
pub fn extract_maintainer_from_sss(sss_path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(sss_path).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("maintainer") {
            let rest = rest.trim();
            let rest = rest.strip_prefix(':').or_else(|| rest.strip_prefix('='))?;
            let rest = rest.trim();
            let value = rest.trim_matches('"').trim_matches('\'');
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Try to find a .sss file in the package directory and extract the maintainer.
pub fn find_maintainer(config_root: &Path) -> Option<String> {
    if let Ok(entries) = std::fs::read_dir(config_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("sss") {
                if let Some(m) = extract_maintainer_from_sss(&path) {
                    return Some(m);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_new_changelog_entry_format() {
        let entry = new_changelog_entry(
            "geth",
            "1.15.0",
            "1",
            "bookworm",
            "Test User <test@example.com>",
            "New upstream version 1.15.0",
        );

        assert!(entry.starts_with("geth (1.15.0-1) bookworm; urgency=medium"));
        assert!(entry.contains("* New upstream version 1.15.0"));
        assert!(entry.contains("-- Test User <test@example.com>"));
    }

    #[test]
    fn test_prepend_to_existing_changelog() {
        let dir = tempdir().unwrap();
        let changelog_path = dir.path().join("debian").join("changelog");
        fs::create_dir_all(changelog_path.parent().unwrap()).unwrap();
        fs::write(
            &changelog_path,
            "old-pkg (1.0.0-1) bookworm; urgency=medium\n\n  * Old entry\n\n -- Old <old@test.com>  Mon, 01 Jan 2024 00:00:00 +0000\n",
        )
        .unwrap();

        let entry = new_changelog_entry(
            "old-pkg",
            "2.0.0",
            "1",
            "bookworm",
            "New <new@test.com>",
            "New upstream version 2.0.0",
        );
        prepend_changelog_entry(&changelog_path, &entry).unwrap();

        let content = fs::read_to_string(&changelog_path).unwrap();
        assert!(content.starts_with("old-pkg (2.0.0-1)"));
        assert!(content.contains("Old entry"));
    }

    #[test]
    fn test_extract_maintainer_from_sss() {
        let dir = tempdir().unwrap();
        let sss_path = dir.path().join("test.sss");
        fs::write(
            &sss_path,
            "name: test\nmaintainer: \"Test User <test@example.com>\"\n",
        )
        .unwrap();

        let m = extract_maintainer_from_sss(&sss_path);
        assert_eq!(m, Some("Test User <test@example.com>".to_string()));
    }

    #[test]
    fn test_prepend_creates_file_if_missing() {
        let dir = tempdir().unwrap();
        let changelog_path = dir.path().join("src").join("debian").join("changelog");

        let entry = new_changelog_entry(
            "test",
            "1.0.0",
            "1",
            "bookworm",
            "Test <test@test.com>",
            "Initial release",
        );
        prepend_changelog_entry(&changelog_path, &entry).unwrap();

        assert!(changelog_path.exists());
        let content = fs::read_to_string(&changelog_path).unwrap();
        assert!(content.contains("Initial release"));
    }
}
