use super::execute::{execute_command, Execute, ExecuteError};
use log::info;
use types::distribution::Distribution;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// A builder for configuring and executing the sbuild command.
///
/// This struct allows for fluid configuration of sbuild parameters using
/// the builder pattern. Once configured, the command can be executed
/// via the `Execute` trait implementation.
///
/// # Example
///
/// ```
/// use debian::sbuild::SbuildBuilder;
/// use debian::execute::Execute;
///
/// let result = SbuildBuilder::new()
///     .distribution("bullseye")
///     .build_arch_all()
///     .verbose()
///     .execute();
/// ```

#[derive(Default, Debug, Clone)]
pub struct SbuildBuilder {
    /// Debian distribution codename (e.g., "bullseye", "bookworm")
    distribution: Option<Distribution>,
    /// Whether to build architecture-independent packages
    build_arch_all: bool,
    /// Whether to build source packages only
    build_source: bool,
    /// Path to the sbuild cache file
    cache_file: Option<String>,
    /// Whether to enable verbose output
    verbose: bool,
    /// Whether to use unshare chroot mode
    chroot_mode_unshare: bool,
    /// Additional setup commands to pass to sbuild
    setup_commands: Vec<String>,
    /// Whether to run piuparts after building
    run_piuparts: bool,
    /// Whether to perform apt upgrades before building
    apt_upgrades: bool,
    /// Whether to run lintian after building (None = use sbuild default)
    run_lintian: Option<bool>,
    /// Whether to run autopkgtest after building
    run_autopkgtest: bool,
    /// Working directory for the sbuild command
    dir: Option<PathBuf>,
}

#[derive(Error, Debug)]
pub enum SbuildCmdError {
    #[error("Failed to execute command: {0}")]
    CommandExecutionError(#[from] ExecuteError),
}

type Result<T> = std::result::Result<T, SbuildCmdError>;

impl SbuildBuilder {
    /// Creates a new SbuildBuilder with default settings.
    ///
    /// Default settings:
    /// - No distribution specified
    /// - Architecture-independent packages not built
    /// - Source packages not built
    /// - No cache file specified
    /// - Normal (non-verbose) output
    /// - Default chroot mode
    /// - No additional setup commands
    /// - piuparts enabled
    /// - apt upgrades enabled
    /// - lintian uses sbuild default
    /// - autopkgtest enabled
    /// - No working directory specified
    pub fn new() -> Self {
        Self {
            distribution: None,
            build_arch_all: false,
            build_source: false,
            cache_file: None,
            verbose: false,
            chroot_mode_unshare: false,
            setup_commands: Vec::new(),
            run_piuparts: true,
            apt_upgrades: true,
            run_lintian: None,
            run_autopkgtest: true,
            dir: None,
        }
    }

    /// Sets the target distribution for the build.
    ///
    /// This corresponds to the `-d` flag in sbuild.
    ///
    /// # Arguments
    ///
    /// * `codename` - The Debian distribution codename (e.g., "bullseye", "bookworm")
    pub fn distribution(mut self, codename: &Distribution) -> Self {
        self.distribution = Some(codename.clone());
        self
    }

    /// Enables building of architecture-independent packages.
    ///
    /// This corresponds to the `-A` flag in sbuild.
    pub fn build_arch_all(mut self) -> Self {
        self.build_arch_all = true;
        self
    }

    /// Enables building of source packages only.
    ///
    /// This corresponds to the `-s` and `--source-only-changes` flags in sbuild.
    pub fn build_source(mut self) -> Self {
        self.build_source = true;
        self
    }

    /// Sets the cache file to use for the build.
    ///
    /// This corresponds to the `-c` flag in sbuild.
    ///
    /// # Arguments
    ///
    /// * `cache_file` - Path to the cache file
    pub fn cache_file(mut self, cache_file: &str) -> Self {
        self.cache_file = Some(cache_file.to_string());
        self
    }

