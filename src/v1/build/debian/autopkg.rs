use eyre::Result;
use std::path::Path;

pub struct Autopkgtest {
    args: Vec<String>,
    dir: Option<PathBuf>,
}

use std::path::PathBuf;

use log::info;

use super::execute::execute_command;
use super::execute::Execute;

impl Autopkgtest {
    pub fn new() -> Self {
        Self {
            args: Vec::new(),
            dir: None,
        }
    }

    pub fn changes_file(mut self, file: &str) -> Self {
        self.args.push(file.to_string());
        self
    }

    pub fn no_built_binaries(mut self) -> Self {
        self.args.push("--no-built-binaries".to_string());
        self
    }

    pub fn apt_upgrade(mut self) -> Self {
        self.args.push("--apt-upgrade".to_string());
        self
    }

    pub fn test_deps_not_in_debian(mut self, deps: &[String]) -> Self {
        self.args
            .extend(deps.iter().map(|dep| format!("--setup-commands={}", dep)));
        self
    }

    pub fn qemu(mut self, image_path: &str) -> Self {
        self.args.push("--".to_string());
        self.args.push("qemu".to_string());
        self.args.push(image_path.to_string());
        self
    }

    pub fn working_dir(mut self, dir: &Path) -> Self {
        self.dir = Some(dir.to_path_buf());
        self
    }
}

impl Execute for Autopkgtest {
    fn execute(&self) -> Result<()> {
        info!("Running: autopkgtest {}", self.args.join(" "));
        execute_command("autopkgtest", &self.args, self.dir.as_deref())?;
        Ok(())
    }
}
