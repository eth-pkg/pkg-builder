use std::fs;
use std::process::Command;

use crate::paths::BuildPaths;
use crate::ExecutorError;

/// Step 2: Extract the source tarball into src_dir.
pub fn extract(paths: &BuildPaths) -> Result<(), ExecutorError> {
    fs::create_dir_all(&paths.src_dir)?;
    let status = Command::new("tar")
        .args(["-C"])
        .arg(&paths.src_dir)
        .args(["-xf"])
        .arg(&paths.src_tarball)
        .status()?;
    if !status.success() {
        return Err(ExecutorError::CommandFailed {
            command: "tar extract".to_string(),
            code: status.code().unwrap_or(1),
        });
    }
    Ok(())
}
