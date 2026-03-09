use std::path::Path;

use config::build_env::Distribution;
use log::info;

use crate::command::run_command;
use crate::ToolError;

/// Sbuild command wrapper.
pub struct Sbuild<'a> {
    pub distribution: &'a Distribution,
    pub cache_file: &'a Path,
    pub build_dir: &'a Path,
    pub chroot_setup_commands: &'a [String],
    pub lintian: bool,
}

impl Sbuild<'_> {
    pub fn run(&self) -> Result<(), ToolError> {
        let args = self.build_args();
        info!("Running: sbuild {}", args.join(" "));
        run_command("sbuild", &args, Some(self.build_dir))
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        args.push("-d".to_string());
        args.push(self.distribution.as_short().to_string());

        args.push("-A".to_string());
        args.push("-s".to_string());
        args.push("--source-only-changes".to_string());

        args.push("-c".to_string());
        args.push(self.cache_file.display().to_string());

        args.push("-v".to_string());
        args.push("--chroot-mode=unshare".to_string());

        args.extend(self.chroot_setup_commands.iter().cloned());

        args.push("--no-run-piuparts".to_string());
        args.push("--no-apt-upgrade".to_string());
        args.push("--no-apt-distupgrade".to_string());

        if self.lintian {
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

        args.push("--no-run-autopkgtest".to_string());

        args
    }
}

/// SbuildCreateChroot command wrapper.
pub struct SbuildCreateChroot<'a> {
    pub distribution: &'a Distribution,
    pub cache_file: &'a Path,
    pub temp_dir: &'a Path,
    pub repo_url: &'a str,
    pub snapshot: bool,
}

impl SbuildCreateChroot<'_> {
    pub fn run(&self) -> Result<(), ToolError> {
        let args = self.build_args();
        info!("Running: sbuild-createchroot {}", args.join(" "));
        run_command("sbuild-createchroot", &args, None)
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = vec![
            "--chroot-mode=unshare".to_string(),
            "--make-sbuild-tarball".to_string(),
            self.cache_file.display().to_string(),
        ];

        if self.snapshot {
            // Snapshot archives have expired Valid-Until; tell debootstrap to ignore it
            args.push("--debootstrapopts=--no-check-gpg".to_string());
        }

        args.push(self.distribution.as_short().to_string());
        args.push(self.temp_dir.display().to_string());
        args.push(self.repo_url.to_string());

        args
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_sbuild_build_args_without_lintian() {
        let dist = Distribution::bookworm();
        let sbuild = Sbuild {
            distribution: &dist,
            cache_file: Path::new("/cache/bookworm-amd64.tar.gz"),
            build_dir: Path::new("/build/hello-1.0.0"),
            chroot_setup_commands: &[],
            lintian: false,
        };

        let args = sbuild.build_args();
        assert!(args.contains(&"-d".to_string()));
        assert!(args.contains(&"bookworm".to_string()));
        assert!(args.contains(&"-A".to_string()));
        assert!(args.contains(&"-s".to_string()));
        assert!(args.contains(&"--chroot-mode=unshare".to_string()));
        assert!(args.contains(&"--no-run-lintian".to_string()));
        assert!(args.contains(&"--no-run-piuparts".to_string()));
        assert!(args.contains(&"--no-run-autopkgtest".to_string()));
        assert!(!args.contains(&"--run-lintian".to_string()));
    }

    #[test]
    fn test_sbuild_build_args_with_lintian() {
        let dist = Distribution::bookworm();
        let sbuild = Sbuild {
            distribution: &dist,
            cache_file: Path::new("/cache/bookworm-amd64.tar.gz"),
            build_dir: Path::new("/build/hello-1.0.0"),
            chroot_setup_commands: &[],
            lintian: true,
        };

        let args = sbuild.build_args();
        assert!(args.contains(&"--run-lintian".to_string()));
        assert!(!args.contains(&"--no-run-lintian".to_string()));
    }

    #[test]
    fn test_sbuild_build_args_with_chroot_commands() {
        let dist = Distribution::noble();
        let cmds = vec![
            "--chroot-setup-commands=apt install -y wget".to_string(),
            "--chroot-setup-commands=apt update".to_string(),
        ];
        let sbuild = Sbuild {
            distribution: &dist,
            cache_file: Path::new("/cache/noble-amd64.tar.gz"),
            build_dir: Path::new("/build"),
            chroot_setup_commands: &cmds,
            lintian: false,
        };

        let args = sbuild.build_args();
        assert!(args.contains(&cmds[0]));
        assert!(args.contains(&cmds[1]));
        assert!(args.contains(&"noble".to_string()));
    }

    #[test]
    fn test_sbuild_create_chroot_build_args() {
        let dist = Distribution::bookworm();
        let chroot = SbuildCreateChroot {
            distribution: &dist,
            cache_file: Path::new("/cache/bookworm-amd64.tar.gz"),
            temp_dir: Path::new("/tmp/temp_12345"),
            repo_url: "http://deb.debian.org/debian",
            snapshot: false,
        };

        let args = chroot.build_args();
        assert!(args.contains(&"--chroot-mode=unshare".to_string()));
        assert!(args.contains(&"--make-sbuild-tarball".to_string()));
        assert!(args.contains(&"/cache/bookworm-amd64.tar.gz".to_string()));
        assert!(args.contains(&"bookworm".to_string()));
        assert!(args.contains(&"/tmp/temp_12345".to_string()));
        assert!(args.contains(&"http://deb.debian.org/debian".to_string()));
        assert!(!args.iter().any(|a| a.contains("no-check-gpg")));
    }

    #[test]
    fn test_sbuild_create_chroot_snapshot() {
        let dist = Distribution::bookworm();
        let chroot = SbuildCreateChroot {
            distribution: &dist,
            cache_file: Path::new("/cache/bookworm-amd64-20250101T000000Z.tar.gz"),
            temp_dir: Path::new("/tmp/temp_12345"),
            repo_url: "http://snapshot.debian.org/archive/debian/20250101T000000Z/",
            snapshot: true,
        };

        let args = chroot.build_args();
        assert!(args.contains(
            &"http://snapshot.debian.org/archive/debian/20250101T000000Z/".to_string()
        ));
        assert!(args.iter().any(|a| a.contains("no-check-gpg")));
    }
}
