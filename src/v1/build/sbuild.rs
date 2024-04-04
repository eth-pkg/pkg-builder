use crate::v1::distribution::debian::bookworm_config_builder::BookwormPackagerConfig;
use crate::v1::packager::{BackendBuildEnv, LanguageEnv};
use glob::glob;
use log::info;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;
use std::{fs, io};
use std::fmt::format;
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
    #[error("Please create build_env first by running the following: sudo pkg-builder build-env <CONFIG_FILE>")]
    BuildEnvMissing,
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
    fn get_build_directories(&self) -> Vec<(String, bool)> {
        let build_prefix = self.get_build_name();
        let schroot = format!(
            "/etc/schroot/chroot.d/{}-{}-sbuild-*",
            build_prefix,
            self.config.build_env().arch()
        );
        let schroot_pattern = match glob(&schroot) {
            Ok(pattern) => Some(pattern),
            Err(_) => None, // keep wrong string
        }
        .unwrap();
        let mut directories = vec![
            (
                format!(
                    "/etc/sbuild/chroot/{}-{}-sbuild",
                    build_prefix,
                    self.config.build_env().arch()
                ),
                false, // File
            ),
            (
                format!("/srv/chroot/{}", build_prefix),
                true, // Directory
            ),
        ];
        for file_path in schroot_pattern.into_iter() {
            let glob_pattern = file_path.unwrap();
            let file_path = glob_pattern.to_str().unwrap();
            directories.push((file_path.to_string(), false));
        }
        directories
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

        let directories = self.get_build_directories();
        for (path, is_directory) in directories.iter() {
            let _ = remove_file_or_directory(path, *is_directory);
        }

        Ok(())
    }

    fn create(&self) -> Result<(), Error> {
        let build_prefix = self.get_build_name();

        check_if_root()?;

        let create_result = Command::new("sbuild-createchroot")
            .arg("--merged-usr")
            .arg("--include=ca-certificates curl")
            .arg("--chroot-prefix")
            .arg(&build_prefix)
            .arg(self.config.build_env().codename())
            .arg(&format!("/srv/chroot/{}", &build_prefix))
            .arg("http://deb.debian.org/debian")
            .status();

        if let Err(err) = create_result {
            return Err(Error::CreateBuildEnvFailure(format!(
                "Failed to create new chroot: {}",
                err
            )));
        }

        match self.config.lang_env() {
            None => {
                // add nothing
            }
            Some(lang_env) => match lang_env {
                LanguageEnv::Rust => {
                    let rust_version = "1.76.0";
                    let install_rust = vec![
                        format!("cd /tmp && curl -o rust.tar.xz -L https://static.rust-lang.org/dist/rust-{}-x86_64-unknown-linux-gnu.tar.xz", rust_version),
                        "cd /tmp && tar xvJf rust.tar.xz -C . --strip-components=1 --exclude=rust-docs".to_string(),
                        "cd /tmp && /bin/bash install.sh --without=rust-docs".to_string()

                    ];

                    for action in install_rust.iter(){

                        let cmd = Command::new("chroot")
                            .arg("/srv/chroot/bookworm-amd64-rust")
                            .arg("/bin/bash")
                            .arg("-c")
                            .arg(action)
                            .status();

                        if let Err(err) = cmd {
                            return Err(Error::CreateBuildEnvFailure(format!(
                                "Failed to install rust in env: {}",
                                err
                            )));
                        }
                    }
                }
                LanguageEnv::Go => {}
                LanguageEnv::JavaScript => {}
                LanguageEnv::Java => {}
                LanguageEnv::CSharp => {}
                LanguageEnv::TypeScript => {}
                LanguageEnv::Nim => {}
            },
        }

        let cmd = Command::new("chroot")
            .arg("/srv/chroot/bookworm-amd64-rust")
            .arg("/bin/bash")
            .arg("-c")
            .arg("apt remove -y curl ca-certificates")
            .status();


        if let Err(err) = cmd {
            return Err(Error::CreateBuildEnvFailure(format!(
                "Failed to remove ca-certificates and curl: {}",
                err
            )));
        }

        Ok(())
    }
    fn build(&self) -> Result<(), Error> {
        check_if_not_root()?;

        let directories = self.get_build_directories();
        for (path, _) in directories.iter() {
            if fs::metadata(path).is_err() {
                println!("{}", path);
                return Err(Error::BuildEnvMissing);
            }
        }


        let mut cmd_args = vec![
            "-c".to_string(),
            format!(
                "{}-{}-sbuild",
                self.get_build_name(),
                self.config.build_env().arch()
            ),
            "-d".to_string(),
            self.config.build_env().codename().to_string(),
            "-A".to_string(),                    // build_arch_all
            "-s".to_string(),                    // build source
            "--source-only-changes".to_string(), // source_only_changes
            "-v".to_string(),                    // verbose
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
        println!("Building package by invoking: sbuild {}", cmd_args.join(" "));

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
fn check_if_not_root() -> Result<(), Error> {
    if let Ok(user) = std::env::var("USER") {
        if user != "root" {
            Ok(())
        } else {
            Err(Error::PriviligeNotRoot(
                "This program was invoked with sudo.".to_string(),
            ))
        }
    } else {
        Err(Error::Unknown(
            "The USER environment variable is not set.".to_string(),
        ))
    }
}
fn remove_file_or_directory(path: &str, is_directory: bool) -> io::Result<()> {
    if is_directory {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
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
    fn test_build_nim_package_in_sbuild_env() {
        setup();
        unreachable!("Test case not implemented yet");
    }
}
