use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use log::info;

use crate::context::Workspace;
use crate::PipelineError;

/// Set up the build directory (remove old, create fresh).
pub fn setup_build_dir(ws: &Workspace) -> Result<(), PipelineError> {
    if ws.build_artifacts_dir.exists() {
        info!("Removing old build artifacts: {:?}", ws.build_artifacts_dir);
        fs::remove_dir_all(&ws.build_artifacts_dir)?;
    }
    fs::create_dir_all(&ws.build_artifacts_dir)?;
    Ok(())
}

/// Extract the source tarball into build_files_dir.
pub fn extract_source(ws: &Workspace) -> Result<(), PipelineError> {
    tool::archive::extract_tar(&ws.tarball_path, &ws.build_files_dir)?;
    Ok(())
}

/// Apply patches to the extracted source.
pub fn patch_source(ws: &Workspace) -> Result<(), PipelineError> {
    patch_quilt(&ws.build_files_dir)?;
    patch_pc_dir(&ws.build_files_dir)?;
    patch_standards_version(&ws.build_files_dir, &ws.config.package.homepage)?;
    copy_src_dir(&ws.build_files_dir, &ws.src_dir)?;
    patch_rules_permission(&ws.build_files_dir)?;
    info!("Patching finished successfully!");
    Ok(())
}

/// Write the .sbuildrc configuration file.
pub fn setup_sbuildrc() -> Result<(), PipelineError> {
    let home_dir = dirs::home_dir().ok_or(PipelineError::Phase {
        phase: "setup_sbuildrc",
        message: "Could not determine home directory".to_string(),
    })?;

    let dest_path = home_dir.join(".sbuildrc");
    let content = SBUILDRC_TEMPLATE.replace("<HOME>", "/home/runner");
    fs::write(&dest_path, content)?;

    Ok(())
}

const SBUILDRC_TEMPLATE: &str = r#"##############################################################################
# PACKAGE BUILD RELATED (source-only-upload as default)
##############################################################################

$build_environment = {
'HOME' => '<HOME>'
};

$lintian_require_success = 1;
$piuparts_require_success = 1;
$autopkgtest_require_success = 1;

##############################################################################
# PERL MAGIC
##############################################################################
1;"#;

fn patch_quilt(build_files_dir: &Path) -> Result<(), PipelineError> {
    let format_path = build_files_dir.join("debian/source/format");

    if let Some(parent) = format_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    if !format_path.exists() {
        fs::write(&format_path, "3.0 (quilt)\n")?;
        info!("Quilt format file created at: {:?}", format_path);
    }
    Ok(())
}

fn patch_pc_dir(build_files_dir: &Path) -> Result<(), PipelineError> {
    let pc_dir = build_files_dir.join(".pc");
    fs::create_dir_all(&pc_dir)?;
    let mut file = fs::File::create(pc_dir.parent().unwrap().join(".pc/version"))?;
    writeln!(file, "2")?;
    Ok(())
}

fn patch_standards_version(build_files_dir: &Path, homepage: &str) -> Result<(), PipelineError> {
    let control_path = build_files_dir.join("debian/control");
    let input_file = fs::File::open(&control_path)?;
    let reader = BufReader::new(input_file);

    let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();
    let has_standards = lines.iter().any(|l| l.starts_with("Standards-Version"));

    if !has_standards {
        let mut insert_idx = 0;
        for (i, line) in lines.iter().enumerate() {
            if line.starts_with("Priority:") {
                insert_idx = i + 1;
                break;
            }
        }

        let mut updated = lines.clone();
        updated.insert(insert_idx, "Standards-Version: 4.5.1".to_string());
        updated.insert(insert_idx + 1, format!("Homepage: {}", homepage));

        let mut out = fs::File::create(&control_path)?;
        for line in updated {
            writeln!(out, "{}", line)?;
        }
        info!("Standards-Version added to debian/control");
    }
    Ok(())
}

