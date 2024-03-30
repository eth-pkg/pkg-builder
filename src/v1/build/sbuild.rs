use crate::v1::distribution::debian::bookworm_config_builder::BookwormPackagerConfig;
use crate::v1::packager::BackendBuildEnv;
use log::info;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::{fs, io};
use thiserror::Error;

pub struct Sbuild {
    config: BookwormPackagerConfig,
}
#[derive(Debug, Error)]
pub enum Error {
    #[error("Clean failed: {0}")]
    Clean(#[from] io::Error),
    #[error("Cannot run this command as non-root: {0}")]
    PriviligeNotRoot(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
    #[error("Failed to create build env: {0}")]
    CreateBuildEnvFailure(String),
    #[error("Failed to create build env: {0}")]
    FailedToExecute(String),
    #[error("Failed to create build env: {0}")]
    FailedToBuildPackage(String),
    #[error("Failed to create build env: {0}")]
    FailedToLogOutput(String),
}
impl Sbuild {
    pub fn new(config: BookwormPackagerConfig) -> Sbuild {
        Sbuild { config }
    }

    fn get_build_name(&self) -> String {
        format!(
            "{}-{}-{}",
            self.config.build_env().codename(),
            self.config.build_env().arch(),
            self.config
                .lang_env()
                .map_or("empty-env".to_string(), |v| v.to_string())
        )
    }
}

impl BackendBuildEnv for Sbuild {
    type Error = Error;
    fn clean(&self) -> Result<(), Error> {
        check_if_root()?;

        let build_prefix = self.get_build_name();
        info!(
            "Cleaning up sbuild directories with prefix: {}",
            build_prefix
        );

        remove_dir_recursive(&format!("/etc/sbuild/chroot/{}", build_prefix))?;
        remove_dir_recursive(&format!("/etc/schroot/chroot.d/{}*", build_prefix))?;
        remove_dir_recursive(&format!("/srv/chroot/{}", build_prefix))?;

        Ok(())
    }

    fn create(&self) -> Result<(), Error> {
        let build_prefix = self.get_build_name();

        check_if_root()?;

        let create_result = Command::new("sbuild-createchroot")
            .arg("--merged-usr")
            .arg("--chroot-prefix")
            .arg(&build_prefix)
            .arg(self.config.build_env().codename())
            .arg(&format!("/srv/chroot/{}", &build_prefix))
            .arg("http://deb.debian.org/debian")
            .status();

        if let Err(err) = create_result {
            return Err(Error::CreateBuildEnvFailure(format!("Failed to create new chroot: {}", err)));
        }

        Ok(())
    }
    fn build(&self) -> Result<(), Error> {
        let sbuild_command = format!(
            "sbuild -c {} -d {}",
            self.get_build_name(),
            self.config.build_env().codename(),
        );
        info!("Building package by invoking: {}", sbuild_command);
        let mut cmd_args = vec![
            "-c".to_string(),
            format!(
                "{}-{}-sbuild",
                self.get_build_name(),
                self.config.build_env().arch()
            ),
            "-d".to_string(),
            self.config.build_env().codename().to_string(),
        ];

        if !self.config.build_env().run_lintian() {
            cmd_args.push("--no-run-lintian".to_string());
        }
        if !self.config.build_env().run_autopkgtest() {
            cmd_args.push("--no-run-autopkgtest".to_string());
        }
        if !self.config.build_env().run_piuparts() {
            cmd_args.push("--no-run-piuparts".to_string());
        }

        let mut child = Command::new("sbuild")
            .current_dir(self.config.build_files_dir())
            .args(&cmd_args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|err| Error::FailedToExecute(err.to_string()))?;

        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);

            for line in reader.lines() {
                let line = line.map_err(|err| Error::FailedToLogOutput(err.to_string()))?;
                println!("{}", line);
            }
        }
        io::stdout()
            .flush()
            .map_err(|err| Error::FailedToLogOutput(err.to_string()))?;

        child
            .wait()
            .map_err(|err| Error::FailedToBuildPackage(err.to_string()))?;

        Ok(())
    }
}

fn check_if_root() -> Result<(), Error> {
    if let Ok(user) = std::env::var("USER") {
        if user == "root" {
            Ok(())
        } else {
            Err(Error::PriviligeNotRoot(
                "This program was not invoked with sudo.".to_string(),
            ))
        }
    } else {
        Err(Error::Unknown(
            "The USER environment variable is not set.".to_string(),
        ))
    }
}

fn remove_dir_recursive(dir_path: &str) -> Result<(), io::Error> {
    if Path::new(dir_path).exists() {
        fs::remove_dir_all(dir_path)?;
        info!("Removed directory: {}", dir_path);
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use env_logger::Env;
    use std::sync::Once;
    static INIT: Once = Once::new();

    // Set up logging for tests
    fn setup() {
        INIT.call_once(|| {
            env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
        });
    }
    #[test]
    fn test_clean_sbuild_env() {
        setup();
        // let build_env = Sbuild::new(
        //     BuildConfig::new("bookworm", "", None, &"".to_string()),
        //     SbuildBuildOptions::default(),
        // );
        unreachable!("Test case not implemented yet");
    }

    #[test]
    fn test_create_sbuild_env() {
        setup();

        unreachable!("Test case not implemented yet");
    }

    #[test]
    fn test_build_virtual_package_in_sbuild_env() {
        setup();

        unreachable!("Test case not implemented yet");
    }
    #[test]
    fn test_build_rust_package_in_sbuild_env() {
        setup();

        unreachable!("Test case not implemented yet");
    }
    #[test]
    fn test_build_go_package_in_sbuild_env() {
        setup();

        unreachable!("Test case not implemented yet");
    }

    #[test]
    fn test_build_javascript_package_in_sbuild_env() {
        setup();

        unreachable!("Test case not implemented yet");
    }

    #[test]
    fn test_build_java_package_in_sbuild_env() {
        setup();

        unreachable!("Test case not implemented yet");
    }

    #[test]
    fn test_build_csharp_package_in_sbuild_env() {
        setup();

        unreachable!("Test case not implemented yet");
    }

    #[test]
    fn test_build_typescript_package_in_sbuild_env() {
        setup();

        unreachable!("Test case not implemented yet");
    }

    #[test]
    fn test_build_zig_package_in_sbuild_env() {
        setup();
        unreachable!("Test case not implemented yet");
    }
}
