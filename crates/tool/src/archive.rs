use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use filetime::FileTime;
use log::info;

use crate::ToolError;
use config::source::Submodule;

/// Download a file via wget.
pub fn wget(url: &str, output: &Path) -> Result<(), ToolError> {
    info!("Downloading: {} to {:?}", url, output);
    let status = Command::new("wget")
        .arg("-q")
        .arg("-O")
        .arg(output)
        .arg(url)
        .status()
        .map_err(|e| ToolError::NotFound {
            tool: "wget".to_string(),
            source: e,
        })?;

    if !status.success() {
        return Err(ToolError::Other {
            tool: "wget".to_string(),
            message: format!("Download failed: {}", url),
        });
    }
    Ok(())
}

/// Extract a tar.gz archive, auto-detecting strip-components.
pub fn extract_tar(tarball: &Path, dest: &Path) -> Result<(), ToolError> {
    fs::create_dir_all(dest)?;

    let strip = detect_strip_components(tarball)?;

    let mut args = vec![
        "zxvf".to_string(),
        tarball.display().to_string(),
        "-C".to_string(),
        dest.display().to_string(),
    ];

    if strip > 0 {
        args.push(format!("--strip-components={}", strip));
    }

    info!("Extracting: tar {}", args.join(" "));
    let output = Command::new("tar").args(&args).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ToolError::Other {
            tool: "tar".to_string(),
            message: format!("Extraction error: {}", stderr),
        });
    }

    Ok(())
}

/// Create an empty tar.gz (for virtual packages).
pub fn create_empty_tar(tarball: &Path, working_dir: &Path) -> Result<(), ToolError> {
    info!("Creating empty .tar.gz for virtual package");
    let output = Command::new("tar")
        .args([
            "--sort=name",
            "--owner=0",
            "--group=0",
            "--numeric-owner",
            "--mtime=2022-01-01 00:00:00",
            "--pax-option=exthdr.name=%d/PaxHeaders/%f,delete=atime,delete=ctime",
            "-czf",
            &tarball.display().to_string(),
            "--files-from",
            "/dev/null",
        ])
        .current_dir(working_dir)
        .output()?;

    if !output.status.success() {
        return Err(ToolError::Other {
            tool: "tar".to_string(),
            message: "Failed to create empty tarball".to_string(),
        });
    }
    Ok(())
}

/// Clone a git repo at a specific tag, initialize submodules, and create a tarball.
pub fn clone_and_archive(
    url: &str,
    tag: &str,
    submodules: &[Submodule],
    tarball_path: &Path,
    package_name: &str,
    build_artifacts_dir: &Path,
) -> Result<(), ToolError> {
    check_git_lfs()?;

    let clone_path = build_artifacts_dir.join(package_name);
    if clone_path.exists() {
        fs::remove_dir_all(&clone_path)?;
    }
    fs::create_dir_all(&clone_path)?;

    // Clone
    git_clone(url, tag, &clone_path)?;

    // Init submodules
    git_submodule_init(&clone_path)?;

    // Checkout specific submodule commits
    for submodule in submodules {
        checkout_submodule_commit(submodule, &clone_path)?;
    }

    // Remove .git dir
    fs::remove_dir_all(clone_path.join(".git"))?;

    // Set consistent timestamps for reproducibility (2022-01-01)
    let timestamp = FileTime::from_unix_time(1640995200, 0);
    set_timestamps_recursive(&clone_path, timestamp)?;

    // Create tarball
    info!(
        "Creating tar from git repo from {}",
        clone_path.display()
    );
    let output = Command::new("tar")
        .args([
            "--sort=name",
            "--owner=0",
            "--group=0",
            "--numeric-owner",
            "--pax-option=exthdr.name=%d/PaxHeaders/%f,delete=atime,delete=ctime",
            "-czf",
            &tarball_path.display().to_string(),
            package_name,
        ])
        .current_dir(build_artifacts_dir)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ToolError::Other {
            tool: "tar".to_string(),
            message: format!("Failed to create tarball: {}", stderr),
        });
    }

    Ok(())
}

