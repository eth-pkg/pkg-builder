use super::execute::{execute_command_with_sudo, Execute};
use eyre::Result;
use log::info;
use std::path::Path;

/// A builder for the piuparts command, which tests Debian package installation,
/// upgrading, and removal processes.
///
/// Piuparts (Package Installation, UPgrading And Removal Testing Suite) helps validate 
/// Debian packages by testing their installation, upgrade paths, and purging in a clean 
/// chroot environment. This struct implements a builder pattern to configure and execute
/// piuparts with various options.
///
/// # Examples
///
/// ```rust
/// use std::path::Path;
/// use debian::piuparts::Piuparts;
/// use crate::debian::execute::Execute;
///
/// // Basic usage
/// let deb_file = Path::new("/path/to/package.deb");
/// let result = Piuparts::new()
///     .distribution("bookworm")
///     .mirror("http://deb.debian.org/debian")
///     .verbose()
///     .deb_file(deb_file)
///     .execute();
///
/// // With .NET environment setup
/// let result = Piuparts::new()
///     .distribution("bookworm")
///     .with_dotnet_env(true, "bookworm")
///     .deb_file(deb_file)
///     .execute();
/// ```
///
/// # Note
///
/// This command requires sudo privileges to run piuparts, as it operates
/// on system package management and creates isolated environments.
pub struct Piuparts<'a> {
    /// Distribution codename (e.g., "bookworm", "jammy")
    distribution: Option<String>,
    /// Mirror URL for package repository
    mirror: Option<String>,
    /// Paths to bind mount into the chroot
    bindmounts: Vec<String>,
    /// Path to keyring file for package verification
    keyring: Option<String>,
    /// Whether to enable verbose output
    verbose: bool,
    /// Additional repositories to use
    extra_repos: Vec<String>,
    /// Whether to verify package signatures
    verify_signatures: bool,
    /// Path to the .deb file to test
    deb_file: Option<&'a Path>,
    /// Directory containing the .deb file
    deb_path: Option<&'a Path>,
}

impl<'a> Piuparts<'a> {
    /// Creates a new Piuparts builder with default settings.
///
/// Default values:
/// - No distribution specified
/// - No mirror specified
/// - No bind mounts
/// - No keyring specified
/// - Verbose mode disabled
/// - No extra repositories
/// - Package signature verification enabled
/// - No .deb file or path specified
    pub fn new() -> Self {
        Self {
            distribution: None,
            mirror: None,
            bindmounts: Vec::new(),
            keyring: None,
            verbose: false,
            extra_repos: Vec::new(),
            verify_signatures: true,
            deb_file: None,
            deb_path: None,
        }
    }

    /// Sets the distribution codename (e.g., "bookworm", "jammy").
///
/// This corresponds to the `-d` option in piuparts and specifies the 
/// Debian/Ubuntu distribution to use for testing. The distribution must
/// be available in the specified mirror.
///
/// # Arguments
///
/// * `codename` - The distribution codename (e.g., "bookworm", "jammy", "bullseye")
    pub fn distribution(mut self, codename: &str) -> Self {
        self.distribution = Some(codename.to_string());
        self
    }

    /// Sets the mirror URL for the package repository.
///
/// This corresponds to the `-m` option in piuparts and specifies the
/// package repository mirror to use for downloading packages.
///
/// # Arguments
///
/// * `url` - The URL of the repository mirror (e.g., "http://deb.debian.org/debian")
    pub fn mirror(mut self, url: &str) -> Self {
        self.mirror = Some(url.to_string());
        self
    }

    /// Adds /dev to the list of directories to bind mount into the chroot.
///
/// This corresponds to the `--bindmount=/dev` option in piuparts and allows
/// the chroot environment to access the host's /dev directory. This can be
/// necessary for certain packages that need access to device files.
///
/// # Note
///
/// Binding /dev can introduce security risks by giving the chroot
/// environment access to the host's devices.
    pub fn bindmount_dev(mut self) -> Self {
        self.bindmounts.push("/dev".to_string());
        self
    }

    /// Sets the keyring file to use for package verification.
///
/// This corresponds to the `--keyring` option in piuparts and specifies the
/// GPG keyring file to use for package signature verification.
///
/// # Arguments
///
/// * `keyring` - The path to the keyring file
    pub fn keyring(mut self, keyring: &str) -> Self {
        self.keyring = Some(keyring.to_string());
        self
    }

    /// Enables verbose output.
///
/// This corresponds to the `--verbose` option in piuparts and causes
/// piuparts to output more detailed information during testing, which
/// can be helpful for debugging.
    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }

    /// Adds an additional repository to use.
///
/// This corresponds to the `--extra-repo` option in piuparts and allows
/// specifying additional package repositories beyond the main mirror.
///
/// # Arguments
///
/// * `repo` - The repository definition in sources.list format 
///            (e.g., "deb http://security.debian.org/debian-security bookworm-security main")
    pub fn extra_repo(mut self, repo: &str) -> Self {
        self.extra_repos.push(repo.to_string());
        self
    }

    /// Disables package signature verification.
