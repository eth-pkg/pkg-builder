pub mod changelog;
pub mod files;
pub mod github;
pub mod patches;
pub mod toml_rewrite;

use changelog::{find_maintainer, new_changelog_entry, prepend_changelog_entry};
use config::PkgConfig;
use files::{apply_substitutions, FileSubstitutions};
use github::{fetch_latest_release, fetch_tag_commit, parse_github_repo};
use log::{info, warn};
use patches::test_patches;
use std::path::{Path, PathBuf};
use thiserror::Error;
use toml_rewrite::{rewrite_toml, TomlUpdates};

#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("Config error: {0}")]
    Config(#[from] config::ConfigError),
    #[error("{0}")]
    Other(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl From<Box<dyn std::error::Error>> for UpdateError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        UpdateError::Other(e.to_string())
    }
}

/// Configuration for the update command.
pub struct UpdateConfig {
    pub version: Option<String>,
    pub revision: String,
    pub changelog_msg: Option<String>,
    pub github_repo: Option<String>,
    pub in_place: bool,
    pub output: Option<String>,
    pub hash: Option<String>,
    pub git_commit: Option<String>,
    pub skip_download: bool,
    pub update_runtime: bool,
    pub skip_patch_check: bool,
}

/// Summary of what was updated.
pub struct UpdateSummary {
    pub old_version: String,
    pub new_version: String,
    pub new_revision: String,
    pub source_hash: Option<String>,
    pub git_commit: Option<String>,
    pub runtime_updated: bool,
    pub files_modified: Vec<String>,
    pub patch_summary: Option<String>,
    pub output_dir: PathBuf,
}

