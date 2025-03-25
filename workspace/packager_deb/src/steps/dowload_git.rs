use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

use filetime::FileTime;
use log::info;
use thiserror::Error;

use crate::{
    build_pipeline::{BuildContext, BuildError, BuildStep},
    pkg_config::SubModule,
};

#[derive(Error, Debug)]
pub enum DownloadGitError {
    #[error("git-lfs is not installed, please install it!")]
    GitLfsNotInstalled,

    #[error("Failed to checkout tag {tag}: {reason}")]
    CheckoutTagFailed { tag: String, reason: String },

    #[error("Failed to initialize submodules: {0}")]
    SubmoduleInitFailed(String),

    #[error("Failed to checkout submodule: {0}")]
    SubmoduleCheckoutFailed(String),

    #[error("Failed to checkout commit {commit} for submodule {path}: {reason}")]
    SubmoduleCommitFailed {
        commit: String,
        path: String,
        reason: String,
    },

    #[error("Failed to create tarball: {0}")]
    TarballCreationFailed(String),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
}

#[derive(Default)]
pub struct DownloadGit {
    build_artifacts_dir: String,
    package_name: String,
    git_tag: String,
    git_url: String,
    submodules: Vec<SubModule>,
    tarball_path: String,
}

impl From<BuildContext> for DownloadGit {
    fn from(context: BuildContext) -> Self {
        DownloadGit {
            build_artifacts_dir: context.build_artifacts_dir.clone(),
            package_name: context.package_name.clone(),
            git_tag: context.git_tag.clone(),
            git_url: context.git_url.clone(),
            submodules: context.submodules.clone(),
            tarball_path: context.tarball_path.clone(),
        }
    }
}

impl DownloadGit {
    pub fn clone_and_checkout_tag(
        git_url: &str,
        tag_version: &str,
        path: &str,
        git_submodules: &Vec<SubModule>,
    ) -> Result<(), DownloadGitError> {
        match Command::new("which").arg("git-lfs").output() {
            Ok(_) => Ok(()),
            Err(_) => Err(DownloadGitError::GitLfsNotInstalled),
        }?;

        let output = Command::new("git")
            .args(&[
                "clone",
                "--depth",
                "1",
                "--branch",
                tag_version,
                git_url,
                path,
            ])
            .output()
            .map_err(|e| DownloadGitError::IoError(e))?;

        if !output.status.success() {
            return Err(DownloadGitError::CheckoutTagFailed {
                tag: tag_version.to_string(),
                reason: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        // Initialize submodules
        let output = Command::new("git")
            .current_dir(path)
            .args(&["submodule", "update", "--init", "--recursive"])
            .output()
            .map_err(|e| DownloadGitError::IoError(e))?;

        if !output.status.success() {
            return Err(DownloadGitError::SubmoduleInitFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Self::update_submodules(git_submodules, path)?;

        Ok(())
    }

    pub fn update_submodules(
        git_submodules: &Vec<SubModule>,
        current_dir: &str,
    ) -> Result<(), DownloadGitError> {
        // DO not use git2, it has very little git supported functionality
        // Initialize all submodules if they are not already initialized
        // Update submodules to specific commits
        for submodule in git_submodules.clone() {
            let output = Command::new("git")
                .current_dir(Path::new(current_dir).join(submodule.path.clone()))
                .args(&["checkout", &submodule.commit.clone()])
                .output()
                .map_err(|err| DownloadGitError::SubmoduleCheckoutFailed(err.to_string()))?;

            if !output.status.success() {
                return Err(DownloadGitError::SubmoduleCommitFailed {
                    commit: submodule.commit,
                    path: submodule.path,
                    reason: String::from_utf8_lossy(&output.stderr).to_string(),
                });
            }
        }

        Ok(())
    }

    fn set_creation_time<P: AsRef<Path>>(dir_path: P, timestamp: FileTime) -> io::Result<()> {
        filetime::set_file_mtime(&dir_path, timestamp)?;
        filetime::set_file_atime(&dir_path, timestamp)?;

        let mut stack = vec![PathBuf::from(dir_path.as_ref())];

        while let Some(current) = stack.pop() {
            for entry in fs::read_dir(&current)? {
                let entry = entry?;
                let file_type = entry.file_type()?;
                let file_path = entry.path();

                if file_type.is_dir() {
                    stack.push(file_path.clone()); // Push directory onto stack for processing
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
}

impl BuildStep for DownloadGit {
    fn step(&self) -> Result<(), BuildError> {
        let path = Path::new(&self.build_artifacts_dir).join(&self.package_name);
        if path.exists() {
            fs::remove_dir_all(path.clone()).map_err(DownloadGitError::IoError)?;
        }
        fs::create_dir_all(&path.clone()).map_err(DownloadGitError::IoError)?;
        //let path = Path::new("/tmp/nimbus");
        Self::clone_and_checkout_tag(
            &self.git_url,
            &self.git_tag,
            path.clone().to_str().unwrap(),
            &self.submodules,
        )?;
        // remove .git directory, no need to package it
        fs::remove_dir_all(path.join(".git")).map_err(DownloadGitError::IoError)?;

        // // Back in the path for reproducibility: January 1, 2022
        let timestamp = FileTime::from_unix_time(1640995200, 0);
        Self::set_creation_time(path.clone(), timestamp).map_err(DownloadGitError::IoError)?;

        info!("Creating tar from git repo from {}", path.display());
        let output = Command::new("tar")
            .args(&[
                "--sort=name",
                "--owner=0",
                "--group=0",
                "--numeric-owner",
                // does not work
                // "--mtime='2019-01-01 00:00'",
                "--pax-option=exthdr.name=%d/PaxHeaders/%f,delete=atime,delete=ctime",
                "-czf",
                &self.tarball_path,
                &self.package_name,
            ])
            .current_dir(&self.build_artifacts_dir)
            .output()
            .map_err(DownloadGitError::IoError)?;

        if !output.status.success() {
            return Err(DownloadGitError::TarballCreationFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            )
            .into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::pkg_config::{PackageType, PkgConfig};

    use super::*;
    use tempfile::tempdir;

    #[test]
    #[ignore = "Only run on CI"]
    fn test_clone_and_checkout_tag() {
        let url = "https://github.com/status-im/nimbus-eth2.git";
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let repo_path = temp_dir.path();
        let repo_path_str = repo_path.to_str().unwrap();
        let tag_version = "v24.3.0";
        let str = fs::read_to_string("examples/bookworm/git-package/nimbus/pkg-builder.toml")
            .expect("File does not exist");
        let config: PkgConfig = toml::from_str(&str).expect("Cannot parse file.");
        match config.package_type {
            PackageType::Git(gitconfig) => {
                let result = DownloadGit::clone_and_checkout_tag(
                    url,
                    tag_version,
                    repo_path_str,
                    &gitconfig.submodules,
                );
                assert!(
                    result.is_ok(),
                    "Failed to clone and checkout tag: {:?}",
                    result
                );
            }
            _ => panic!("Wrong type of file."),
        }

        fs::remove_dir_all(temp_dir).unwrap();
    }
}
