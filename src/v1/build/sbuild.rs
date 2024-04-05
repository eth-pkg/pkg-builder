use crate::v1::distribution::debian::bookworm_config_builder::BookwormPackagerConfig;
use crate::v1::packager::{BackendBuildEnv, LanguageEnv};
use glob::glob;
use log::info;
use std::io::{BufRead, BufReader, Write};
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

        let result: Result<(), Error> = match self.config.lang_env() {
            None => {
                // add nothing
                Ok(())
            }
            Some(lang_env) => {
                setup_chroot("rm -rf /tmp/* && chmod 777 /tmp")?;

                let additional_build_deps_for_langs =
                    "curl ca-certificates wget apt-transport-https gnupg apt-utils mlocate";
                setup_chroot(
                    format!("apt install -y {}", additional_build_deps_for_langs).as_str(),
                )?;
                match lang_env {
                    LanguageEnv::Rust => {
                        let rust_version = "1.76.0";
                        let install_rust = vec![
                            format!("cd /tmp && curl -o rust.tar.xz -L https://static.rust-lang.org/dist/rust-{}-x86_64-unknown-linux-gnu.tar.xz", rust_version),
                            "cd /tmp && tar xvJf rust.tar.xz -C . --strip-components=1 --exclude=rust-docs".to_string(),
                            "cd /tmp && /bin/bash install.sh --without=rust-docs".to_string()
                        ];

                        for action in install_rust.iter() {
                            setup_chroot(action)?;
                        }
                    }
                    LanguageEnv::Go => {
                        let go_version = "1.22.2";
                        let install = vec![
                            format!("cd /tmp && curl -o go.tar.gz -L https://go.dev/dl/go{}.linux-amd64.tar.gz", go_version),
                            "cd /tmp && rm -rf /usr/local/go && mkdir /usr/local/go && tar -C /usr/local -xzf go.tar.gz".to_string(),
                            "rm /etc/profile.d/go.sh && touch /etc/profile.d/go.sh".to_string(),
                            "echo 'export GOROOT=/usr/local/go' > /etc/profile.d/go.sh".to_string(),
                            "echo 'export PATH=$PATH:$GOROOT/bin' >> /etc/profile.d/go.sh".to_string(),
                            "source /etc/profile".to_string(),
                            "source /etc/profile && go version".to_string(),
                            "go version".to_string(),
                        ];

                        for action in install.iter() {
                            setup_chroot(action)?;
                        }
                    }
                    LanguageEnv::JavaScript | LanguageEnv::TypeScript => {
                        let is_yarn = true;
                        let yarn_version = "1.22.10";
                        // let node_version = "20.x";
                        // from nodesource
                        // TODO switch from nodesource to actual binary without repository
                        let mut install = vec![
                            "curl -fsSL https://deb.nodesource.com/setup_lts.x | bash - && apt-get install -y nodejs".to_string(),
                            "node --version".to_string(),
                            "npm --version".to_string(),
                        ];
                        if is_yarn {
                            install.push(format!("npm install --global yarn@{}", yarn_version));
                            install.push("yarn --version".to_string());
                        }
                        for action in install.iter() {
                            setup_chroot(action)?;
                        }
                    }
                    LanguageEnv::Java => {
                        let is_oracle = true;
                        if is_oracle {
                            // TODO proc mount is not nice
                            let jdk_version = "17";
                            let install = vec![
                                format!("rm -rf /opt/lib/jvm/jdk-{version} && rm -rf /usr/lib/jvm/jdk-{version}", version=jdk_version),
                                format!("mkdir -p /opt/lib/jvm/jdk-{version}-oracle && mkdir -p /usr/lib/jvm", version=jdk_version),
                                format!("cd /tmp && wget https://download.oracle.com/java/{version}/latest/jdk-{version}_linux-x64_bin.tar.gz", version=jdk_version),
                                format!("cd /tmp && tar -zxf jdk-{version}_linux-x64_bin.tar.gz -C /opt/lib/jvm/jdk-{version}-oracle --strip-components=1", version=jdk_version),
                                format!("ln -s /opt/lib/jvm/jdk-{version}-oracle  /usr/lib/jvm/jdk-{version}", version=jdk_version),
                                "rm /etc/profile.d/jdk.sh".to_string(),
                                format!("echo 'export JAVA_HOME=/usr/lib/jvm/jdk-{}' >> /etc/profile.d/jdk.sh", jdk_version),
                                "echo 'export PATH=$PATH:$JAVA_HOME/bin' >> /etc/profile.d/jdk.sh".to_string(),
                                "mount -t proc proc /proc".to_string(),
                                "/usr/lib/jvm/jdk-17/bin/java -version".to_string(),
                                "source /etc/profile.d/jdk.sh && java -version".to_string(),
                                "umount /proc".to_string()
                            ];
                            for action in install.iter() {
                                setup_chroot(action)?;
                            }
                        }
                    }
                    LanguageEnv::CSharp => {
                        // TODO proc mount is not nice
                        let dotnet_version = "7.0";
                        let install = vec![
                            "mount -t proc proc /proc".to_string(),
                            "cd /tmp && wget https://packages.microsoft.com/config/debian/12/packages-microsoft-prod.deb -O packages-microsoft-prod.deb".to_string(),
                            "cd /tmp && dpkg -i packages-microsoft-prod.deb ".to_string(),
                            "apt-get update -y".to_string(),
                            format!("apt-get install -y dotnet-sdk-{}", dotnet_version),
                            "dotnet --version".to_string(),
                            "umount /proc".to_string()
                        ];
                        for action in install.iter() {
                            setup_chroot(action)?;
                        }
                    }
                    LanguageEnv::Nim => {
                        let nim_version = "2.0.2";
                        let install = vec![
                            format!("rm -rf /tmp/nim-{version} && rm -rf /usr/lib/nim/nim-{version}&& rm -rf /opt/lib/nim/nim-{version} && mkdir /tmp/nim-{version}", version = nim_version),
                            "mkdir -p /opt/lib/nim && mkdir -p /usr/lib/nim".to_string(),
                            format!("cd /tmp && wget https://nim-lang.org/download/nim-{}-linux_x64.tar.xz", nim_version),
                            format!("cd /tmp && tar xJf nim-{version}-linux_x64.tar.xz -C nim-{version} --strip-components=1", version=nim_version),
                            format!("cd /tmp  && mv nim-{version} /opt/lib/nim", version=nim_version),
                            format!("ln -s /opt/lib/nim/nim-{version}  /usr/lib/nim/nim-{version}", version=nim_version),
                            "rm /etc/profile.d/nim.sh".to_string(),
                            format!("echo 'export NIM_HOME=/usr/lib/nim/nim-{}' >> /etc/profile.d/nim.sh", nim_version),
                            "echo 'export PATH=$PATH:$NIM_HOME/bin' >> /etc/profile.d/nim.sh".to_string(),
                            "source /etc/profile.d/nim.sh && nim --version".to_string(),
                        ];
                        for action in install.iter() {
                            setup_chroot(action)?;
                        }
                    }
                }
                // let is_docker_needed_for_tests = true;
                // if is_docker_needed_for_tests {
                //     // Note this doesn't install docker, please put into that into build_depends
                //     let install = vec![
                //         "apt install -y gnupg".to_string(),
                //         "install -m 0755 -d /etc/apt/keyrings".to_string(),
                //         "curl -fsSL https://download.docker.com/linux/debian/gpg -o /etc/apt/keyrings/docker.asc".to_string(),
                //         "chmod a+r /etc/apt/keyrings/docker.asc".to_string(),
                //         "echo deb [arch=amd64 signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/debian bookworm stable | \
                //                 tee /etc/apt/sources.list.d/docker.list > /dev/null".to_string(),
                //         "apt-get update".to_string(),
                //         "apt-get remove gnupg".to_string()
                //     ];
                //     for action in install.iter() {
                //         setup_chroot(action)?;
                //     }
                // }
                setup_chroot(
                    format!("apt remove -y {}", additional_build_deps_for_langs).as_str(),
                )?;
                Ok(())
            }
        };
        result
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
        println!(
            "Building package by invoking: sbuild {}",
            cmd_args.join(" ")
        );

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

fn setup_chroot(action: &str) -> Result<(), Error> {
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
    Ok(())
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
