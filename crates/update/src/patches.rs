use dialoguer::Select;
use log::{info, warn};
use std::path::Path;
use std::process::Command;

/// Result of testing patches against a new source.
pub struct PatchTestResult {
    pub applied_cleanly: usize,
    pub fixed_interactively: usize,
    pub skipped: usize,
    pub aborted: bool,
}

impl PatchTestResult {
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        if self.applied_cleanly > 0 {
            parts.push(format!("{} applied cleanly", self.applied_cleanly));
        }
        if self.fixed_interactively > 0 {
            parts.push(format!("{} fixed interactively", self.fixed_interactively));
        }
        if self.skipped > 0 {
            parts.push(format!("{} skipped", self.skipped));
        }
        if self.aborted {
            parts.push("aborted".to_string());
        }
        if parts.is_empty() {
            "no patches to test".to_string()
        } else {
            parts.join(", ")
        }
    }
}

/// Test patches against extracted source in a temp directory.
///
/// - `source_dir`: extracted upstream source (temp dir)
/// - `patches_dir`: package's `src/debian/patches/` directory
/// - `interactive`: whether to prompt on failure (false = just report)
///
/// Returns the patch test result and optionally updates patches in-place if fixed.
pub fn test_patches(
    source_dir: &Path,
    patches_dir: &Path,
    package_dir: &Path,
    interactive: bool,
) -> Result<PatchTestResult, Box<dyn std::error::Error>> {
    let series_path = patches_dir.join("series");
    if !series_path.exists() {
        info!("No patches/series file found, skipping patch test");
        return Ok(PatchTestResult {
            applied_cleanly: 0,
            fixed_interactively: 0,
            skipped: 0,
            aborted: false,
        });
    }

    let series_content = std::fs::read_to_string(&series_path)?;
    let patches: Vec<&str> = series_content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();

    if patches.is_empty() {
        info!("No patches in series file");
        return Ok(PatchTestResult {
            applied_cleanly: 0,
            fixed_interactively: 0,
            skipped: 0,
            aborted: false,
        });
    }

    info!("Testing {} patches against new source...", patches.len());

    // Copy patches into the source dir for quilt
    let dest_patches = source_dir.join("debian").join("patches");
    copy_dir_recursive(patches_dir, &dest_patches)?;

    // Also copy .pc directory if it exists
    let pc_src = package_dir.join("src").join(".pc");
    if pc_src.exists() {
        let pc_dest = source_dir.join(".pc");
        copy_dir_recursive(&pc_src, &pc_dest)?;
    }

    let mut result = PatchTestResult {
        applied_cleanly: 0,
        fixed_interactively: 0,
        skipped: 0,
        aborted: false,
    };

    let quilt_patches = dest_patches.display().to_string();

    for patch_name in &patches {
        info!("Applying patch: {}", patch_name);

        let output = Command::new("quilt")
            .arg("push")
            .env("QUILT_PATCHES", &quilt_patches)
            .current_dir(source_dir)
            .output()?;

        if output.status.success() {
            result.applied_cleanly += 1;
            info!("  OK: {}", patch_name);
            continue;
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        warn!(
            "Patch {} failed to apply:\n{}\n{}",
            patch_name, stdout, stderr
        );

        if !interactive {
            warn!("Non-interactive mode: skipping failed patch {}", patch_name);
            result.skipped += 1;
            continue;
        }

        let choices = &[
            "Drop to shell (fix manually, then exit)",
            "Force apply + drop to shell (resolve .rej files)",
            "Skip this patch (remove from series)",
            "Abort patch testing",
        ];

        let selection = Select::new()
            .with_prompt(format!(
                "Patch '{}' failed. What would you like to do?",
                patch_name
            ))
            .items(choices)
            .default(0)
            .interact()?;

        match selection {
            0 => {
                // Drop to shell
                if drop_to_shell(source_dir, &quilt_patches)? {
                    copy_refreshed_patches(&dest_patches, patches_dir)?;
                    result.fixed_interactively += 1;
                } else {
                    result.skipped += 1;
                }
            }
            1 => {
                // Force apply + drop to shell
                let _ = Command::new("quilt")
                    .args(["push", "-f"])
                    .env("QUILT_PATCHES", &quilt_patches)
                    .current_dir(source_dir)
                    .status();

                if drop_to_shell(source_dir, &quilt_patches)? {
                    copy_refreshed_patches(&dest_patches, patches_dir)?;
                    result.fixed_interactively += 1;
                } else {
                    result.skipped += 1;
                }
            }
            2 => {
                // Skip patch
                info!("Skipping patch: {}", patch_name);
                remove_from_series(&dest_patches.join("series"), patch_name)?;
                // Also update the original series file
                remove_from_series(&patches_dir.join("series"), patch_name)?;
                result.skipped += 1;
            }
            3 => {
                // Abort
                warn!("Aborting patch testing");
                result.aborted = true;
                break;
            }
            _ => unreachable!(),
        }
    }

    if !result.aborted {
        // Copy any updated patches back
        copy_refreshed_patches(&dest_patches, patches_dir)?;
    }

    Ok(result)
}

/// Spawn a shell in the source directory with QUILT_PATCHES set.
/// Returns true if the user exited successfully (exit 0).
fn drop_to_shell(
    source_dir: &Path,
    quilt_patches: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    info!(
        "Dropping to shell in {}. Run 'quilt refresh' after fixing, then 'exit 0' to continue.",
        source_dir.display()
    );

    let status = Command::new(&shell)
        .env("QUILT_PATCHES", quilt_patches)
        .current_dir(source_dir)
        .status()?;

    Ok(status.success())
}

/// Remove a patch name from a series file.
fn remove_from_series(
    series_path: &Path,
    patch_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(series_path)?;
    let new_content: String = content
        .lines()
        .filter(|line| line.trim() != patch_name)
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(series_path, format!("{}\n", new_content))?;
    Ok(())
}

/// Copy refreshed patches from the source's patch dir back to the package's patch dir.
fn copy_refreshed_patches(
    source_patches: &Path,
    dest_patches: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if !source_patches.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(source_patches)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let dest = dest_patches.join(entry.file_name());
            std::fs::copy(&path, &dest)?;
        }
    }
    Ok(())
}

