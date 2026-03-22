mod acquire;
mod chroot_setup;
mod debian;
mod env;
mod extract;
mod paths;
mod preflight;
pub mod runtime;
mod sbuild;
mod sbuild_conf;

use std::fs;

use config::PkgConfig;
use log::info;
use thiserror::Error;

use paths::BuildPaths;

#[derive(Debug, Error)]
pub enum ExecutorError {
    #[error("Runtime profile not found: {0}")]
    ProfileNotFound(String),
    #[error("Missing required tools: {}", .0.join(", "))]
    MissingTools(Vec<String>),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Download failed for {0}: {1}")]
    Download(String, reqwest::Error),
    #[error("Download failed for {0}: HTTP {1}")]
    DownloadStatus(String, u16),
    #[error("Hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },
    #[error("Command '{command}' failed with exit code {code}")]
    CommandFailed { command: String, code: i32 },
    #[error("Debcrafter error: {0}")]
    Debcrafter(String),
    #[error("Missing maintainer: set 'maintainer' in .sss spec or DEBEMAIL environment variable")]
    MissingMaintainer,
}

/// Run the full build pipeline.
pub fn build(config: &PkgConfig, resume: bool) -> Result<(), ExecutorError> {
    let paths = BuildPaths::from_config(config);
    preflight::check_required_tools(config)?;

    if !resume {
        if paths.out_dir.exists() {
            info!("Cleaning previous build artifacts...");
            fs::remove_dir_all(&paths.out_dir)?;
        }
    }

    fs::create_dir_all(&paths.out_dir)?;

    if !resume || !paths.src_tarball.exists() {
        info!("Step 1/4: Acquiring source...");
        acquire::acquire(config, &paths)?;
    } else {
        info!("Step 1/4: Acquiring source... (skipped, tarball exists)");
    }

    if !resume || !paths.src_dir.exists() {
        info!("Step 2/4: Extracting source...");
        extract::extract(&paths)?;
    } else {
        info!("Step 2/4: Extracting source... (skipped, src_dir exists)");
    }

    if !resume || !paths.src_dir.join("debian/source/format").exists() {
        info!("Step 3/4: Generating Debian packaging...");
        debian::generate_debian(config, &paths)?;
    } else {
        info!("Step 3/4: Generating Debian packaging... (skipped)");
    }

    info!("Step 4/4: Running sbuild...");
    sbuild::run_sbuild(config, &paths)?;

    Ok(())
}

/// Create the sbuild chroot environment.
pub fn create_env(config: &PkgConfig) -> Result<(), ExecutorError> {
    let paths = BuildPaths::from_config(config);
    env::create_env(config, &paths)
}

/// Remove the chroot tarball.
pub fn clean_env(config: &PkgConfig) -> Result<(), ExecutorError> {
    let paths = BuildPaths::from_config(config);
    env::clean_env(&paths)
}

/// Remove build artifacts (out_dir).
pub fn clean(config: &PkgConfig) -> Result<(), ExecutorError> {
    let paths = BuildPaths::from_config(config);
    if paths.out_dir.exists() {
        info!("Removing {:?}", paths.out_dir);
        fs::remove_dir_all(&paths.out_dir)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::runtime::*;

    #[test]
    fn test_builtin_runtime_perl_exist() {
        assert!(load_runtime_perl("go", std::path::Path::new("/nonexistent")).is_ok());
        assert!(load_runtime_perl("c", std::path::Path::new("/nonexistent")).is_ok());
        assert!(load_runtime_perl("nonexistent", std::path::Path::new("/nonexistent")).is_err());
    }
}
