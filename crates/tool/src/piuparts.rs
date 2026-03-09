use std::path::Path;

use config::build_env::{Distribution, UbuntuCodename};
use log::info;

use crate::command::run_command_sudo;
use crate::ToolError;

/// Piuparts command wrapper.
pub struct Piuparts<'a> {
    pub distribution: &'a Distribution,
    pub deb_file: &'a Path,
    pub deb_dir: &'a Path,
    pub is_dotnet: bool,
    pub repo_url: &'a str,
}

impl Piuparts<'_> {
    pub fn run(&self) -> Result<(), ToolError> {
        let args = self.build_args();
        info!("Running: sudo -S piuparts {}", args.join(" "));
        run_command_sudo("piuparts", &args, Some(self.deb_dir))
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        args.push("-d".to_string());
        args.push(self.distribution.as_short().to_string());

        args.push("-m".to_string());
        args.push(self.repo_url.to_string());

        args.push("--bindmount=/dev".to_string());
        args.push(format!("--keyring={}", self.distribution.keyring()));
        args.push("--verbose".to_string());

        // .NET environment setup
        if self.is_dotnet {
            match self.distribution {
                Distribution::Debian(debian) => {
                    let repo = format!(
                        "deb https://packages.microsoft.com/debian/12/prod {} main",
                        debian
                    );
                    args.push(format!("--extra-repo={}", repo));
                    args.push("--do-not-verify-signatures".to_string());
                }
                Distribution::Ubuntu(UbuntuCodename::Jammy) => {
                    let repo =
                        "deb https://packages.microsoft.com/debian/12/prod jammy main".to_string();
                    args.push(format!("--extra-repo={}", repo));
                    args.push("--do-not-verify-signatures".to_string());
                }
                _ => {}
            }
        }

        args.push(self.deb_file.display().to_string());

        args
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::build_env::{DebianCodename, UbuntuCodename};
    use std::path::Path;

    #[test]
    fn test_piuparts_build_args_basic() {
        let dist = Distribution::Debian(DebianCodename::Bookworm);
        let piuparts = Piuparts {
            distribution: &dist,
            deb_file: Path::new("hello_1.0.0-1_amd64.deb"),
            deb_dir: Path::new("/build"),
            is_dotnet: false,
            repo_url: dist.repo_url(),
        };

        let args = piuparts.build_args();
        assert!(args.contains(&"-d".to_string()));
        assert!(args.contains(&"bookworm".to_string()));
        assert!(args.contains(&"--bindmount=/dev".to_string()));
        assert!(args.contains(&"--verbose".to_string()));
        assert!(args.iter().any(|a| a.contains("debian-archive-keyring")));
        // No dotnet extra repo
        assert!(!args.iter().any(|a| a.contains("packages.microsoft.com")));
    }

    #[test]
    fn test_piuparts_build_args_snapshot_url() {
        let dist = Distribution::Debian(DebianCodename::Bookworm);
        let snapshot_url = "http://snapshot.debian.org/archive/debian/20250101T000000Z/";
        let piuparts = Piuparts {
            distribution: &dist,
            deb_file: Path::new("hello_1.0.0-1_amd64.deb"),
            deb_dir: Path::new("/build"),
            is_dotnet: false,
            repo_url: snapshot_url,
        };

        let args = piuparts.build_args();
        assert!(args.contains(&snapshot_url.to_string()));
    }

    #[test]
    fn test_piuparts_build_args_dotnet_debian() {
        let dist = Distribution::Debian(DebianCodename::Bookworm);
        let piuparts = Piuparts {
            distribution: &dist,
            deb_file: Path::new("hello_1.0.0-1_amd64.deb"),
            deb_dir: Path::new("/build"),
            is_dotnet: true,
            repo_url: dist.repo_url(),
        };

        let args = piuparts.build_args();
        assert!(args.iter().any(|a| a.contains("packages.microsoft.com")));
        assert!(args.contains(&"--do-not-verify-signatures".to_string()));
    }

    #[test]
    fn test_piuparts_build_args_dotnet_jammy() {
        let dist = Distribution::Ubuntu(UbuntuCodename::Jammy);
        let piuparts = Piuparts {
            distribution: &dist,
            deb_file: Path::new("hello.deb"),
            deb_dir: Path::new("/build"),
            is_dotnet: true,
            repo_url: dist.repo_url(),
        };

        let args = piuparts.build_args();
        assert!(args.iter().any(|a| a.contains("packages.microsoft.com")));
        assert!(args.contains(&"--do-not-verify-signatures".to_string()));
    }

    #[test]
    fn test_piuparts_build_args_dotnet_noble_no_extra() {
        let dist = Distribution::Ubuntu(UbuntuCodename::Noble);
        let piuparts = Piuparts {
            distribution: &dist,
            deb_file: Path::new("hello.deb"),
            deb_dir: Path::new("/build"),
            is_dotnet: true,
            repo_url: dist.repo_url(),
        };

        let args = piuparts.build_args();
        // Noble dotnet does NOT get Microsoft repo
        assert!(!args.iter().any(|a| a.contains("packages.microsoft.com")));
    }
}
