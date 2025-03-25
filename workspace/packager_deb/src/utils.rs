use std::{
    fs::{create_dir_all, remove_dir_all, remove_file},
    path::{Path, PathBuf},
    process::Command,
};

use crate::sbuild::SbuildError;
use log::{info, warn};
use sha1::{Digest, Sha1};
use types::version::Version;

pub fn check_tool_version(tool: &str, expected_version: &Version) -> Result<(), SbuildError> {
    let (cmd, args) = match tool {
        "lintian" | "piuparts" => (tool, vec!["--version"]),
        "autopkgtest" => ("apt", vec!["list", "--installed", "autopkgtest"]),
        _ => {
            return Err(SbuildError::GenericError(format!(
                "Unsupported tool: {}",
                tool
            )))
        }
    };

    let output = Command::new(cmd).args(args).output()?;
    if !output.status.success() {
        return Err(SbuildError::GenericError(format!(
            "Failed to check {} version",
            tool
        )));
    }
    let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
    let actual_version = Version::try_from(stdout_str)?;

    info!(
        "versions: expected:{} actual:{}",
        expected_version, actual_version
    );
    warn_compare_versions(expected_version, &actual_version, tool)?;
    Ok(())
}

fn warn_compare_versions(
    expected: &Version,
    actual: &Version,
    tool: &str,
) -> Result<(), SbuildError> {
    match expected.cmp(&actual) {
        std::cmp::Ordering::Less => warn!(
            "Using newer {} version ({}) than expected ({})",
            tool, actual, expected
        ),
        std::cmp::Ordering::Greater => warn!(
            "Using older {} version ({}) than expected ({})",
            tool, actual, expected
        ),
        std::cmp::Ordering::Equal => info!("{} versions match ({})", tool, expected),
    }

    Ok(())
}

pub fn calculate_sha1(data: &[u8]) -> Result<String, SbuildError> {
    let mut hasher = Sha1::new();
    hasher.update(data);
    Ok(hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect())
}

pub fn ensure_parent_dir<T: AsRef<Path>>(path: T) -> Result<(), SbuildError> {
    let parent = Path::new(path.as_ref())
        .parent()
        .ok_or(SbuildError::GenericError(format!(
            "Invalid path: {:?}",
            path.as_ref()
        )))?;
    create_dir_all(parent)?;
    Ok(())
}
pub fn remove_file_or_directory(path: &PathBuf, is_dir: bool) -> Result<(), SbuildError> {
    if Path::new(path).exists() {
        if is_dir {
            remove_dir_all(path)?;
        } else {
            remove_file(path)?;
        }
    }
    Ok(())
}