/// Run the update process.
pub fn run_update(
    config_path: &Path,
    update_cfg: &UpdateConfig,
) -> Result<UpdateSummary, UpdateError> {
    // Stage 1: Load current state
    info!("Loading current config...");
    let config = PkgConfig::load(config_path)?;
    let config_root = config.config_root.clone();
    let old_version = config.package.version.clone();

    let (old_source_url, old_hash, old_tag) = match &config.source {
        config::source::SourceKind::Tarball { url, hash } => {
            (Some(url.clone()), hash.clone(), None)
        }
        config::source::SourceKind::Git { url, tag, .. } => {
            (Some(url.clone()), None, Some(tag.clone()))
        }
        config::source::SourceKind::Virtual => (None, None, None),
    };

    // Stage 2: Determine new version
    let (new_version, new_tag_name) = resolve_new_version(
        &update_cfg.version,
        &update_cfg.github_repo,
        old_source_url.as_deref(),
    )?;
    info!("Updating {} -> {}", old_version, new_version);

    // Stage 3: Resolve new source
    let (new_source_url, new_hash, new_commit, _temp_dir) = resolve_new_source(
        &config,
        &old_version,
        &new_version,
        &new_tag_name,
        old_source_url.as_deref(),
        update_cfg,
    )?;

    // Resolve runtime updates
    let runtime_vars = if update_cfg.update_runtime {
        resolve_runtime_update(&config)?
    } else {
        None
    };

    // Determine output directory
    let output_dir = determine_output_dir(&config_root, update_cfg)?;

    // If --output, copy the entire package directory first
    if update_cfg.output.is_some() && output_dir != config_root {
        info!("Copying package directory to {}...", output_dir.display());
        copy_dir_recursive(&config_root, &output_dir)?;
    }

    // Stage 4: Rewrite pkg-builder.toml
    info!("Updating pkg-builder.toml...");
    let toml_path = output_dir.join("pkg-builder.toml");
    let updates = TomlUpdates {
        version: Some(new_version.clone()),
        revision: Some(update_cfg.revision.clone()),
        source_url: new_source_url.clone(),
        source_hash: new_hash.clone(),
        source_tag: new_tag_name.clone(),
        runtime_vars,
    };
    let new_toml = rewrite_toml(&toml_path, &updates)?;
    std::fs::write(&toml_path, &new_toml)?;

    // Stage 5: Update changelog
    info!("Updating changelog...");
    let changelog_path = output_dir.join("src").join("debian").join("changelog");
    let distribution = config.build_env.distribution.as_short();
    let maintainer =
        find_maintainer(&output_dir).unwrap_or_else(|| "Unknown <unknown@unknown>".to_string());
    let default_msg = format!("New upstream version {}", new_version);
    let changelog_msg = update_cfg.changelog_msg.as_deref().unwrap_or(&default_msg);
    let entry = new_changelog_entry(
        &config.package.name,
        &new_version,
        &update_cfg.revision,
        distribution,
        &maintainer,
        changelog_msg,
    );
    prepend_changelog_entry(&changelog_path, &entry)?;

    // Stage 6: Test patches
    let patch_summary =
        if !update_cfg.skip_patch_check && !update_cfg.skip_download && update_cfg.hash.is_none() {
            let patches_dir = output_dir.join("src").join("debian").join("patches");
            if patches_dir.exists() {
                // We need the extracted source for patch testing
                // For now, if we have a temp dir with source, use it
                // Otherwise skip
                if let Some(ref _temp) = _temp_dir {
                    let interactive = atty_is_interactive();
                    match test_patches(_temp.path(), &patches_dir, &output_dir, interactive) {
                        Ok(result) => {
                            let summary = result.summary();
                            info!("Patch test: {}", summary);
                            Some(summary)
                        }
                        Err(e) => {
                            warn!("Patch testing failed: {}", e);
                            Some(format!("error: {}", e))
                        }
                    }
                } else {
                    info!("No extracted source available for patch testing");
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

    // Stage 7: Update other files
    info!("Updating version strings in package files...");
    let subs = FileSubstitutions {
        old_version: old_version.clone(),
        new_version: new_version.clone(),
        old_hash: old_hash.clone(),
        new_hash: new_hash.clone(),
        old_tag: old_tag.clone(),
        new_tag: new_tag_name.clone(),
        old_commit: None,
        new_commit: new_commit.clone(),
    };
    let files_modified = apply_substitutions(&output_dir, &subs)?;

    Ok(UpdateSummary {
        old_version,
        new_version: new_version.clone(),
        new_revision: update_cfg.revision.clone(),
        source_hash: new_hash,
        git_commit: new_commit,
        runtime_updated: update_cfg.update_runtime,
        files_modified,
        patch_summary,
        output_dir,
    })
}

/// Stage 2: Resolve the new version, either from CLI flag or GitHub API.
fn resolve_new_version(
    version_flag: &Option<String>,
    github_repo_flag: &Option<String>,
    source_url: Option<&str>,
) -> Result<(String, Option<String>), UpdateError> {
    if let Some(version) = version_flag {
        // If explicit version given, try to construct a tag name
        let tag = if version.starts_with('v') {
            Some(version.clone())
        } else {
            Some(format!("v{}", version))
        };
        let clean_version = version.strip_prefix('v').unwrap_or(version).to_string();
        return Ok((clean_version, tag));
    }

    // Auto-detect from GitHub
    let repo = github_repo_flag
        .clone()
        .or_else(|| source_url.and_then(parse_github_repo))
        .ok_or_else(|| {
            UpdateError::Other(
                "Cannot determine GitHub repo. Use --version or --github-repo.".to_string(),
            )
        })?;

    let (tag_name, version) = fetch_latest_release(&repo)?;
    Ok((version, Some(tag_name)))
}

type SourceResolution = (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<tempfile::TempDir>,
);

/// Stage 3: Resolve new source URL, hash, and commit.
fn resolve_new_source(
    config: &PkgConfig,
    old_version: &str,
    new_version: &str,
    new_tag: &Option<String>,
    old_source_url: Option<&str>,
    update_cfg: &UpdateConfig,
) -> Result<SourceResolution, UpdateError> {
    // If hash was provided directly, skip download
    if let Some(ref hash) = update_cfg.hash {
        let new_url = old_source_url.map(|url| url.replace(old_version, new_version));
        return Ok((
            new_url,
            Some(hash.clone()),
            update_cfg.git_commit.clone(),
            None,
        ));
    }

    // If git commit was provided directly
    if let Some(ref commit) = update_cfg.git_commit {
        return Ok((None, None, Some(commit.clone()), None));
    }

    match &config.source {
        config::source::SourceKind::Tarball { url, .. } => {
            let new_url = url.replace(old_version, new_version);

            if update_cfg.skip_download {
                return Ok((Some(new_url), None, None, None));
            }

            // Download and hash
            let temp_dir = tempfile::tempdir()?;
            match pkg_builder_init::download_and_hash_tarball(&new_url, temp_dir.path(), None) {
                Ok(result) => {
                    info!("Downloaded and hashed: {}", result.hash);
                    // Extract for patch testing
                    let extract_dir = tempfile::tempdir()?;
                    if let Some(ref local_path) = result.local_path {
                        extract_tarball(local_path, extract_dir.path())?;
                    }
                    Ok((Some(new_url), Some(result.hash), None, Some(extract_dir)))
                }
                Err(e) => {
                    warn!("Failed to download {}: {}", new_url, e);
                    Ok((Some(new_url), None, None, None))
                }
            }
        }
        config::source::SourceKind::Git { url, .. } => {
            let default_tag = format!("v{}", new_version);
            let new_tag_str = new_tag.as_deref().unwrap_or(&default_tag);

            // Try to get commit SHA from GitHub
            let commit = if !update_cfg.skip_download {
                if let Some(repo) = update_cfg
                    .github_repo
                    .clone()
                    .or_else(|| parse_github_repo(url))
                {
                    match fetch_tag_commit(&repo, new_tag_str) {
                        Ok(sha) => {
                            info!("Commit for tag {}: {}", new_tag_str, sha);
                            Some(sha)
                        }
                        Err(e) => {
                            warn!("Could not fetch commit for tag {}: {}", new_tag_str, e);
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            };

            Ok((None, None, commit, None))
        }
        config::source::SourceKind::Virtual => Ok((None, None, None, None)),
    }
}

/// Resolve runtime update by fetching latest versions.
fn resolve_runtime_update(
    config: &PkgConfig,
) -> Result<Option<Vec<(String, String)>>, UpdateError> {
    if let Some(ref rt) = config.runtime {
        if let Some(runtime) = pkg_builder_init::Runtime::from_str(&rt.recipe) {
            match runtime.resolve_version(
                // Try to extract current version from runtime vars
                &extract_runtime_version(&rt.vars).unwrap_or_default(),
            ) {
                Ok(vars) => {
                    info!("Runtime updated");
                    return Ok(Some(vars));
                }
                Err(e) => {
                    warn!("Could not auto-update runtime: {}", e);
                }
            }
        }
    }
    Ok(None)
}

/// Try to extract a version from runtime variables (e.g., from binary_url).
fn extract_runtime_version(
    vars: &std::collections::BTreeMap<String, toml::Value>,
) -> Option<String> {
    // Try to find a version in the binary_url
    if let Some(url) = vars.get("binary_url").and_then(|v| v.as_str()) {
        // Look for version-like patterns in the URL
        let re = regex::Regex::new(r"(\d+\.\d+\.\d+)").ok()?;
        if let Some(cap) = re.captures(url) {
            return Some(cap[1].to_string());
        }
    }
    None
}

fn determine_output_dir(
    config_root: &Path,
    update_cfg: &UpdateConfig,
) -> Result<PathBuf, UpdateError> {
    if let Some(ref output) = update_cfg.output {
        Ok(PathBuf::from(output))
    } else if update_cfg.in_place {
        Ok(config_root.to_path_buf())
    } else {
        // Default: in-place (non-interactive mode) or prompt
        if atty_is_interactive() {
            let choices = &["Update in-place", "Write to new directory"];
            let selection = dialoguer::Select::new()
                .with_prompt("How should the update be written?")
                .items(choices)
                .default(0)
                .interact()
                .map_err(|e| UpdateError::Other(e.to_string()))?;

            match selection {
                0 => Ok(config_root.to_path_buf()),
                1 => {
                    let dir: String = dialoguer::Input::new()
                        .with_prompt("Output directory")
                        .interact_text()
                        .map_err(|e| UpdateError::Other(e.to_string()))?;
                    Ok(PathBuf::from(dir))
                }
                _ => unreachable!(),
            }
        } else {
            // Non-interactive: default to in-place
            Ok(config_root.to_path_buf())
        }
    }
}

/// Check if stdin is a TTY (interactive mode).
fn atty_is_interactive() -> bool {
    use std::io::IsTerminal;
    std::io::stdin().is_terminal()
}

/// Extract a tarball to a destination directory.
fn extract_tarball(tarball: &Path, dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let status = std::process::Command::new("tar")
        .args([
            "xf",
            &tarball.display().to_string(),
            "--strip-components=1",
            "-C",
            &dest.display().to_string(),
        ])
        .status()?;
    if !status.success() {
        return Err(format!("tar extraction failed for {}", tarball.display()).into());
    }
    Ok(())
}

/// Recursively copy a directory.
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), UpdateError> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write_test_config(dir: &Path) {
        fs::write(
            dir.join("pkg-builder.toml"),
            r#"[package]
name = "test-pkg"
version = "1.0.0"
revision = "1"
homepage = "https://example.com"
spec = "test-pkg.sss"

[source]
type = "tarball"
url = "https://github.com/example/test-pkg/archive/refs/tags/v1.0.0.tar.gz"
hash = "abc123def456"

[build]
distribution = "bookworm"
arch = "amd64"
workdir = "/tmp/test"

[tools]
pkg_builder = "0.3.1"
debcrafter = "8189263"
sbuild = "0.85.6"
lintian = "2.116.3"
piuparts = "1.1.7"
autopkgtest = "5.28"
"#,
        )
        .unwrap();

        fs::write(
            dir.join("test-pkg.sss"),
            "name: test-pkg\nmaintainer: \"Test User <test@example.com>\"\n",
        )
        .unwrap();

        // Create a file with version strings
        fs::create_dir_all(dir.join("src").join("debian")).unwrap();
        fs::write(
            dir.join("src").join("debian").join("rules"),
            "#!/usr/bin/make -f\nVERSION=1.0.0\nHASH=abc123def456\n",
        )
        .unwrap();
    }

    #[test]
    fn test_update_in_place_with_skip_download() {
        let dir = tempdir().unwrap();
        write_test_config(dir.path());

        let update_cfg = UpdateConfig {
            version: Some("2.0.0".to_string()),
            revision: "1".to_string(),
            changelog_msg: Some("Test update".to_string()),
            github_repo: None,
            in_place: true,
            output: None,
            hash: Some("newsha256hash".to_string()),
            git_commit: None,
            skip_download: false,
            update_runtime: false,
            skip_patch_check: true,
        };

        let summary = run_update(dir.path(), &update_cfg).unwrap();
        assert_eq!(summary.old_version, "1.0.0");
        assert_eq!(summary.new_version, "2.0.0");

        // Check TOML was updated
        let toml_content = fs::read_to_string(dir.path().join("pkg-builder.toml")).unwrap();
        assert!(toml_content.contains("version = \"2.0.0\""));
        assert!(toml_content.contains("hash = \"newsha256hash\""));
        assert!(toml_content.contains("v2.0.0.tar.gz"));

        // Check other files were updated
        let rules = fs::read_to_string(dir.path().join("src/debian/rules")).unwrap();
        assert!(rules.contains("VERSION=2.0.0"));
        assert!(rules.contains("HASH=newsha256hash"));

        // Check changelog was created
        let changelog = fs::read_to_string(dir.path().join("src/debian/changelog")).unwrap();
        assert!(changelog.contains("test-pkg (2.0.0-1)"));
        assert!(changelog.contains("Test update"));
    }

    #[test]
    fn test_update_to_output_dir() {
        let dir = tempdir().unwrap();
        write_test_config(dir.path());

        let output_dir = tempdir().unwrap();

        let update_cfg = UpdateConfig {
            version: Some("3.0.0".to_string()),
            revision: "1".to_string(),
            changelog_msg: None,
            github_repo: None,
            in_place: false,
            output: Some(output_dir.path().display().to_string()),
            hash: Some("outputhash".to_string()),
            git_commit: None,
            skip_download: false,
            update_runtime: false,
            skip_patch_check: true,
        };

        let summary = run_update(dir.path(), &update_cfg).unwrap();
        assert_eq!(summary.new_version, "3.0.0");

        // Original should be unchanged
        let original_toml = fs::read_to_string(dir.path().join("pkg-builder.toml")).unwrap();
        assert!(original_toml.contains("version = \"1.0.0\""));

        // Output should have updates
        let output_toml = fs::read_to_string(output_dir.path().join("pkg-builder.toml")).unwrap();
        assert!(output_toml.contains("version = \"3.0.0\""));
    }

    #[test]
    fn test_resolve_new_version_explicit() {
        let (version, tag) = resolve_new_version(&Some("1.15.0".to_string()), &None, None).unwrap();
        assert_eq!(version, "1.15.0");
        assert_eq!(tag, Some("v1.15.0".to_string()));
    }

    #[test]
    fn test_resolve_new_version_with_v_prefix() {
        let (version, tag) =
            resolve_new_version(&Some("v1.15.0".to_string()), &None, None).unwrap();
        assert_eq!(version, "1.15.0");
        assert_eq!(tag, Some("v1.15.0".to_string()));
    }

    #[test]
    fn test_resolve_new_version_no_source() {
        let result = resolve_new_version(&None, &None, None);
        assert!(result.is_err());
    }
}
