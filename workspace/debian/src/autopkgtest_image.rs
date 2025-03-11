use super::execute::{execute_command_with_sudo, Execute};
/// Provides functionality for building and managing Autopkgtest VM images
///
/// This module contains structures and implementations for creating
/// virtual machine images compatible with autopkgtest for different
/// Linux distributions.
use eyre::{eyre, Result};
use log::info;
use std::path::{Path, PathBuf};

/// Represents supported Linux distributions for VM image creation
///
/// Each variant contains the specific codename for the distribution
/// which is used to identify compatible build commands and arguments.
#[derive(Debug, Clone)]
enum Distribution {
    /// Debian distribution with codename (e.g., "bookworm")
    Debian(String),
    /// Ubuntu distribution with codename (e.g., "noble", "jammy")
    Ubuntu(String),
}

impl Distribution {
    /// Creates a Distribution from a codename string
    ///
    /// # Arguments
    /// * `codename` - The distribution codename (e.g., "bookworm", "noble")
    ///
    /// # Returns
    /// * `Result<Self>` - A Distribution instance or an error if unsupported
    pub fn from_codename(codename: &str) -> Result<Self> {
        match codename {
            "bookworm" => Ok(Distribution::Debian(codename.to_string())),
            "noble" | "jammy"=> {
                Ok(Distribution::Ubuntu(codename.to_string()))
            }
            _ => Err(eyre!("Unsupported distribution codename: {}", codename)),
        }
    }

    /// Returns the appropriate command for building an image for this distribution
    ///
    /// # Returns
    /// * `&'static str` - The command to use for image creation
    pub fn get_command(&self) -> &'static str {
        match self {
            Distribution::Debian(_) => "autopkgtest-build-qemu",
            Distribution::Ubuntu(_) => "autopkgtest-buildvm-ubuntu-cloud",
        }
    }

    /// Returns the codename formatted as an argument for the build command
    ///
    /// # Returns
    /// * `String` - The formatted codename argument
    pub fn get_codename_arg(&self) -> String {
        match self {
            Distribution::Debian(codename) => codename.clone(),
            Distribution::Ubuntu(codename) => format!("--release={}", codename),
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
    pub fn codename(mut self, codename: &str) -> Result<Self> {
        self.distribution = Some(Distribution::from_codename(codename)?);
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
    pub fn image_path(mut self, cache_dir: &str, codename: &str, arch: &str) -> Self {
        let image_name = format!("autopkgtest-{}-{}.img", codename, arch);
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
            args.push(dist.get_codename_arg());
        } else {
            return Err(eyre!("Distribution not specified"));
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
            args.push(path.to_string_lossy().to_string());
        } else {
            return Err(eyre!("Image path not specified"));
        }

        Ok(args)
    }
}

/// Implementation of the Execute trait for AutopkgtestImageBuilder
///
/// Allows the builder to be executed to create the VM image.
impl Execute for AutopkgtestImageBuilder {
    /// Executes the VM image creation process
    ///
    /// # Returns
    /// * `Result<()>` - Success or an error if the build fails
    fn execute(&self) -> Result<()> {
        let cmd = self
            .distribution
            .as_ref()
            .ok_or_else(|| eyre!("Distribution not specified"))?
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
            .codename("bookworm").unwrap()
            .image_path("/tmp", "bookworm", "amd64")
            .arch("amd64")
            .mirror("http://example.com/debian");

        let args = builder.build_args().unwrap();
        assert!(args.contains(&"bookworm".to_string()));
        assert!(args.iter().any(|arg| arg.contains("/tmp/autopkgtest-bookworm-amd64.img")));
        assert!(args.contains(&"--arch=amd64".to_string()));
        assert!(args.contains(&"--mirror=http://example.com/debian".to_string()));
    }
}
