use std::{
    env, fs::{self, create_dir_all, remove_dir_all, remove_file}, path::{Path, PathBuf}
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

pub fn expand_path(dir: &PathBuf, dir_to_expand: Option<&PathBuf>) -> PathBuf {
    if dir.to_string_lossy().starts_with('~') {
        let dir_str = dir.to_string_lossy();
        PathBuf::from(shellexpand::tilde(&dir_str).to_string())
    } else if dir.is_absolute() {
        dir.clone()
    } else {
        let parent_dir = match dir_to_expand {
            None => env::current_dir().unwrap(),
            Some(path) => path.clone(),
        };

        let path = parent_dir.join(dir);
        fs::canonicalize(path.clone()).unwrap_or(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_path_expands_tilde_correctly() {
        let tilde = PathBuf::from("~");
        let result = expand_path(&tilde, None);
        assert_ne!(result, tilde);
        assert!(!result.display().to_string().contains('~'));
    }

    #[test]
    fn expand_path_handles_absolute_paths() {
        let absolute_path = PathBuf::from("/absolute/path");
        let result = expand_path(&absolute_path, None);
        assert_eq!(result, absolute_path);
    }

    #[test]
    fn expand_path_expands_relative_paths_with_parent() {
        let file = PathBuf::from("somefile");
        let mut tmp = PathBuf::from("/tmp");
        let result = expand_path(&file, Some(&tmp));
        tmp.push(file);
        assert_eq!(result, tmp);
    }

    #[test]
    fn expand_path_expands_relative_paths_without_parent() {
        let file = PathBuf::from("somefile");

        let result = expand_path(&file, None);
        assert!(result.display().to_string().starts_with('/'));
    }
}