use std::fs;
use std::process::Command;

use config::PkgConfig;
use log::info;

use crate::paths::BuildPaths;
use crate::ExecutorError;

/// Step 1: Acquire the source tarball.
pub fn acquire(config: &PkgConfig, paths: &BuildPaths) -> Result<(), ExecutorError> {
    match &config.source {
        config::source::SourceKind::Tarball { url, hash } => {
            acquire_tarball(url, hash.as_deref(), paths)?;
        }
        config::source::SourceKind::Git {
            url,
            tag,
            submodules,
        } => {
            acquire_git(url, tag, submodules, paths)?;
        }
        config::source::SourceKind::Virtual => {
            acquire_virtual(paths)?;
        }
    }
    Ok(())
}

fn acquire_tarball(
    url: &str,
    hash: Option<&str>,
    paths: &BuildPaths,
) -> Result<(), ExecutorError> {
    if url.starts_with("http://") || url.starts_with("https://") {
        info!("Downloading {}", url);
        let response = reqwest::blocking::get(url)
            .map_err(|e| ExecutorError::Download(url.to_string(), e))?;
        if !response.status().is_success() {
            return Err(ExecutorError::DownloadStatus(
                url.to_string(),
                response.status().as_u16(),
            ));
        }
        let bytes = response
            .bytes()
            .map_err(|e| ExecutorError::Download(url.to_string(), e))?;
        fs::write(&paths.src_tarball, &bytes)?;
    } else {
        // Local file — already resolved to absolute by config loader
        let local_path = std::path::PathBuf::from(url);
        info!("Copying local tarball from {:?}", local_path);
        fs::copy(&local_path, &paths.src_tarball)?;
    }

    if let Some(expected) = hash {
        verify_hash(&paths.src_tarball, expected)?;
    }

    Ok(())
}

fn acquire_git(
    url: &str,
    tag: &str,
    submodules: &[config::source::Submodule],
    paths: &BuildPaths,
) -> Result<(), ExecutorError> {
    info!("Cloning {} (tag: {})", url, tag);

    // Clone
    run_cmd(
        Command::new("git")
            .args(["clone", "--depth=1", "--branch", tag, url])
            .arg(&paths.src_dir),
        "git clone",
    )?;

    // Init submodules
    run_cmd(
        Command::new("git")
            .args(["submodule", "update", "--init", "--recursive"])
            .current_dir(&paths.src_dir),
        "git submodule update",
    )?;

    // Checkout specific submodule commits
    for sub in submodules {
        let sub_dir = paths.src_dir.join(&sub.path);
        let commit = sub.commit.trim();
        run_cmd(
            Command::new("git")
                .args(["fetch", "origin", commit])
                .current_dir(&sub_dir),
            &format!("git fetch for submodule {}", sub.path),
        )?;
        run_cmd(
            Command::new("git")
                .args(["checkout", commit])
                .current_dir(&sub_dir),
            &format!("git checkout for submodule {}", sub.path),
        )?;
    }

    // Remove .git directories
    let _ = fs::remove_dir_all(paths.src_dir.join(".git"));
    run_cmd(
        Command::new("find")
            .arg(&paths.src_dir)
            .args(["-name", ".git", "-exec", "rm", "-rf", "{}", "+"]),
        "remove .git dirs",
    )?;

    // Create reproducible tarball
    let src_dir_name = paths
        .src_dir
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();
    run_cmd(
        Command::new("tar")
            .args([
                "--sort=name",
                "--owner=0",
                "--group=0",
                "--numeric-owner",
                "--pax-option=exthdr.name=%d/PaxHeaders/%f,delete=atime,delete=ctime",
                "-czf",
            ])
            .arg(&paths.src_tarball)
            .args(["-C", paths.out_dir.to_str().unwrap(), src_dir_name]),
        "tar create",
    )?;

    Ok(())
}

fn acquire_virtual(paths: &BuildPaths) -> Result<(), ExecutorError> {
    fs::create_dir_all(&paths.src_dir)?;
    run_cmd(
        Command::new("tar")
            .args(["czvf"])
            .arg(&paths.src_tarball)
            .args(["--files-from", "/dev/null"]),
        "create empty tarball",
    )?;
    Ok(())
}

fn verify_hash(file: &std::path::Path, expected: &str) -> Result<(), ExecutorError> {
    use sha2::Digest;
    let bytes = fs::read(file)?;
    let actual = if expected.len() == 128 {
        format!("{:x}", sha2::Sha512::digest(&bytes))
    } else {
        format!("{:x}", sha2::Sha256::digest(&bytes))
    };
    if actual != expected {
        return Err(ExecutorError::HashMismatch {
            expected: expected.to_string(),
            actual,
        });
    }
    Ok(())
}

fn run_cmd(cmd: &mut Command, description: &str) -> Result<(), ExecutorError> {
    let status = cmd.status()?;
    if !status.success() {
        return Err(ExecutorError::CommandFailed {
            command: description.to_string(),
            code: status.code().unwrap_or(1),
        });
    }
    Ok(())
}
