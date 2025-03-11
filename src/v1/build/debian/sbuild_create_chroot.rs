use std::path::Path;
use eyre::Result;

use super::execute::{execute_command, Execute};

pub struct SbuildCreateChroot {
    args: Vec<String>,
}

impl SbuildCreateChroot {
    pub fn new() -> Self {
        Self { args: Vec::new() }
    }

    pub fn chroot_mode(mut self, mode: &str) -> Self {
        self.args.push(format!("--chroot-mode={}", mode));
        self
    }

    pub fn make_tarball(mut self) -> Self {
        self.args.push("--make-sbuild-tarball".to_string());
        self
    }

    pub fn cache_file(mut self, path: &str) -> Self {
        self.args.push(path.to_string());
        self
    }

    pub fn codename(mut self, name: &str) -> Self {
        self.args.push(name.to_string());
        self
    }

    pub fn temp_dir(mut self, dir: &Path) -> Self {
        self.args.push(dir.to_string_lossy().to_string());
        self
    }

    pub fn repo_url(mut self, url: &str) -> Self {
        self.args.push(url.to_string());
        self
    }
}

impl Execute for SbuildCreateChroot {
    fn execute(&self) -> Result<()> {
        execute_command("sbuild-createchroot", &self.args, None)?;
        Ok(())
    }
}
