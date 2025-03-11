use eyre::{Result, WrapErr};
use log::info;
use std::path::{Path, PathBuf};

use super::execute::{execute_command, Execute};

/// `Autopkgtest` provides a builder interface for autopkgtest commands.
///
/// This struct implements the builder pattern to configure and execute
/// autopkgtest commands, which are used for testing Debian packages.
/// It allows configuring various options such as changes files, test setup,
/// virtualization options, and more.
///
/// # Examples
///
/// ```
/// use debian::autopkgtest::Autopkgtest;
/// use crate::debian::execute::Execute;
/// use std::path::Path;
///
/// let result = Autopkgtest::new()
///     .changes_file("package.changes")
///     .apt_upgrade()
///     .setup_commands("apt-get install dependency")
///     .qemu("/path/to/image.img")
///     .working_dir(Path::new("/tmp/working_dir"))
///     .execute();
/// ```
#[derive(Debug, Clone, Default)]
pub struct Autopkgtest {
    changes_file: Option<String>,
    no_built_binaries: bool,
    apt_upgrade: bool,
    setup_commands: Vec<String>,
    qemu_image: Option<String>,
    dir: Option<PathBuf>,
}

/// Builder for Autopkgtest commands
impl Autopkgtest {
    /// Creates a new instance of `Autopkgtest` with default configuration.
    ///
    /// All options are initialized to their default values:
    /// - No changes file
    /// - No built binaries flag disabled
    /// - Apt upgrade flag disabled
    /// - No setup commands
    /// - No QEMU image
    /// - No working directory
    ///
    /// # Returns
    ///
    /// A new `Autopkgtest` instance with default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Specifies a changes file to be processed by autopkgtest.
    ///
    /// This sets the path to a .changes file that will be used by autopkgtest.
    /// The .changes file contains information about the package to be tested.
    ///
    /// # Arguments
    ///
    /// * `file` - Path to the .changes file
    ///
    /// # Returns
    ///
    /// The updated builder instance.
    pub fn changes_file(mut self, file: &str) -> Self {
        self.changes_file = Some(file.to_string());
        self
    }

    /// Adds the `--no-built-binaries` flag to the command.
    ///
    /// When this flag is set, autopkgtest will ignore any built binaries
    /// that might be available and build everything from source.
    ///
    /// # Returns
    ///
    /// The updated builder instance.
    pub fn no_built_binaries(mut self) -> Self {
        self.no_built_binaries = true;
        self
    }

    /// Adds the `--apt-upgrade` flag to the command.
    ///
    /// When this flag is set, autopkgtest will run apt-get upgrade
    /// before starting the tests.
    ///
    /// # Returns
    ///
    /// The updated builder instance.
    pub fn apt_upgrade(mut self) -> Self {
        self.apt_upgrade = true;
        self
    }

    /// Adds a setup command to the autopkgtest.
    ///
    /// This method allows specifying shell commands that will be run
    /// in the test environment before the tests are executed.
    ///
    /// # Arguments
    ///
    /// * `command` - A shell command to run in the test environment
    ///
    /// # Returns
    ///
    /// The updated builder instance.
    pub fn setup_commands(mut self, command: &str) -> Self {
        self.setup_commands.push(command.to_string());
        self
    }

    /// Specifies test dependencies that are not in Debian repositories.
    ///
    /// This is a convenience method that adds each dependency as a separate setup command.
    /// It can be used to install dependencies that are not available in standard repositories.
    ///
    /// # Arguments
    ///
    /// * `deps` - A slice of strings representing the dependencies
    ///
    /// # Returns
    ///
    /// The updated builder instance.
    pub fn test_deps_not_in_debian(mut self, deps: &[String]) -> Self {
        for dep in deps {
            self.setup_commands.push(dep.clone());
        }
        self
    }

    /// Configures the command to use QEMU with the specified image.
    ///
    /// This adds the necessary arguments to run the tests in a QEMU virtual machine
    /// using the specified disk image.
    ///
    /// # Arguments
    ///
    /// * `image_path` - Path to the QEMU disk image
    ///
    /// # Returns
    ///
    /// The updated builder instance.
    pub fn qemu(mut self, image_path: &str) -> Self {
        self.qemu_image = Some(image_path.to_string());
        self
    }

    /// Sets the working directory for the command execution.
    ///
    /// This specifies the directory from which the autopkgtest command
    /// will be executed.
    ///
    /// # Arguments
    ///
    /// * `dir` - Path to the working directory
    ///
    /// # Returns
    ///
    /// The updated builder instance.
    pub fn working_dir(mut self, dir: &Path) -> Self {
        self.dir = Some(dir.to_path_buf());
        self
    }

