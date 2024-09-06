use git2::Repository;
use log::info;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Command not found: {0}")]
    CommandNotFound(#[from] io::Error),

    #[error("Failed to execute command: {0}")]
    CommandFailed(CommandError),

    #[error("Failed to clone: {0}")]
    GitClone(#[from] git2::Error),
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("{0}")]
    StringError(String),
    #[error("{0}")]
    IOError(io::Error),
}

impl From<String> for CommandError {
    fn from(err: String) -> Self {
        CommandError::StringError(err)
    }
}

impl From<io::Error> for CommandError {
    fn from(err: io::Error) -> Self {
        CommandError::IOError(err)
    }
}

// TODO use from crates.io
pub fn check_if_dpkg_parsechangelog_installed() -> Result<(), Error> {
    let mut cmd = Command::new("which");
    cmd.arg("dpkg-parsechangelog");

    handle_failure(
        &mut cmd,
        "dpkg-parsechangelog is not installed, please install it.".to_string(),
    )?;
    Ok(())
}

pub fn check_if_installed(debcrafter_version: &String) -> eyre::Result<()> {
    match Command::new("which")
        .arg(format!("debcrafter_{}", debcrafter_version))
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                eyre::bail!(
                    "Command failed with exit code {:?}\nstdout: {}\nstderr: {}",
                    output.status.code(),
                    stdout,
                    stderr
                );
            }
        }
        Err(err) => eyre::bail!("Failed to execute 'which' command: {}", err),
    }
}

// pub fn check_version_compatibility(debcrafter_version: &str) -> Result<(), String> {
//     let output = Command::new("debcrafter")
//         .arg("--version")
//         .output()
//         .map_err(|_| "Failed to execute debcrafter --version")?;

//     let output_str = String::from_utf8_lossy(&output.stdout);

//     if !output.status.success() && output_str.contains(debcrafter_version) {
//         return Err("debcrafter does not have the required version".to_string());
//     }
//     Ok(())
// }

pub fn create_debian_dir(specification_file: &str, target_dir: &str) -> Result<(), Error> {
    let debcrafter_dir = tempdir().expect("Failed to create temporary directory");

    let spec_file_path = fs::canonicalize(PathBuf::from(specification_file)).map_err(|_| {
        Error::CommandFailed(format!("{} spec_file doesn't exist", specification_file).into())
    })?;
    if !spec_file_path.exists() {
        return Err(Error::CommandFailed(
            format!("{} spec_file doesn't exist", specification_file).into(),
        ));
    }
    let spec_dir = spec_file_path.parent().unwrap();
    let spec_file_name = spec_file_path.file_name().unwrap();
    info!("Spec directory: {:?}", spec_dir.to_str().unwrap());
    info!("Spec file: {:?}", spec_file_name);
    info!("Debcrafter directory: {:?}", debcrafter_dir);
    let mut cmd = Command::new("debcrafter");
    cmd.arg(spec_file_name)
        .current_dir(spec_dir)
        .arg(debcrafter_dir.path());

    handle_failure(&mut cmd, "Debcrafter error".to_string())?;

    if let Some(first_directory) = get_first_directory(debcrafter_dir.path()) {
        let tmp_debian_dir = first_directory.join("debian");
        let dest_dir = Path::new(target_dir).join("debian");
        copy_dir_contents_recursive(&tmp_debian_dir, &dest_dir)
            .map_err(|err| Error::CommandFailed(err.into()))?;
    } else {
        return Err(Error::CommandFailed(
            "Unable to create debian dir.".to_string().into(),
        ));
    }
    Ok(())
}

fn copy_dir_contents_recursive(src_dir: &Path, dest_dir: &Path) -> io::Result<()> {
    info!(
        "Copying directory: {:?} to {:?}",
        src_dir.display(),
        dest_dir.display()
    );

    if src_dir.is_dir() {
        if !dest_dir.exists() {
            fs::create_dir_all(dest_dir)?;
        }

        for entry in fs::read_dir(src_dir)? {
            let entry = entry?;
            let src_path = entry.path();
            let dest_path = dest_dir.join(entry.file_name());

            if src_path.is_dir() {
                copy_dir_contents_recursive(&src_path, &dest_path)?;
            } else {
                fs::copy(&src_path, &dest_path)?;
            }
        }
    }

    Ok(())
}
fn handle_failure(cmd: &mut Command, error: String) -> Result<(), Error> {
    let output = cmd
        .output()
        .map_err(|_| Error::CommandFailed(error.clone().into()))?;

    if !output.status.success() {
        //let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(Error::CommandFailed(error.into()));
    }
    Ok(())
}
fn get_first_directory(dir: &Path) -> Option<PathBuf> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir).ok()? {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                return Some(path);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // fn test_debcrafter_installed() {
    //     let result = check_if_installed();
    //     assert!(result.is_err());
    //     assert_eq!(result.unwrap_err(), "debcrafter is not installed");
    // }

    // #[test]
    // fn test_debcrafter_version_incompatibility() {
    //     let result = check_version_compatibility("1.0.0");
    //     assert!(result.is_err());
    //     assert_eq!(
    //         result.unwrap_err(),
    //         "debcrafter does not have the required version"
    //     );
    // }
}