fn copy_src_dir(build_files_dir: &Path, src_dir: &Path) -> Result<(), PipelineError> {
    if src_dir.exists() {
        copy_directory_recursive(src_dir, build_files_dir)?;
    }
    Ok(())
}

fn copy_directory_recursive(src: &Path, dest: &Path) -> Result<(), PipelineError> {
    if !dest.exists() {
        fs::create_dir_all(dest)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let entry_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if entry_path.is_dir() {
            copy_directory_recursive(&entry_path, &dest_path)?;
        } else {
            fs::copy(&entry_path, &dest_path)?;
        }
    }
    Ok(())
}

fn patch_rules_permission(build_files_dir: &Path) -> Result<(), PipelineError> {
    let rules_path = build_files_dir.join("debian/rules");
    info!("Adding executable permission for {:?}", rules_path);

    let mut permissions = fs::metadata(&rules_path)
        .map_err(|e| PipelineError::Phase {
            phase: "patch_rules",
            message: format!("Cannot read debian/rules metadata: {}", e),
        })?
        .permissions();

    permissions.set_mode(permissions.mode() | 0o111);
    fs::set_permissions(&rules_path, permissions)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::tempdir;

    #[test]
    fn test_patch_quilt_creates_format_file() {
        let dir = tempdir().unwrap();
        let build_dir = dir.path().to_path_buf();

        patch_quilt(&build_dir).unwrap();

        let format_path = build_dir.join("debian/source/format");
        assert!(format_path.exists());
        assert_eq!(fs::read_to_string(format_path).unwrap(), "3.0 (quilt)\n");
    }

    #[test]
    fn test_patch_quilt_idempotent() {
        let dir = tempdir().unwrap();
        let build_dir = dir.path().to_path_buf();

        // Create existing format file
        fs::create_dir_all(build_dir.join("debian/source")).unwrap();
        File::create(build_dir.join("debian/source/format")).unwrap();

        // Should not error
        patch_quilt(&build_dir).unwrap();
    }

    #[test]
    fn test_patch_pc_dir_creates_version_file() {
        let dir = tempdir().unwrap();
        let build_dir = dir.path().to_path_buf();

        patch_pc_dir(&build_dir).unwrap();

        let version_path = build_dir.join(".pc/version");
        assert!(version_path.exists());
        let content = fs::read_to_string(version_path).unwrap();
        assert!(content.trim() == "2");
    }

    #[test]
    fn test_patch_standards_version_adds_fields() {
        let dir = tempdir().unwrap();
        let build_dir = dir.path().to_path_buf();
        fs::create_dir_all(build_dir.join("debian")).unwrap();
        fs::write(
            build_dir.join("debian/control"),
            "Source: hello\nPriority: optional\nMaintainer: Test\n",
        )
        .unwrap();

        patch_standards_version(&build_dir, "https://example.com").unwrap();

        let content = fs::read_to_string(build_dir.join("debian/control")).unwrap();
        assert!(content.contains("Standards-Version: 4.5.1"));
        assert!(content.contains("Homepage: https://example.com"));
    }

    #[test]
    fn test_patch_standards_version_skips_if_present() {
        let dir = tempdir().unwrap();
        let build_dir = dir.path().to_path_buf();
        fs::create_dir_all(build_dir.join("debian")).unwrap();
        let original =
            "Source: hello\nStandards-Version: 4.6.0\nPriority: optional\n";
        fs::write(build_dir.join("debian/control"), original).unwrap();

        patch_standards_version(&build_dir, "https://example.com").unwrap();

        let content = fs::read_to_string(build_dir.join("debian/control")).unwrap();
        // Should keep the original Standards-Version, not add a new one
        assert!(content.contains("Standards-Version: 4.6.0"));
        assert!(!content.contains("Standards-Version: 4.5.1"));
    }

    #[test]
    fn test_patch_rules_permission_adds_exec() {
        let dir = tempdir().unwrap();
        let rules_path = dir.path().join("debian/rules");
        fs::create_dir_all(dir.path().join("debian")).unwrap();
        File::create(&rules_path).unwrap();

        patch_rules_permission(dir.path()).unwrap();

        let perms = fs::metadata(&rules_path).unwrap().permissions();
        assert_ne!(perms.mode() & 0o111, 0, "Should have exec permission");
    }

    #[test]
    fn test_patch_rules_permission_nonexistent() {
        let result = patch_rules_permission(Path::new("/nonexistent/dir"));
        assert!(result.is_err());
    }

    #[test]
    fn test_copy_src_dir_copies_files() {
        let dir = tempdir().unwrap();
        let src_dir = dir.path().join("src");
        let dest_dir = dir.path().join("dest");
        fs::create_dir_all(&src_dir).unwrap();
        fs::create_dir_all(&dest_dir).unwrap();
        fs::write(src_dir.join("test.txt"), "hello").unwrap();

        copy_src_dir(&dest_dir, &src_dir).unwrap();

        assert!(dest_dir.join("test.txt").exists());
        assert_eq!(
            fs::read_to_string(dest_dir.join("test.txt")).unwrap(),
            "hello"
        );
    }

    #[test]
    fn test_copy_src_dir_skips_nonexistent() {
        let dir = tempdir().unwrap();
        // Should not error when src doesn't exist
        copy_src_dir(dir.path(), &dir.path().join("nonexistent")).unwrap();
    }

    #[test]
    fn test_copy_directory_recursive_nested() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src");
        let dest = dir.path().join("dest");
        fs::create_dir_all(src.join("subdir")).unwrap();
        fs::write(src.join("top.txt"), "top").unwrap();
        fs::write(src.join("subdir/nested.txt"), "nested").unwrap();

        copy_directory_recursive(&src, &dest).unwrap();

        assert!(dest.join("top.txt").exists());
        assert!(dest.join("subdir/nested.txt").exists());
        assert_eq!(
            fs::read_to_string(dest.join("subdir/nested.txt")).unwrap(),
            "nested"
        );
    }

    #[test]
    fn test_setup_build_dir_creates_dir() {
        let dir = tempdir().unwrap();
        let artifacts_dir = dir.path().join("build-artifacts");

        // Create a fake workspace
        let cfg = config::PkgConfig {
            package: config::package::PackageFields {
                spec: "test.sss".into(),
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                revision: "1".to_string(),
                homepage: "https://example.com".to_string(),
            },
            source: config::source::SourceKind::Virtual,
            runtime: None,
            build_env: config::build_env::BuildEnv {
                distribution: config::build_env::Distribution::bookworm(),
                arch: config::build_env::Architecture::Amd64,
                pkg_builder_version: "0.3.1".to_string(),

                chroot_dir: dir.path().join("cache"),
                workdir: dir.path().to_path_buf(),
                testing: config::build_env::TestingConfig {
                    run_lintian: false,
                    run_piuparts: false,
                    run_autopkgtest: false,
                },
                tool_versions: config::build_env::ToolVersions {
                    sbuild: "0.85.6".to_string(),
                    lintian: "2.116.3".to_string(),
                    piuparts: "1.1.7".to_string(),
                    autopkgtest: "5.28".to_string(),
                },
                snapshot_date: None,
                snapshot_security_date: None,
            },
            config_root: dir.path().to_path_buf(),
        };

        let ws = crate::context::Workspace::new(std::sync::Arc::new(cfg)).unwrap();

        // Create and then re-create
        fs::create_dir_all(&ws.build_artifacts_dir).unwrap();
        fs::write(ws.build_artifacts_dir.join("old-file.txt"), "old").unwrap();

        setup_build_dir(&ws).unwrap();

        assert!(ws.build_artifacts_dir.exists());
        assert!(!ws.build_artifacts_dir.join("old-file.txt").exists());
    }
}
