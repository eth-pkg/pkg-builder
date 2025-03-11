use eyre::Result;
use std::path::{Path, PathBuf};

use super::execute::{execute_command_with_sudo, Execute};

pub struct AutopkgtestImageBuilder {
    cmd: &'static str,
    args: Vec<String>,
    dir: Option<PathBuf>,
}

impl AutopkgtestImageBuilder {
    pub fn new() -> Self {
        Self {
            cmd: "autopkgtest-build-qemu", // Default, overridden as needed
            args: Vec::new(),
            dir: None,
        }
    }

    pub fn codename(mut self, codename: &str) -> Self {
        match codename {
            "bookworm" => {
                self.cmd = "autopkgtest-build-qemu";
                self.args.push(codename.to_string());
            }
            "noble numbat" | "jammy jellyfish" => {
                self.cmd = "autopkgtest-buildvm-ubuntu-cloud";
                self.args.push(format!("--release={}", codename));
            }
            _ => panic!("Unsupported codename: {}", codename), // Replace with error handling
        }
        self
    }

    pub fn image_path(mut self, cache_dir: &str, codename: &str, arch: &str) -> Self {
        let image_name = format!("autopkgtest-{}-{}.img", codename, arch);
        let cache_dir = shellexpand::tilde(cache_dir).to_string();
        let image_path = Path::new(&cache_dir).join(&image_name);
        self.args.push(format!("{:?}", &image_path));
        self.dir = Some(image_path.parent().unwrap().to_path_buf());
        self
    }

    pub fn mirror(mut self, repo_url: &str) -> Self {
        self.args.push(format!("--mirror={}", repo_url));
        self
    }

    pub fn arch(mut self, arch: &str) -> Self {
        self.args.push(format!("--arch={}", arch));
        self
    }

    // pub fn verbose(mut self) -> Self {
    //     if self.cmd == "autopkgtest-buildvm-ubuntu-cloud" {
    //         self.args.push("-v".to_string());
    //     }
    //     self
    // }

    pub fn get_image_path(&self) -> Option<PathBuf> {
        self.args.get(1).map(|p| {
            let path_str = p.trim_matches('"'); // Remove quotes from format!("{:?}")
            PathBuf::from(path_str)
        })
    }
}

impl Execute for AutopkgtestImageBuilder {
    fn execute(&self) -> Result<()> {
        execute_command_with_sudo(
            self.cmd,
            self.args.clone(),
            self.get_image_path().unwrap().as_path(),
            self.dir.as_deref(),
        )?;
        Ok(())
    }
}
