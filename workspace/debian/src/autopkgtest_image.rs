use crate::execute::ExecuteError;

use super::execute::{execute_command_with_sudo, Execute};
/// Provides functionality for building and managing Autopkgtest VM images
///
/// This module contains structures and implementations for creating
/// virtual machine images compatible with autopkgtest for different
/// Linux distributions.
use log::info;
use std::path::{Path, PathBuf};
use thiserror::Error;
use types::distribution::Distribution;

/// Custom error type for autopkgtest image building operations
#[derive(Error, Debug)]
pub enum AutopkgtestImageError {
    #[error("Distribution not specified")]
    MissingDistribution,

    #[error("Image path not specified")]
    MissingImagePath,

    #[error("Unsupported or invalid distribution codename: {0}")]
    UnsupportedDistribution(String),

    #[error("Failed to execute command: {0}")]
    CommandExecutionError(#[from] ExecuteError),

    #[error("Work directory path error: {0}")]
    PathError(String),
}

// Type alias for Result with our custom error type
type Result<T> = std::result::Result<T, AutopkgtestImageError>;

trait BuildCommandProvider {
    fn get_command(&self) -> &'static str;
    fn get_formatted_codename(&self) -> String;
}
impl BuildCommandProvider for Distribution {
    /// Returns the appropriate command for building an image for this distribution
    ///
    /// # Returns
    /// * `&'static str` - The command to use for image creation
    fn get_command(&self) -> &'static str {
        match self {
            Distribution::Debian(_) => "autopkgtest-build-qemu",
            Distribution::Ubuntu(_) => "autopkgtest-buildvm-ubuntu-cloud",
        }
    }

    /// Returns the codename formatted as an argument for the build command
    ///
    /// # Returns
    /// * `String` - The formatted codename argument
    fn get_formatted_codename(&self) -> String {
        match self {
            Distribution::Debian(_) => self.as_short().to_string(),
            Distribution::Ubuntu(_) => format!("--release={}", self.as_short()),
        }
    }
}

/// Builder for creating Autopkgtest VM images
///
/// Provides a fluent interface for configuring and building VM images
/// for different distributions, architectures, and repositories.
#[derive(Debug, Clone, Default)]
pub struct AutopkgtestImageBuilder {
    /// The target Linux distribution
    distribution: Option<Distribution>,
    /// Path where the image will be created
    image_path: Option<PathBuf>,
    /// Directory to use for temporary files during build
    work_dir: Option<PathBuf>,
    /// Repository mirror URL to use for package installation
    mirror: Option<String>,
    /// Target architecture for the VM image
    arch: Option<String>,
}

impl AutopkgtestImageBuilder {
    /// Creates a new empty builder instance
    ///
    /// # Returns
    /// * `Self` - A new AutopkgtestImageBuilder
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the distribution codename for the image
    ///
    /// # Arguments
    /// * `codename` - The distribution codename (e.g., "bookworm", "noble")
    ///
    /// # Returns
    /// * `Result<Self>` - Modified builder or error if codename is unsupported
    pub fn codename(mut self, codename: &Distribution) -> Result<Self> {
        // Assuming Distribution::from_codename has been modified to return our Result type
        // or we're mapping the error here
        self.distribution = Some(codename.clone());
        Ok(self)
    }

    /// Sets the image output path based on cache directory, codename and architecture
    ///
    /// # Arguments
    /// * `cache_dir` - Directory where the image will be stored
    /// * `codename` - Distribution codename
    /// * `arch` - Target architecture
    ///
    /// # Returns
    /// * `Self` - Modified builder
    pub fn image_path(mut self, cache_dir: &str, codename: &Distribution, arch: &str) -> Self {
        let image_name = format!("autopkgtest-{}-{}.img", codename.as_short(), arch);
        let cache_dir = shellexpand::tilde(cache_dir).to_string();
        let image_path = Path::new(&cache_dir).join(&image_name);
        self.image_path = Some(image_path.clone());
        self.work_dir = Some(image_path.parent().unwrap_or(Path::new("")).to_path_buf());
        self
    }

    /// Sets the package repository mirror URL
    ///
    /// # Arguments
    /// * `repo_url` - The URL of the package repository mirror
    ///
    /// # Returns
    /// * `Self` - Modified builder
    pub fn mirror(mut self, repo_url: &str) -> Self {
        self.mirror = Some(repo_url.to_string());
        self
    }