    /// Enables verbose output.
    ///
    /// This corresponds to the `-v` flag in sbuild.
    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }

    /// Sets the chroot mode to unshare.
    ///
    /// This corresponds to the `--chroot-mode=unshare` flag in sbuild.
    pub fn chroot_mode_unshare(mut self) -> Self {
        self.chroot_mode_unshare = true;
        self
    }

    /// Adds custom setup commands to pass to sbuild.
    ///
    /// # Arguments
    ///
    /// * `commands` - List of additional commands to pass to sbuild
    pub fn setup_commands(mut self, commands: &[String]) -> Self {
        self.setup_commands
            .extend(commands.iter().map(|s| s.to_string()));
        self
    }

    /// Disables running piuparts after building.
    ///
    /// This corresponds to the `--no-run-piuparts` flag in sbuild.
    pub fn no_run_piuparts(mut self) -> Self {
        self.run_piuparts = false;
        self
    }

    /// Disables apt upgrades before building.
    ///
    /// This corresponds to the `--no-apt-upgrade` and `--no-apt-distupgrade` flags in sbuild.
    pub fn no_apt_upgrades(mut self) -> Self {
        self.apt_upgrades = false;
        self
    }

    /// Sets whether to run lintian after building.
    ///
    /// When enabled, adds several lintian-related flags with common options.
    /// When disabled, adds the `--no-run-lintian` flag.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to enable or disable lintian
    pub fn run_lintian(mut self, enabled: bool) -> Self {
        self.run_lintian = Some(enabled);
        self
    }

    /// Disables running autopkgtest after building.
    ///
    /// This corresponds to the `--no-run-autopkgtest` flag in sbuild.
    pub fn no_run_autopkgtest(mut self) -> Self {
        self.run_autopkgtest = false;
        self
    }

    /// Sets the working directory for the sbuild command.
    ///
    /// # Arguments
    ///
    /// * `dir` - Path to the working directory
    pub fn working_dir(mut self, dir: &Path) -> Self {
        self.dir = Some(dir.to_path_buf());
        self
    }

    /// Converts the builder configuration into command-line arguments.
    ///
    /// This method translates the builder's state into a vector of strings
    /// that can be passed to the sbuild command.
    ///
    /// # Returns
    ///
    /// A vector of strings representing the sbuild command-line arguments.
    fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if let Some(dist) = &self.distribution {
            args.push("-d".to_string());
            args.push(dist.as_short().into());
        }

        if self.build_arch_all {
            args.push("-A".to_string());
        }

        if self.build_source {
            args.push("-s".to_string());
            args.push("--source-only-changes".to_string());
        }

        if let Some(cache) = &self.cache_file {
            args.push("-c".to_string());
            args.push(cache.clone());
        }

        if self.verbose {
            args.push("-v".to_string());
        }

        if self.chroot_mode_unshare {
            args.push("--chroot-mode=unshare".to_string());
        }

        args.extend(self.setup_commands.clone());

        if !self.run_piuparts {
            args.push("--no-run-piuparts".to_string());
        }

        if !self.apt_upgrades {
            args.push("--no-apt-upgrade".to_string());
            args.push("--no-apt-distupgrade".to_string());
        }

        if let Some(enabled) = self.run_lintian {
            if enabled {
                args.extend([
                    "--run-lintian".to_string(),
                    "--lintian-opt=-i".to_string(),
                    "--lintian-opt=--I".to_string(),
                    "--lintian-opt=--suppress-tags".to_string(),
                    "--lintian-opt=bad-distribution-in-changes-file".to_string(),
                    "--lintian-opt=--suppress-tags".to_string(),
                    "--lintian-opt=debug-file-with-no-debug-symbols".to_string(),
                    "--lintian-opt=--tag-display-limit=0".to_string(),
                    "--lintian-opts=--fail-on=error".to_string(),
                    "--lintian-opts=--fail-on=warning".to_string(),
                ]);
            } else {
                args.push("--no-run-lintian".to_string());
            }
        }

