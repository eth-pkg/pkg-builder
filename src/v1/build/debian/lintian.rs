use std::path::Path;

use log::info;
use eyre::Result;

use super::execute::{execute_command, Execute};

pub struct Lintian {
    args: Vec<String>,
}

impl Lintian {
    pub fn new() -> Self {
        Self { args: Vec::new() }
    }

    pub fn suppress_tag(mut self, tag: &str) -> Self {
        self.args.push("--suppress-tags".to_string());
        self.args.push(tag.to_string());
        self
    }

    pub fn info(mut self) -> Self {
        self.args.push("-i".to_string());
        self
    }

    pub fn extended_info(mut self) -> Self {
        self.args.push("--I".to_string());
        self
    }

    pub fn changes_file<T: AsRef<Path>>(mut self, file: T) -> Self {
        self.args.push(format!("{:?}", file.as_ref()));
        self
    }

    pub fn tag_display_limit(mut self, limit: u32) -> Self {
        self.args.push(format!("--tag-display-limit={}", limit));
        self
    }

    pub fn fail_on_warning(mut self) -> Self {
        self.args.push("--fail-on=warning".to_string());
        self
    }

    pub fn fail_on_error(mut self) -> Self {
        self.args.push("--fail-on=error".to_string());
        self
    }

    pub fn with_codename(mut self, codename: &str) -> Self {
        if codename == "jammy" || codename == "noble" {
            self.args.push("--suppress-tags".to_string());
            self.args.push("malformed-deb-archive".to_string());
        }
        self
    }
}

impl Execute for Lintian {
    fn execute(&self) -> Result<()> {
        info!("Running: lintian {:?}", &self.args);

        execute_command("lintian", &self.args, None)?;
        Ok(())
    }
}