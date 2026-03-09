use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

use config::build_env::{Architecture, Distribution};
use log::info;

use crate::command::run_command_sudo;
use crate::ToolError;

/// Autopkgtest command wrapper.
pub struct Autopkgtest<'a> {
    pub changes_file: &'a Path,
    pub image_path: &'a Path,
    pub deb_dir: &'a Path,
    pub setup_commands: &'a [String],
}

impl Autopkgtest<'_> {
    pub fn run(&self) -> Result<(), ToolError> {
        let args = self.build_args();
        info!("Running: autopkgtest {}", args.join(" "));
        run_command_sudo("autopkgtest", &args, Some(self.deb_dir))
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        args.push(self.changes_file.display().to_string());
        args.push("--no-built-binaries".to_string());
        args.push("--apt-upgrade".to_string());

        for cmd in self.setup_commands {
            args.push(format!("--setup-commands={}", cmd));
        }

        args.push("--".to_string());
        args.push("qemu".to_string());
        args.push(self.image_path.display().to_string());

        args
    }
}

/// Build an autopkgtest QEMU image if it doesn't exist.
pub fn ensure_autopkgtest_image(
    cache_dir: &Path,
    distribution: &Distribution,
    arch: &Architecture,
    repo_url: &str,
) -> Result<PathBuf, ToolError> {
    let image_name = format!("autopkgtest-{}-{}.img", distribution.as_short(), arch);
    let cache_dir_expanded = shellexpand::tilde(&cache_dir.display().to_string()).to_string();
    let image_path = Path::new(&cache_dir_expanded).join(&image_name);

    if image_path.exists() {
        return Ok(image_path);
    }

    let parent = image_path.parent().unwrap_or(Path::new(""));
    create_dir_all(parent)?;

    let cmd = match distribution {
        Distribution::Debian(_) => "autopkgtest-build-qemu",
        Distribution::Ubuntu(_) => "autopkgtest-buildvm-ubuntu-cloud",
    };

    let mut args = Vec::new();

    match distribution {
        Distribution::Debian(_) => {
            args.push(distribution.as_short().to_string());
        }
        Distribution::Ubuntu(_) => {
            args.push(format!("--release={}", distribution.as_short()));
        }
    }

    args.push(format!("--mirror={}", repo_url));
    args.push(format!("--arch={}", arch));

    match distribution {
        Distribution::Ubuntu(_) => {
            args.push("-v".to_string());
            args.push(format!("--timeout={}", 3600));
            args.push(format!("--ram-size={}", 2048));
        }
        Distribution::Debian(_) => {
            args.push(image_path.display().to_string());
        }
    }

    info!("Running: sudo -S {} {}", cmd, args.join(" "));

    // Try once, retry on failure (workaround for flaky Ubuntu runners)
    let result = run_command_sudo(cmd, &args, Some(parent));
    if result.is_err() {
        run_command_sudo(cmd, &args, Some(parent))?;
    }

    Ok(image_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_autopkgtest_build_args() {
        let autopkgtest = Autopkgtest {
            changes_file: Path::new("/build/hello_1.0.0-1_amd64.changes"),
            image_path: Path::new("/cache/autopkgtest-bookworm-amd64.img"),
            deb_dir: Path::new("/build"),
            setup_commands: &[],
        };

        let args = autopkgtest.build_args();
        assert_eq!(args[0], "/build/hello_1.0.0-1_amd64.changes");
        assert!(args.contains(&"--no-built-binaries".to_string()));
        assert!(args.contains(&"--apt-upgrade".to_string()));
        assert!(args.contains(&"--".to_string()));
        assert!(args.contains(&"qemu".to_string()));
        assert!(args.iter().any(|a| a.contains("autopkgtest-bookworm")));
    }

    #[test]
    fn test_autopkgtest_build_args_with_setup_commands() {
        let cmds = vec![
            "apt install -y wget".to_string(),
        ];
        let autopkgtest = Autopkgtest {
            changes_file: Path::new("/build/test.changes"),
            image_path: Path::new("/cache/test.img"),
            deb_dir: Path::new("/build"),
            setup_commands: &cmds,
        };

        let args = autopkgtest.build_args();
        assert!(args.iter().any(|a| a.contains("apt install -y wget")));
    }

    #[test]
    fn test_ensure_autopkgtest_image_path_construction() {
        // Just test the image name format without actually running the command
        let dist = Distribution::bookworm();
        let arch = Architecture::Amd64;
        let expected_name = format!("autopkgtest-{}-{}.img", dist.as_short(), arch);
        assert_eq!(expected_name, "autopkgtest-bookworm-amd64.img");
    }
}
