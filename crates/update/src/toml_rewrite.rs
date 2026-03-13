use std::path::Path;
use toml_edit::DocumentMut;

/// Fields that can be updated in the TOML document.
pub struct TomlUpdates {
    pub version: Option<String>,
    pub revision: Option<String>,
    pub source_url: Option<String>,
    pub source_hash: Option<String>,
    pub source_tag: Option<String>,
    pub runtime_vars: Option<Vec<(String, String)>>,
}

/// Rewrite specific fields in a pkg-builder.toml file, preserving formatting and comments.
///
/// Returns the modified TOML content as a string.
pub fn rewrite_toml(
    toml_path: &Path,
    updates: &TomlUpdates,
) -> Result<String, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(toml_path)?;
    let mut doc: DocumentMut = content.parse()?;

    if let Some(ref version) = updates.version {
        doc["package"]["version"] = toml_edit::value(version.as_str());
    }
    if let Some(ref revision) = updates.revision {
        doc["package"]["revision"] = toml_edit::value(revision.as_str());
    }
    if let Some(ref url) = updates.source_url {
        doc["source"]["url"] = toml_edit::value(url.as_str());
    }
    if let Some(ref hash) = updates.source_hash {
        doc["source"]["hash"] = toml_edit::value(hash.as_str());
    }
    if let Some(ref tag) = updates.source_tag {
        doc["source"]["tag"] = toml_edit::value(tag.as_str());
    }
    if let Some(ref vars) = updates.runtime_vars {
        if let Some(runtime) = doc.get_mut("runtime") {
            for (key, value) in vars {
                runtime[key.as_str()] = toml_edit::value(value.as_str());
            }
        }
    }

    Ok(doc.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_rewrite_version_and_hash() {
        let dir = tempdir().unwrap();
        let toml_path = dir.path().join("pkg-builder.toml");
        fs::write(
            &toml_path,
            r#"# Package config
[package]
name = "geth"
version = "1.14.0"
revision = "1"
homepage = "https://geth.ethereum.org"
spec = "geth.sss"

[source]
type = "tarball"
url = "https://github.com/ethereum/go-ethereum/archive/refs/tags/v1.14.0.tar.gz"
hash = "oldsha256"

[build]
distribution = "bookworm"
arch = "amd64"
"#,
        )
        .unwrap();

        let updates = TomlUpdates {
            version: Some("1.15.0".to_string()),
            revision: Some("1".to_string()),
            source_url: Some(
                "https://github.com/ethereum/go-ethereum/archive/refs/tags/v1.15.0.tar.gz"
                    .to_string(),
            ),
            source_hash: Some("newsha256".to_string()),
            source_tag: None,
            runtime_vars: None,
        };

        let result = rewrite_toml(&toml_path, &updates).unwrap();

        assert!(result.contains("# Package config"), "comments preserved");
        assert!(result.contains("version = \"1.15.0\""));
        assert!(result.contains("hash = \"newsha256\""));
        assert!(result.contains("v1.15.0.tar.gz"));
    }

    #[test]
    fn test_rewrite_git_tag() {
        let dir = tempdir().unwrap();
        let toml_path = dir.path().join("pkg-builder.toml");
        fs::write(
            &toml_path,
            r#"[package]
name = "prysm"
version = "5.0.0"
revision = "1"
homepage = "https://example.com"
spec = "prysm.sss"

[source]
type = "git"
url = "https://github.com/prysmaticlabs/prysm.git"
tag = "v5.0.0"
"#,
        )
        .unwrap();

        let updates = TomlUpdates {
            version: Some("5.1.0".to_string()),
            revision: Some("1".to_string()),
            source_url: None,
            source_hash: None,
            source_tag: Some("v5.1.0".to_string()),
            runtime_vars: None,
        };

        let result = rewrite_toml(&toml_path, &updates).unwrap();
        assert!(result.contains("version = \"5.1.0\""));
        assert!(result.contains("tag = \"v5.1.0\""));
    }

    #[test]
    fn test_rewrite_runtime_vars() {
        let dir = tempdir().unwrap();
        let toml_path = dir.path().join("pkg-builder.toml");
        fs::write(
            &toml_path,
            r#"[package]
name = "test"
version = "1.0.0"
revision = "1"
homepage = "https://example.com"
spec = "test.sss"

[source]
type = "virtual"

[runtime]
recipe = "go"
binary_url = "https://go.dev/dl/go1.22.2.linux-amd64.tar.gz"
binary_checksum = "oldchecksum"
"#,
        )
        .unwrap();

        let updates = TomlUpdates {
            version: None,
            revision: None,
            source_url: None,
            source_hash: None,
            source_tag: None,
            runtime_vars: Some(vec![
                (
                    "binary_url".to_string(),
                    "https://go.dev/dl/go1.23.0.linux-amd64.tar.gz".to_string(),
                ),
                ("binary_checksum".to_string(), "newchecksum".to_string()),
            ]),
        };

        let result = rewrite_toml(&toml_path, &updates).unwrap();
        assert!(result.contains("go1.23.0"));
        assert!(result.contains("newchecksum"));
    }
}