/// Recursively copy a directory.
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), Box<dyn std::error::Error>> {
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

    #[test]
    fn test_no_patches_dir() {
        let source = tempdir().unwrap();
        let patches = tempdir().unwrap();
        let pkg = tempdir().unwrap();

        let result = test_patches(
            source.path(),
            &patches.path().join("nonexistent"),
            pkg.path(),
            false,
        )
        .unwrap();
        assert_eq!(result.applied_cleanly, 0);
        assert_eq!(result.skipped, 0);
    }

    #[test]
    fn test_empty_series() {
        let source = tempdir().unwrap();
        let patches = tempdir().unwrap();
        let pkg = tempdir().unwrap();

        fs::write(patches.path().join("series"), "# empty\n").unwrap();

        let result = test_patches(source.path(), patches.path(), pkg.path(), false).unwrap();
        assert_eq!(result.applied_cleanly, 0);
    }

    #[test]
    fn test_remove_from_series() {
        let dir = tempdir().unwrap();
        let series = dir.path().join("series");
        fs::write(&series, "patch1.diff\npatch2.diff\npatch3.diff\n").unwrap();

        remove_from_series(&series, "patch2.diff").unwrap();

        let content = fs::read_to_string(&series).unwrap();
        assert!(!content.contains("patch2.diff"));
        assert!(content.contains("patch1.diff"));
        assert!(content.contains("patch3.diff"));
    }

    #[test]
    fn test_patch_result_summary() {
        let result = PatchTestResult {
            applied_cleanly: 3,
            fixed_interactively: 1,
            skipped: 0,
            aborted: false,
        };
        assert_eq!(result.summary(), "3 applied cleanly, 1 fixed interactively");

        let empty = PatchTestResult {
            applied_cleanly: 0,
            fixed_interactively: 0,
            skipped: 0,
            aborted: false,
        };
        assert_eq!(empty.summary(), "no patches to test");
    }
}
