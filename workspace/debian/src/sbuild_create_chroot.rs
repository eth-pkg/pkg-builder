use super::execute::{execute_command, Execute, ExecuteError};
use log::info;
use types::distribution::Distribution;
use std::path::Path;
use thiserror::Error;

/// Represents options for creating an sbuild chroot environment
///
/// This struct provides a builder pattern to configure and execute
/// the `sbuild-createchroot` command with various options.
#[derive(Debug, Default)]
pub struct SbuildCreateChroot {
    /// The chroot mode (e.g., "schroot", "unshare")
    chroot_mode: Option<String>,
    /// Whether to create a tarball
    make_tarball: bool,
    /// Path to the cache file
    cache_file: Option<String>,
    /// Codename of the distribution (e.g., "bullseye")
    codename: Option<Distribution>,
    /// Directory to use for temporary files
    temp_dir: Option<String>,
    /// URL of the repository to use
    repo_url: Option<String>,
}

/// Custom error type for sbuild-createchroot operations
#[derive(Error, Debug)]
pub enum SbuildCreateChrootError {
    #[error("Failed to execute command: {0}")]
    CommandExecutionError(#[from] ExecuteError),
}

type Result<T> = std::result::Result<T, SbuildCreateChrootError>;

impl SbuildCreateChroot {
    /// Creates a new instance with default options
    ///
    /// # Examples
    ///
    /// ```
    /// use debian::sbuild_create_chroot::SbuildCreateChroot;
    /// let chroot = SbuildCreateChroot::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the chroot mode
    ///
    /// # Arguments
    ///
    /// * `mode` - The chroot mode to use (e.g., "schroot", "unshare")
    ///
    /// # Examples
    ///
    /// ```
    /// use debian::sbuild_create_chroot::SbuildCreateChroot;
    /// let chroot = SbuildCreateChroot::new().chroot_mode("unshare");
    /// ```
    pub fn chroot_mode(mut self, mode: &str) -> Self {
        self.chroot_mode = Some(mode.to_string());
        self
    }

    /// Enables creation of a tarball
    ///
    /// # Examples
    ///
    /// ```
    /// use debian::sbuild_create_chroot::SbuildCreateChroot;
    /// let chroot = SbuildCreateChroot::new().make_tarball();
    /// ```
    pub fn make_tarball(mut self) -> Self {
        self.make_tarball = true;
        self
    }

    /// Sets the cache file path
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the cache file
    ///
    /// # Examples
    ///
    /// ```
    /// use debian::sbuild_create_chroot::SbuildCreateChroot;
    /// let chroot = SbuildCreateChroot::new().cache_file("/path/to/cache");
    /// ```
    pub fn cache_file(mut self, path: &str) -> Self {
        self.cache_file = Some(path.to_string());
        self
    }

    /// Sets the distribution codename
    ///
    /// # Arguments
    ///
    /// * `name` - Codename of the distribution (e.g., "bullseye")
    ///
    /// # Examples
    ///
    /// ```
    /// use debian::sbuild_create_chroot::SbuildCreateChroot;
    /// let chroot = SbuildCreateChroot::new().codename("bullseye");
    /// ```
    pub fn codename(mut self, name: &Distribution) -> Self {
        self.codename = Some(name.clone());
        self
    }

    /// Sets the temporary directory
    ///
    /// # Arguments
    ///
    /// * `dir` - Path to the temporary directory
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use debian::sbuild_create_chroot::SbuildCreateChroot;
    /// let temp = Path::new("/tmp/sbuild");
    /// let chroot = SbuildCreateChroot::new().temp_dir(temp);
    /// ```
    pub fn temp_dir(mut self, dir: &Path) -> Self {
        self.temp_dir = Some(dir.to_string_lossy().to_string());
        self
    }

    /// Sets the repository URL
    ///
    /// # Arguments
    ///
    /// * `url` - URL of the repository to use
    ///
    /// # Examples
    ///
    /// ```
    /// use debian::sbuild_create_chroot::SbuildCreateChroot;
    /// let chroot = SbuildCreateChroot::new().repo_url("http://deb.debian.org/debian");
    /// ```
    pub fn repo_url(mut self, url: &str) -> Self {
        self.repo_url = Some(url.to_string());
        self
    }