    /// Sets the target architecture for the VM image
    ///
    /// # Arguments
    /// * `arch` - Architecture name (e.g., "amd64", "arm64")
    ///
    /// # Returns
    /// * `Self` - Modified builder
    pub fn arch(mut self, arch: &str) -> Self {
        self.arch = Some(arch.to_string());
        self
    }

    /// Returns the configured image path if set
    ///
    /// # Returns
    /// * `Option<&PathBuf>` - The image path or None if not set
    pub fn get_image_path(&self) -> Option<&PathBuf> {
        self.image_path.as_ref()
    }

    /// Builds the command-line arguments for the image creation command
    ///
    /// # Returns
    /// * `Result<Vec<String>>` - The list of arguments or an error if configuration is incomplete
    fn build_args(&self) -> Result<Vec<String>> {
        let mut args = Vec::new();

        if let Some(dist) = &self.distribution {
            args.push(dist.get_formatted_codename());
        } else {
            return Err(AutopkgtestImageError::MissingDistribution);
        }

        if let Some(mirror) = &self.mirror {
            args.push(format!("--mirror={}", mirror));
        }

        if let Some(arch) = &self.arch {
            args.push(format!("--arch={}", arch));
        }

        if let Some(Distribution::Ubuntu(_)) = &self.distribution {
            args.push("-v".to_string());
        }

        if let Some(path) = &self.image_path {
            if let Some(Distribution::Debian(_)) = &self.distribution {
                args.push(path.to_string_lossy().to_string());
            }
        } else {
            return Err(AutopkgtestImageError::MissingImagePath);
        }

        Ok(args)
    }
}

/// Implementation of the Execute trait for AutopkgtestImageBuilder
///
/// Allows the builder to be executed to create the VM image.
impl Execute for AutopkgtestImageBuilder {
    type Error = AutopkgtestImageError;
    /// Executes the VM image creation process
    ///
    /// # Returns
    /// * `Result<()>` - Success or an error if the build fails
    fn execute(&self) -> Result<()> {
        let cmd = self
            .distribution
            .as_ref()
            .ok_or(AutopkgtestImageError::MissingDistribution)?
            .get_command();

        let args = self.build_args()?;

        info!("Running: sudo -S {} {}", cmd, args.join(" "));

        execute_command_with_sudo(cmd, args, self.work_dir.as_deref())?;

        Ok(())
    }
}

/// Unit tests for AutopkgtestImageBuilder functionality
#[cfg(test)]
mod tests {
    use super::*;

    /// Tests creation of Distribution from various codenames
    #[test]
    fn test_distribution_from_codename() {
        assert!(matches!(
            Distribution::from_codename("bookworm").unwrap(),
            Distribution::Debian(_)
        ));

        assert!(matches!(
            Distribution::from_codename("noble").unwrap(),
            Distribution::Ubuntu(_)
        ));

        assert!(Distribution::from_codename("unsupported").is_err());
    }

    /// Tests generation of command-line arguments
    #[test]
    fn test_build_args() {
        let builder = AutopkgtestImageBuilder::new()
            .codename(&Distribution::bookworm())
            .unwrap()
            .image_path("/tmp", &Distribution::bookworm(), "amd64")
            .arch("amd64")
            .mirror("http://example.com/debian");

        let args = builder.build_args().unwrap();
        assert!(args.contains(&"bookworm".to_string()));
        assert!(args
            .iter()
            .any(|arg| arg.contains("/tmp/autopkgtest-bookworm-amd64.img")));
        assert!(args.contains(&"--arch=amd64".to_string()));
        assert!(args.contains(&"--mirror=http://example.com/debian".to_string()));
    }

    #[test]
    fn test_noble_build_args() {
        let builder = AutopkgtestImageBuilder::new()
            .codename(&Distribution::noble())
            .unwrap()
            .image_path("/tmp", &Distribution::noble(), "amd64")
            .arch("amd64")
            .mirror("http://example.com/ubuntu");

        let args = builder.build_args().unwrap();
        assert!(args.contains(&"--release=noble".to_string()));
        assert!(!args.contains(&"--release=noble numbat".to_string()));
        assert!(args
            .iter()
            .all(|arg| !arg.contains("/tmp/autopkgtest-noble-amd64.img")));
        assert!(args.contains(&"--arch=amd64".to_string()));
        assert!(args.contains(&"--mirror=http://example.com/ubuntu".to_string()));
    }
}
