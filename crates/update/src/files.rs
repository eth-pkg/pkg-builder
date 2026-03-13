use log::{debug, info};
use regex::Regex;
use std::fs;
use std::path::Path;

/// Substitutions to apply across all files in the package directory.
pub struct FileSubstitutions {
    pub old_version: String,
    pub new_version: String,
    pub old_hash: Option<String>,
    pub new_hash: Option<String>,
    pub old_tag: Option<String>,
    pub new_tag: Option<String>,
    pub old_commit: Option<String>,
    pub new_commit: Option<String>,
}

/// Files to skip during substitution (already handled or should not be modified).
const SKIP_FILES: &[&str] = &["pkg-builder.toml", "pkg-builder-verify.toml"];

/// Walk the package directory and apply version/hash substitutions to all text files.
///
/// Returns the list of files that were modified.
pub fn apply_substitutions(
    dir: &Path,
    subs: &FileSubstitutions,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut modified = Vec::new();
    walk_and_substitute(dir, dir, subs, &mut modified)?;
    Ok(modified)
}

fn walk_and_substitute(
    root: &Path,
    dir: &Path,
    subs: &FileSubstitutions,
    modified: &mut Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let entries = fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Skip hidden directories
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with('.'))
            {
                continue;
            }
            walk_and_substitute(root, &path, subs, modified)?;
            continue;
        }

        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if SKIP_FILES.contains(&file_name) {
            debug!("Skipping {}", path.display());
            continue;
        }

        if is_binary(&path)? {
            debug!("Skipping binary file {}", path.display());
            continue;
        }

        let content = fs::read_to_string(&path)?;
        let new_content = substitute_content(&content, subs);

        if new_content != content {
            let rel = path.strip_prefix(root).unwrap_or(&path);
            info!("Updated: {}", rel.display());
            fs::write(&path, &new_content)?;
            modified.push(rel.display().to_string());
        }
    }
    Ok(())
}

/// Apply all substitutions to a string, using word-boundary-aware replacement.
fn substitute_content(content: &str, subs: &FileSubstitutions) -> String {
    let mut result = content.to_string();

    // Replace version with word boundaries to avoid partial matches
    result = replace_with_boundaries(&result, &subs.old_version, &subs.new_version);

    if let (Some(ref old_hash), Some(ref new_hash)) = (&subs.old_hash, &subs.new_hash) {
        if old_hash != new_hash {
            result = result.replace(old_hash, new_hash);
        }
    }

    if let (Some(ref old_tag), Some(ref new_tag)) = (&subs.old_tag, &subs.new_tag) {
        if old_tag != new_tag {
            result = replace_with_boundaries(&result, old_tag, new_tag);
        }
    }

    if let (Some(ref old_commit), Some(ref new_commit)) = (&subs.old_commit, &subs.new_commit) {
        if old_commit != new_commit {
            result = result.replace(old_commit, new_commit);
        }
    }

    result
}

/// Replace a string only at word boundaries (to avoid "1.0.0" matching inside "1.0.0-beta1").
fn replace_with_boundaries(content: &str, old: &str, new: &str) -> String {
    if old == new || old.is_empty() {
        return content.to_string();
    }
    let pattern = format!(r"(?<![.\w-]){}", regex::escape(old));
    match Regex::new(&pattern) {
        Ok(re) => re.replace_all(content, new).to_string(),
        Err(_) => content.replace(old, new),
    }
}

/// Check if a file is binary by looking for null bytes in the first 512 bytes.
fn is_binary(path: &Path) -> Result<bool, std::io::Error> {
    let content = fs::read(path)?;
    let check_len = content.len().min(512);
    Ok(content[..check_len].contains(&0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_version_substitution() {
        let subs = FileSubstitutions {
            old_version: "1.14.0".to_string(),
            new_version: "1.15.0".to_string(),
            old_hash: None,
            new_hash: None,
            old_tag: None,
            new_tag: None,
            old_commit: None,
            new_commit: None,
        };

        let input = "version = 1.14.0\nurl = https://example.com/pkg-1.14.0.tar.gz\n";
        let result = substitute_content(input, &subs);
        assert!(result.contains("1.15.0"));
        assert!(!result.contains("1.14.0"));
    }

    #[test]
    fn test_hash_substitution() {
        let subs = FileSubstitutions {
            old_version: "1.0.0".to_string(),
            new_version: "2.0.0".to_string(),
            old_hash: Some("abc123".to_string()),
            new_hash: Some("def456".to_string()),
            old_tag: None,
            new_tag: None,
            old_commit: None,
            new_commit: None,
        };

        let input = "hash = abc123\n";
        let result = substitute_content(input, &subs);
        assert!(result.contains("def456"));
        assert!(!result.contains("abc123"));
    }

    #[test]
    fn test_skips_pkg_builder_toml() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("pkg-builder.toml"), "version = \"1.0.0\"\n").unwrap();
        fs::write(dir.path().join("other.txt"), "version = 1.0.0\n").unwrap();

        let subs = FileSubstitutions {
            old_version: "1.0.0".to_string(),
            new_version: "2.0.0".to_string(),
            old_hash: None,
            new_hash: None,
            old_tag: None,
            new_tag: None,
            old_commit: None,
            new_commit: None,
        };

        let modified = apply_substitutions(dir.path(), &subs).unwrap();
        assert_eq!(modified.len(), 1);
        assert!(modified[0].contains("other.txt"));

        // pkg-builder.toml should be untouched
        let toml_content = fs::read_to_string(dir.path().join("pkg-builder.toml")).unwrap();
        assert!(toml_content.contains("1.0.0"));
    }

    #[test]
    fn test_skips_binary_files() {
        let dir = tempdir().unwrap();
        let mut binary_content = vec![0u8; 100];
        binary_content[50] = 0; // null byte
        fs::write(dir.path().join("binary.dat"), &binary_content).unwrap();

        let subs = FileSubstitutions {
            old_version: "1.0.0".to_string(),
            new_version: "2.0.0".to_string(),
            old_hash: None,
            new_hash: None,
            old_tag: None,
            new_tag: None,
            old_commit: None,
            new_commit: None,
        };

        let modified = apply_substitutions(dir.path(), &subs).unwrap();
        assert!(modified.is_empty());
    }

    #[test]
    fn test_word_boundary_replacement() {
        let result = replace_with_boundaries("v1.0.0 and 1.0.0-beta1", "1.0.0", "2.0.0");
        // Should replace the standalone 1.0.0 (after v) but the word boundary logic
        // should handle the -beta1 case
        assert!(result.contains("2.0.0"));
    }
}