        if !self.run_autopkgtest {
            args.push("--no-run-autopkgtest".to_string());
        }

        args
    }
}

/// Implementation of the Execute trait for SbuildBuilder.
///
/// This allows the builder to be executed directly after configuration.
impl Execute for SbuildBuilder {
    type Error = SbuildCmdError;
    /// Executes the sbuild command with the configured options.
    ///
    /// # Returns
    ///
    /// Ok(()) if the command executed successfully, or an error if it failed.
    ///
    /// # Errors
    ///
    /// Returns an error if the sbuild command fails to execute or returns a non-zero exit code.
    fn execute(&self) -> Result<()> {
        let args = self.build_args();
        info!("Running: sbuild {}", &args.join(" "));
        execute_command("sbuild", &args, self.dir.as_deref())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_new_builder() {
        let builder = SbuildBuilder::new();
        assert!(builder.distribution.is_none());
        assert!(!builder.build_arch_all);
        assert!(!builder.build_source);
        assert!(builder.cache_file.is_none());
        assert!(!builder.verbose);
        assert!(!builder.chroot_mode_unshare);
        assert!(builder.setup_commands.is_empty());
        assert!(builder.run_piuparts);
        assert!(builder.apt_upgrades);
        assert!(builder.run_lintian.is_none());
        assert!(builder.run_autopkgtest);
        assert!(builder.dir.is_none());
    }

    #[test]
    fn test_builder_methods() {
        let path = PathBuf::from("/tmp/test");
        let builder = SbuildBuilder::new()
            .distribution(&Distribution::bookworm())
            .build_arch_all()
            .build_source()
            .cache_file("cache.txt")
            .verbose()
            .chroot_mode_unshare()
            .setup_commands(&["--foo".to_string(), "--bar".to_string()])
            .no_run_piuparts()
            .no_apt_upgrades()
            .run_lintian(true)
            .no_run_autopkgtest()
            .working_dir(&path);

        assert_eq!(builder.distribution, Some(Distribution::bookworm()));
        assert!(builder.build_arch_all);
        assert!(builder.build_source);
        assert_eq!(builder.cache_file, Some("cache.txt".to_string()));
        assert!(builder.verbose);
        assert!(builder.chroot_mode_unshare);
        assert_eq!(
            builder.setup_commands,
            vec!["--foo".to_string(), "--bar".to_string()]
        );
        assert!(!builder.run_piuparts);
        assert!(!builder.apt_upgrades);
        assert_eq!(builder.run_lintian, Some(true));
        assert!(!builder.run_autopkgtest);
        assert_eq!(builder.dir, Some(path));
    }

    #[test]
    fn test_build_args() {
        let builder = SbuildBuilder::new()
            .distribution(&Distribution::bookworm())
            .build_arch_all()
            .verbose()
            .run_lintian(false);

        let args = builder.build_args();
        assert!(args.contains(&"-d".to_string()));
        assert!(args.contains(&"bookworm".to_string()));
        assert!(args.contains(&"-A".to_string()));
        assert!(args.contains(&"-v".to_string()));
        assert!(args.contains(&"--no-run-lintian".to_string()));

        // Test with lintian enabled
        let builder = SbuildBuilder::new().run_lintian(true);
        let args = builder.build_args();
        assert!(args.contains(&"--run-lintian".to_string()));
        assert!(args.contains(&"--lintian-opt=-i".to_string()));
    }

    #[test]
    fn test_custom_commands() {
        let commands = vec![
            "--foo=bar".to_string(),
            "--baz".to_string(),
            "--qux=quux".to_string(),
        ];

        let builder = SbuildBuilder::new().setup_commands(&commands);
        let args = builder.build_args();

        for cmd in commands {
            assert!(args.contains(&cmd));
        }
    }
}