///
/// This corresponds to the `--do-not-verify-signatures` option in piuparts
/// and disables GPG signature verification for packages. This can be necessary
/// when using unofficial repositories that don't provide properly signed packages.
///
/// # Security Note
///
/// Disabling signature verification reduces security by allowing potentially
/// tampered packages to be installed.
    pub fn no_verify_signatures(mut self) -> Self {
        self.verify_signatures = false;
        self
    }

    /// Configures the environment for .NET packages if needed.
    ///
    /// If `is_dotnet` is true and the distribution is either "bookworm" or "jammy jellyfish",
    /// adds the Microsoft repository and disables signature verification.
    pub fn with_dotnet_env(self, is_dotnet: bool, codename: &str) -> Self {
        if is_dotnet && (codename == "bookworm" || codename == "jammy") {
            let repo = format!(
                "deb https://packages.microsoft.com/debian/12/prod {} main",
                codename
            );
            return self.extra_repo(&repo).no_verify_signatures();
        }
        self
    }

    /// Sets the .deb file to test.
///
/// This specifies the Debian package file that piuparts will test.
/// This is a required parameter for executing piuparts.
///
/// # Arguments
///
/// * `deb_file` - Path to the .deb file to test
    pub fn deb_file(mut self, deb_file: &'a Path) -> Self {
        self.deb_file = Some(deb_file);
        self
    }

    /// Sets the directory containing the .deb file.
///
/// This specifies the directory where the .deb file is located.
/// It can be useful when the execution needs to know the context directory.
///
/// # Arguments
///
/// * `deb_path` - Path to the directory containing the .deb file
    pub fn deb_path(mut self, deb_path: &'a Path) -> Self {
        self.deb_path = Some(deb_path);
        self
    }

    /// Builds the command-line arguments for piuparts based on the configured options.
///
/// This method converts the builder's state into a vector of string arguments
/// that can be passed to the piuparts command. It's called internally by the
/// `execute()` method.
///
/// # Returns
///
/// A vector of strings representing the command-line arguments
    fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if let Some(dist) = &self.distribution {
            args.push("-d".to_string());
            args.push(dist.clone());
        }

        if let Some(mirror_url) = &self.mirror {
            args.push("-m".to_string());
            args.push(mirror_url.clone());
        }

        for bindmount in &self.bindmounts {
            args.push(format!("--bindmount={}", bindmount));
        }

        if let Some(keyring_path) = &self.keyring {
            args.push(format!("--keyring={}", keyring_path));
        }

        if self.verbose {
            args.push("--verbose".to_string());
        }

        for repo in &self.extra_repos {
            args.push(format!("--extra-repo={}", repo));
        }

        if !self.verify_signatures {
            args.push("--do-not-verify-signatures".to_string());
        }

        if let Some(deb_file) = &self.deb_file {
            args.push(deb_file.display().to_string());
        }

        args
    }
}

impl<'a> Execute for Piuparts<'a> {
    /// Executes the piuparts command with the configured options.
    ///
    /// Returns an error if the .deb file is not set or if the command fails.
    fn execute(&self) -> Result<()> {
        let args = self.build_args();

        let deb_file = self.deb_file.ok_or_else(|| eyre::eyre!("No .deb file specified"))?;

        info!(
            "Running: sudo -S piuparts {} {:?}",
            args.join(" "),
            deb_file
        );

        execute_command_with_sudo("piuparts", args, self.deb_path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_piuparts() {
        let piuparts = Piuparts::new();
        assert_eq!(piuparts.build_args(), Vec::<String>::new());
    }

    #[test]
    fn test_distribution_option() {
        let piuparts = Piuparts::new().distribution("bookworm");
        assert_eq!(piuparts.build_args(), vec!["-d", "bookworm"]);
    }

    #[test]
    fn test_mirror_option() {
        let piuparts = Piuparts::new().mirror("http://deb.debian.org/debian");
        assert_eq!(
            piuparts.build_args(),
            vec!["-m", "http://deb.debian.org/debian"]
        );
    }

    #[test]
    fn test_multiple_options() {
        let piuparts = Piuparts::new()
            .distribution("jammy")
            .mirror("http://archive.ubuntu.com/ubuntu")
            .verbose()
            .bindmount_dev();

        let args = piuparts.build_args();
        assert!(args.contains(&"-d".to_string()));
        assert!(args.contains(&"jammy".to_string()));
        assert!(args.contains(&"-m".to_string()));
        assert!(args.contains(&"http://archive.ubuntu.com/ubuntu".to_string()));
        assert!(args.contains(&"--verbose".to_string()));
        assert!(args.contains(&"--bindmount=/dev".to_string()));
    }

    #[test]
    fn test_dotnet_env_bookworm() {
        let piuparts = Piuparts::new().with_dotnet_env(true, "bookworm");
        let args = piuparts.build_args();
        
        assert!(args.contains(&"--extra-repo=deb https://packages.microsoft.com/debian/12/prod bookworm main".to_string()));
        assert!(args.contains(&"--do-not-verify-signatures".to_string()));
    }

    #[test]
    fn test_dotnet_env_no_effect() {
        let piuparts = Piuparts::new().with_dotnet_env(true, "bullseye");
        assert_eq!(piuparts.build_args(), Vec::<String>::new());
        
        let piuparts = Piuparts::new().with_dotnet_env(false, "bookworm");
        assert_eq!(piuparts.build_args(), Vec::<String>::new());
    }

    #[test]
    fn test_execute_missing_deb_file() {
        let piuparts = Piuparts::new();
        let result = piuparts.execute();
        assert!(result.is_err());
    }

    #[test]
    fn test_deb_file_path() {
        let deb_path = Path::new("/tmp/package.deb");
        let piuparts = Piuparts::new().deb_file(&deb_path);
        assert_eq!(piuparts.deb_file, Some(deb_path));
    }
}