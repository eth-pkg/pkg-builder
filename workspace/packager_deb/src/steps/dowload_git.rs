use std::{
    fs, io,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use filetime::FileTime;
use log::info;
use thiserror::Error;

use crate::{
    configs::pkg_config::SubModule,
    misc::build_pipeline::{BuildContext, BuildError, BuildStep},
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
    build_artifacts_dir: PathBuf,
    package_name: String,
    git_tag: String,
    git_url: String,
    submodules: Vec<SubModule>,
    tarball_path: PathBuf,
}

impl From<BuildContext> for DownloadGit {
    fn from(context: BuildContext) -> Self {
        Self {
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
    fn check_git_lfs() -> Result<(), DownloadGitError> {
        Command::new("which")
            .arg("git-lfs")
            .output()
            .map(|_| ())
            .map_err(|_| DownloadGitError::GitLfsNotInstalled)
    }

    fn get_error_message(output: &Output) -> String {
        String::from_utf8_lossy(&output.stderr).to_string()
    }

    fn run_git_command(
        args: &[&str],
        current_dir: Option<&Path>,
        error_type: impl FnOnce(String) -> DownloadGitError,
    ) -> Result<(), DownloadGitError> {
        let mut cmd = Command::new("git");

        if let Some(dir) = current_dir {
            cmd.current_dir(dir);
        }

        let output = cmd.args(args).output().map_err(DownloadGitError::IoError)?;

        if !output.status.success() {
            return Err(error_type(Self::get_error_message(&output)));
        }

        Ok(())
    }

    fn clone_repo(git_url: &str, tag_version: &str, path: &Path) -> Result<(), DownloadGitError> {
        let output = Command::new("git")
            .args(&[
                "clone",
                "--depth=1",
                "--branch",
                tag_version,
                git_url,
                &path.display().to_string(),
            ])
            .output()
            .map_err(DownloadGitError::IoError)?;

        if !output.status.success() {
            return Err(DownloadGitError::CheckoutTagFailed {
                tag: tag_version.to_string(),
                reason: Self::get_error_message(&output),
            });
        }

        Ok(())
    }

    fn init_submodules(path: &Path) -> Result<(), DownloadGitError> {
        Self::run_git_command(
            &["submodule", "init"],
            Some(&path),
            DownloadGitError::SubmoduleInitFailed,
        )?;

        Self::run_git_command(
            &["submodule", "update", "--depth=1", "--recursive"],
            Some(path),
            DownloadGitError::SubmoduleInitFailed,
        )
    }

    fn checkout_submodule_commits(
        submodule: &SubModule,
        base_path: &Path,
    ) -> Result<(), DownloadGitError> {
        let submodule_path = Path::new(base_path).join(&submodule.path);
        println!(
            "Checking out path: {:?} commit:{}",
            submodule_path, submodule.commit
        );

        let fetch_commit_output = Command::new("git")
            .current_dir(submodule_path.clone())
            .args(&["fetch", "origin", &submodule.commit.trim()])
            .output()
            .map_err(|err| DownloadGitError::SubmoduleCheckoutFailed(err.to_string()))?;

        if !fetch_commit_output.status.success() {
            return Err(DownloadGitError::SubmoduleCommitFailed {
                commit: submodule.commit.clone(),
                path: submodule.path.clone(),
                reason: Self::get_error_message(&fetch_commit_output),
            });
        }
        let output = Command::new("git")
            .current_dir(submodule_path)
            .args(&["checkout", &submodule.commit.trim()])
            .output()
            .map_err(|err| DownloadGitError::SubmoduleCheckoutFailed(err.to_string()))?;

        if !output.status.success() {
            return Err(DownloadGitError::SubmoduleCommitFailed {
                commit: submodule.commit.clone(),
                path: submodule.path.clone(),
                reason: Self::get_error_message(&output),
            });
        }

        Ok(())
    }

    pub fn clone_and_checkout_tag(
        git_url: &str,
        tag_version: &str,
        path: &Path,
        git_submodules: &[SubModule],
    ) -> Result<(), DownloadGitError> {
        Self::check_git_lfs()?;
        Self::clone_repo(git_url, tag_version, path)?;
        Self::init_submodules(path)?;
        Self::update_submodules(git_submodules, path)?;

        Ok(())
    }

    pub fn update_submodules(
        git_submodules: &[SubModule],
        current_dir: &Path,
    ) -> Result<(), DownloadGitError> {
        for submodule in git_submodules {
            Self::checkout_submodule_commits(submodule, current_dir)?;
        }

        Ok(())
    }

    fn create_tarball(&self) -> Result<(), DownloadGitError> {
        info!(
            "Creating tar from git repo from {}",
            self.build_artifacts_dir.join(&self.package_name).display()
        );

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
                &self.tarball_path.display().to_string(),
                &self.package_name,
            ])
            .current_dir(&self.build_artifacts_dir)
            .output()
            .map_err(DownloadGitError::IoError)?;

        if !output.status.success() {
            return Err(DownloadGitError::TarballCreationFailed(
                Self::get_error_message(&output),
            ));
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

                match file_type {
                    _ if file_type.is_dir() => {
                        stack.push(file_path.clone());
                        filetime::set_file_mtime(&file_path, timestamp)?;
                        filetime::set_file_atime(&file_path, timestamp)?;
                    }
                    _ if file_type.is_file() => {
                        filetime::set_file_mtime(&file_path, timestamp)?;
                        filetime::set_file_atime(&file_path, timestamp)?;
                    }
                    _ if file_type.is_symlink() => {
                        filetime::set_symlink_file_times(&file_path, timestamp, timestamp)?;
                    }
                    _ => (), // Skip other file types
                }
            }
        }

        Ok(())
    }

    fn prepare_build_directory(&self) -> Result<PathBuf, DownloadGitError> {
        let path = self.build_artifacts_dir.join(&self.package_name);

        if path.exists() {
            fs::remove_dir_all(&path)?;
        }

        fs::create_dir_all(&path)?;

        Ok(path)
    }
}

impl BuildStep for DownloadGit {
    fn step(&self) -> Result<(), BuildError> {
        let path = self.prepare_build_directory()?;

        Self::clone_and_checkout_tag(&self.git_url, &self.git_tag, &path, &self.submodules)?;

        fs::remove_dir_all(path.join(".git"))?;

        // Set consistent file timestamps for reproducibility: January 1, 2022
        let timestamp = FileTime::from_unix_time(1640995200, 0);
        Self::set_creation_time(path, timestamp)?;

        self.create_tarball()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::configs::pkg_config::{PackageType, PkgConfig};

    use super::*;
    use tempfile::tempdir;

    #[test]
    #[ignore = "Only run on CI"]
    fn test_clone_and_checkout_tag() {
        let url = "https://github.com/status-im/nimbus-eth2.git";
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let repo_path = temp_dir.path();
        let tag_version = "v24.3.0";
        let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
        let cargo_workspace_dir = Path::new(cargo_manifest_dir)
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        let toml_file =
            cargo_workspace_dir.join("examples/bookworm/git-package/nimbus/pkg-builder.toml");

        let str = fs::read_to_string(toml_file).expect("Cannot read example toml");
        let config: PkgConfig = toml::from_str(&str).expect("Cannot parse file.");
        match config.package_type {
            PackageType::Git(gitconfig) => {
                let result = DownloadGit::clone_and_checkout_tag(
                    url,
                    tag_version,
                    repo_path,
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