    /// Builds the command arguments based on the configured options.
    ///
    /// This method constructs a vector of strings representing the
    /// command-line arguments that will be passed to the autopkgtest command.
    ///
    /// # Returns
    ///
    /// A vector of strings containing the command arguments.
    fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        
        // Add changes file if provided
        if let Some(file) = &self.changes_file {
            args.push(file.clone());
        }
        
        // Add flags based on boolean options
        if self.no_built_binaries {
            args.push("--no-built-binaries".to_string());
        }
        
        if self.apt_upgrade {
            args.push("--apt-upgrade".to_string());
        }
        
        // Add setup commands
        for cmd in &self.setup_commands {
            args.push(format!("--setup-commands={}", cmd));
        }
        
        // Add QEMU configuration if provided
        if let Some(image) = &self.qemu_image {
            args.push("--".to_string());
            args.push("qemu".to_string());
            args.push(image.clone());
        }
        
        args
    }
}

/// Implementation of the `Execute` trait for `Autopkgtest`.
///
/// This allows an `Autopkgtest` instance to be executed using the `execute()` method.
impl Execute for Autopkgtest {
    /// Executes the autopkgtest command with the configured options.
    ///
    /// This method builds the command arguments and runs the autopkgtest command.
    /// It logs the command being executed and returns an error if the execution fails.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure. If successful, the result is `Ok(())`.
    /// If there's an error, it will be wrapped with additional context information.
    fn execute(&self) -> Result<()> {
        let args = self.build_args();
        let args_str = args.join(" ");
        info!("Running: autopkgtest {}", args_str);
        
        execute_command("autopkgtest", &args, self.dir.as_deref())
            .wrap_err_with(|| format!("Failed to execute autopkgtest command: {}", args_str))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;


    #[test]
    fn test_new() {
        let autopkgtest = Autopkgtest::new();
        assert!(autopkgtest.changes_file.is_none());
        assert!(!autopkgtest.no_built_binaries);
        assert!(!autopkgtest.apt_upgrade);
        assert!(autopkgtest.setup_commands.is_empty());
        assert!(autopkgtest.qemu_image.is_none());
        assert!(autopkgtest.dir.is_none());
    }

    #[test]
    fn test_changes_file() {
        let autopkgtest = Autopkgtest::new().changes_file("test.changes");
        assert_eq!(autopkgtest.changes_file, Some("test.changes".to_string()));
    }

    #[test]
    fn test_no_built_binaries() {
        let autopkgtest = Autopkgtest::new().no_built_binaries();
        assert!(autopkgtest.no_built_binaries);
    }

    #[test]
    fn test_apt_upgrade() {
        let autopkgtest = Autopkgtest::new().apt_upgrade();
        assert!(autopkgtest.apt_upgrade);
    }

    #[test]
    fn test_setup_command() {
        let autopkgtest = Autopkgtest::new()
            .setup_commands("apt-get install foo")
            .setup_commands("echo 'test'");
        
        assert_eq!(
            autopkgtest.setup_commands,
            vec!["apt-get install foo".to_string(), "echo 'test'".to_string()]
        );
    }

    #[test]
    fn test_test_deps_not_in_debian() {
        let deps = vec!["dep1".to_string(), "dep2".to_string()];
        let autopkgtest = Autopkgtest::new().test_deps_not_in_debian(&deps);
        assert_eq!(autopkgtest.setup_commands, deps);
    }

    #[test]
    fn test_qemu() {
        let autopkgtest = Autopkgtest::new().qemu("/path/to/image.img");
        assert_eq!(autopkgtest.qemu_image, Some("/path/to/image.img".to_string()));
    }

    #[test]
    fn test_working_dir() {
        let dir = PathBuf::from("/tmp/test");
        let autopkgtest = Autopkgtest::new().working_dir(&dir);
        assert_eq!(autopkgtest.dir, Some(dir));
    }

    #[test]
    fn test_build_args() {
        let autopkgtest = Autopkgtest::new()
            .changes_file("test.changes")
            .no_built_binaries()
            .apt_upgrade()
            .setup_commands("apt-get install pkg1")
            .setup_commands("apt-get install pkg2")
            .qemu("/path/to/image.img");
        
        let args = autopkgtest.build_args();
        
        assert_eq!(
            args,
            vec![
                "test.changes",
                "--no-built-binaries",
                "--apt-upgrade",
                "--setup-commands=apt-get install pkg1",
                "--setup-commands=apt-get install pkg2",
                "--",
                "qemu",
                "/path/to/image.img"
            ]
        );
    }

    #[test]
    fn test_multiple_setup_commands() {
        let autopkgtest = Autopkgtest::new()
            .setup_commands("cmd1")
            .setup_commands("cmd2")
            .setup_commands("cmd3");
        
        let args = autopkgtest.build_args();
        
        assert_eq!(
            args,
            vec![
                "--setup-commands=cmd1",
                "--setup-commands=cmd2",
                "--setup-commands=cmd3"
            ]
        );
    }
}