    /// Builds the command arguments based on the configured options
    fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if let Some(mode) = &self.chroot_mode {
            args.push(format!("--chroot-mode={}", mode));
        }

        if self.make_tarball {
            args.push("--make-sbuild-tarball".to_string());
        }

        if let Some(cache) = &self.cache_file {
            args.push(cache.clone());
        }

        if let Some(name) = &self.codename {
            args.push(name.as_short().into());
        }

        if let Some(dir) = &self.temp_dir {
            args.push(dir.clone());
        }

        if let Some(url) = &self.repo_url {
            args.push(url.clone());
        }

        args
    }
}

impl Execute for SbuildCreateChroot {
    type Error = SbuildCreateChrootError;
    /// Executes the sbuild-createchroot command with the configured options
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or containing an error
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// # use debian::sbuild_create_chroot::SbuildCreateChroot;
    /// # use debian::sbuild_create_chroot::SbuildCreateChrootError;
    /// # use debian::execute::Execute;
    /// # fn run() -> Result<(), SbuildCreateChrootError> {
    /// let chroot = SbuildCreateChroot::new()
    ///     .chroot_mode("unshare")
    ///     .make_tarball()
    ///     .codename("bullseye");
    ///
    /// chroot.execute()?;
    /// # Ok(())
    /// # }
    /// ```
    fn execute(&self) -> Result<()> {
        let args = self.build_args();
        info!("Running: sbuild-createchroot {}", args.join(" "));
        execute_command("sbuild-createchroot", &args, None)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    // use mockall::predicate::*;
    // use mockall::mock;

    // // Mock the execute_command function
    // mock! {
    //     pub fn ExecuteCommand {}
    //     impl ExecuteCommand {
    //         pub fn execute_command(cmd: &str, args: &[String], env: Option<&[(String, String)]>) -> Result<()>;
    //     }
    // }

    #[test]
    fn test_default_new() {
        let chroot = SbuildCreateChroot::new();
        assert!(chroot.chroot_mode.is_none());
        assert!(!chroot.make_tarball);
        assert!(chroot.cache_file.is_none());
        assert!(chroot.codename.is_none());
        assert!(chroot.temp_dir.is_none());
        assert!(chroot.repo_url.is_none());
    }

    #[test]
    fn test_build_args_empty() {
        let chroot = SbuildCreateChroot::new();
        let args = chroot.build_args();
        assert!(args.is_empty());
    }

    #[test]
    fn test_build_args_with_options() {
        let temp_path = PathBuf::from("/tmp/sbuild");
        let chroot = SbuildCreateChroot::new()
            .chroot_mode("unshare")
            .make_tarball()
            .cache_file("/var/cache/sbuild.tar.gz")
            .codename(&Distribution::bookworm())
            .temp_dir(&temp_path)
            .repo_url("http://deb.debian.org/debian");

        let args = chroot.build_args();

        assert_eq!(args.len(), 6);
        assert_eq!(args[0], "--chroot-mode=unshare");
        assert_eq!(args[1], "--make-sbuild-tarball");
        assert_eq!(args[2], "/var/cache/sbuild.tar.gz");
        assert_eq!(args[3], "bullseye");
        assert_eq!(args[4], "/tmp/sbuild");
        assert_eq!(args[5], "http://deb.debian.org/debian");
    }

    // #[test]
    // fn test_execute() {
    //     let mut mock = MockExecuteCommand::new();

    //     // Set up expectations
    //     mock.expect_execute_command()
    //         .with(
    //             eq("sbuild-createchroot"),
    //             eq(vec!["--chroot-mode=unshare".to_string(), "bullseye".to_string()]),
    //             eq(None)
    //         )
    //         .times(1)
    //         .returning(|_, _, _| Ok(()));

    //     // Create and execute the command
    //     let chroot = SbuildCreateChroot::new()
    //         .chroot_mode("unshare")
    //         .codename("bullseye");

    //     // This would fail without mocking the actual command execution
    //     // Here we'd need a way to inject our mock into the execution
    //     // For a real test, you might use dependency injection or function pointers
    // }
}
