use std::{
    fs::{create_dir_all, remove_dir_all, remove_file},
    path::{Path, PathBuf},
};

use crate::sbuild::SbuildError;
use sha1::{Digest, Sha1};

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
