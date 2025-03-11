use super::execute::{execute_command_with_sudo, Execute};
use eyre::Result;
use log::info;
use std::path::Path;

pub struct Piuparts<'a> {
    args: Vec<String>,
    deb_file: Option<&'a Path>,
    deb_path: Option<&'a Path>,
}

impl<'a> Piuparts<'a> {
    pub fn new() -> Self {
        Self {
            args: Vec::new(),
            deb_file: None,
            deb_path: None,
        }
    }

    pub fn distribution(mut self, codename: &str) -> Self {
        self.args.push("-d".to_string());
        self.args.push(codename.to_string());
        self
    }

    pub fn mirror(mut self, url: &str) -> Self {
        self.args.push("-m".to_string());
        self.args.push(url.to_string());
        self
    }

    pub fn bindmount_dev(mut self) -> Self {
        self.args.push("--bindmount=/dev".to_string());
        self
    }

    pub fn keyring(mut self, keyring: &str) -> Self {
        self.args.push(format!("--keyring={}", keyring));
        self
    }

    pub fn verbose(mut self) -> Self {
        self.args.push("--verbose".to_string());
        self
    }

    pub fn extra_repo(mut self, repo: &str) -> Self {
        self.args.push(format!("--extra-repo={}", repo));
        self
    }

    pub fn no_verify_signatures(mut self) -> Self {
        self.args.push("--do-not-verify-signatures".to_string());
        self
    }

    pub fn with_dotnet_env(self, is_dotnet: bool, codename: &str) -> Self {
        if is_dotnet && (codename == "bookworm" || codename == "jammy jellyfish") {
            let repo = format!(
                "--extra-repo=deb https://packages.microsoft.com/debian/12/prod {} main",
                codename
            );
            return self.extra_repo(&repo).no_verify_signatures();
        }
        self
    }

    pub fn deb_file(mut self, deb_file: &'a Path) -> Self {
        self.deb_file = Some(deb_file);
        self
    }

    pub fn deb_path(mut self, deb_path: &'a Path) -> Self {
        self.deb_path = Some(deb_path);
        self
    }
}

impl<'a> Execute for Piuparts<'a> {
    fn execute(&self) -> Result<()> {
        info!(
            "Running: sudo -S piuparts {} {:?}",
            &self.args.join(" "),
            &self.deb_file.as_ref().unwrap()
        );
        execute_command_with_sudo(
            "piuparts",
            self.args.clone(),
            self.deb_file.unwrap(),
            self.deb_path,
        )?;
        Ok(())
    }
}
