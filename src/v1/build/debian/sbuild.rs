use std::path::{Path, PathBuf};
use eyre::Result;
use log::info;

use super::execute::{execute_command, Execute};

pub struct SbuildBuilder {
    args: Vec<String>,
    dir: Option<PathBuf>,
}

impl SbuildBuilder {
    pub fn new() -> Self {
        Self {
            args: Vec::new(),
            dir: None,
        }
    }

    pub fn distribution(mut self, codename: &str) -> Self {
        self.args.push("-d".to_string());
        self.args.push(codename.to_string());
        self
    }

    pub fn build_arch_all(mut self) -> Self {
        self.args.push("-A".to_string());
        self
    }

    pub fn build_source(mut self) -> Self {
        self.args.push("-s".to_string());
        self.args.push("--source-only-changes".to_string());
        self
    }

    pub fn cache_file(mut self, cache_file: &str) -> Self {
        self.args.push("-c".to_string());
        self.args.push(cache_file.to_string());
        self
    }

    pub fn verbose(mut self) -> Self {
        self.args.push("-v".to_string());
        self
    }

    pub fn chroot_mode_unshare(mut self) -> Self {
        self.args.push("--chroot-mode=unshare".to_string());
        self
    }

    pub fn setup_commands(mut self, commands: &[String]) -> Self {
        self.args.extend(commands.iter().map(|s| s.to_string()));
        self
    }

    pub fn no_run_piuparts(mut self) -> Self {
        self.args.push("--no-run-piuparts".to_string());
        self
    }

    pub fn no_apt_upgrades(mut self) -> Self {
        self.args.push("--no-apt-upgrade".to_string());
        self.args.push("--no-apt-distupgrade".to_string());
        self
    }

    pub fn run_lintian(mut self, enabled: bool) -> Self {
        if enabled {
            self.args.extend([
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
            self.args.push("--no-run-lintian".to_string());
        }
        self
    }

    pub fn no_run_autopkgtest(mut self) -> Self {
        self.args.push("--no-run-autopkgtest".to_string());
        self
    }

    pub fn working_dir(mut self, dir: &Path) -> Self {
        self.dir = Some(dir.to_path_buf());
        self
    }
}

impl Execute for SbuildBuilder {
    fn execute(&self) -> Result<()> {
        info!("Building package with: sbuild {}", &self.args.join(" "));
        execute_command("sbuild", &self.args, self.dir.as_deref())?;
        Ok(())
    }
}