fn check_git_lfs() -> Result<(), ToolError> {
    Command::new("which")
        .arg("git-lfs")
        .output()
        .map(|_| ())
        .map_err(|_| ToolError::Other {
            tool: "git-lfs".to_string(),
            message: "git-lfs is not installed, please install it!".to_string(),
        })
}

fn git_clone(url: &str, tag: &str, path: &Path) -> Result<(), ToolError> {
    let output = Command::new("git")
        .args(["clone", "--depth=1", "--branch", tag, url, &path.display().to_string()])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ToolError::Other {
            tool: "git".to_string(),
            message: format!("Failed to checkout tag {}: {}", tag, stderr),
        });
    }
    Ok(())
}

fn git_submodule_init(path: &Path) -> Result<(), ToolError> {
    let output = Command::new("git")
        .args(["submodule", "init"])
        .current_dir(path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ToolError::Other {
            tool: "git".to_string(),
            message: format!("Failed to init submodules: {}", stderr),
        });
    }

    let output = Command::new("git")
        .args(["submodule", "update", "--init", "--recursive"])
        .current_dir(path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ToolError::Other {
            tool: "git".to_string(),
            message: format!("Failed to update submodules: {}", stderr),
        });
    }
    Ok(())
}

fn checkout_submodule_commit(submodule: &Submodule, base_path: &Path) -> Result<(), ToolError> {
    let submodule_path = base_path.join(&submodule.path);

    let output = Command::new("git")
        .current_dir(&submodule_path)
        .args(["fetch", "origin", submodule.commit.trim()])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ToolError::Other {
            tool: "git".to_string(),
            message: format!(
                "Failed to fetch commit {} for submodule {}: {}",
                submodule.commit, submodule.path, stderr
            ),
        });
    }

    let output = Command::new("git")
        .current_dir(&submodule_path)
        .args(["checkout", submodule.commit.trim()])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ToolError::Other {
            tool: "git".to_string(),
            message: format!(
                "Failed to checkout commit {} for submodule {}: {}",
                submodule.commit, submodule.path, stderr
            ),
        });
    }
    Ok(())
}

fn detect_strip_components(tarball: &Path) -> Result<usize, ToolError> {
    let output = Command::new("tar")
        .args(["--list", "-z", "-f", &tarball.display().to_string()])
        .output()?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = output_str.lines().filter(|l| !l.ends_with('/')).collect();
    let common_prefix = longest_common_prefix(&lines);
    let components = common_prefix
        .split('/')
        .filter(|x| !x.is_empty())
        .count();
    Ok(components)
}

fn longest_common_prefix(strings: &[&str]) -> String {
    if strings.is_empty() {
        return String::new();
    }
    if strings.len() == 1 {
        let mut path_buf = PathBuf::from(strings[0]);
        path_buf.pop();
        return path_buf.to_string_lossy().to_string();
    }
    let first = &strings[0];
    let mut prefix = String::new();
    'outer: for (i, c) in first.char_indices() {
        for s in &strings[1..] {
            if let Some(next_char) = s.chars().nth(i) {
                if next_char != c {
                    break 'outer;
                }
            } else {
                break 'outer;
            }
        }
        prefix.push(c);
    }
    prefix
}

fn set_timestamps_recursive(dir_path: &Path, timestamp: FileTime) -> io::Result<()> {
    filetime::set_file_mtime(dir_path, timestamp)?;
    filetime::set_file_atime(dir_path, timestamp)?;

    let mut stack = vec![PathBuf::from(dir_path)];

    while let Some(current) = stack.pop() {
        for entry in fs::read_dir(&current)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let file_path = entry.path();

            if file_type.is_dir() {
                stack.push(file_path.clone());
                filetime::set_file_mtime(&file_path, timestamp)?;
                filetime::set_file_atime(&file_path, timestamp)?;
            } else if file_type.is_file() {
                filetime::set_file_mtime(&file_path, timestamp)?;
                filetime::set_file_atime(&file_path, timestamp)?;
            } else if file_type.is_symlink() {
                filetime::set_symlink_file_times(&file_path, timestamp, timestamp)?;
            }
        }
    }

    Ok(())
}
