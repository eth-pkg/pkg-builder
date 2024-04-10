use crate::v1::packager::BackendBuildEnv;
use crate::v1::pkg_config::{LanguageEnv, PackageType, PkgConfig};
use eyre::{eyre, Result};
use log::info;
use rand::random;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::{env, fs, io};

pub struct Sbuild {
    config: PkgConfig,
    build_files_dir: String,
}

impl Sbuild {
    pub fn new(config: PkgConfig, build_files_dir: String) -> Sbuild {
        Sbuild {
            config,
            build_files_dir,
        }
    }

    fn get_additional_deps(&self) -> Vec<String> {
        let package_type = &self.config.package_type;
        let lang_env = match package_type {
            PackageType::Default(config) => Some(&config.language_env),
            PackageType::Git(config) => Some(&config.language_env),
            PackageType::Virtual => None,
        };
        let additional_deps = match lang_env {
            None => {
                vec![]
            }
            Some(lang_env) => {
                let mut additional_deps: Vec<String> = vec![];
                let lang_deps = match lang_env {
                    LanguageEnv::C => {
                        let lang_deps = vec![];
                        lang_deps
                    }
                    LanguageEnv::Rust(config) => {
                        let rust_version = &config.rust_version;
                        let lang_deps = vec![
                            "apt install -y curl".to_string(),
                            format!("cd /tmp && curl -o rust.tar.xz -L https://static.rust-lang.org/dist/rust-{}-x86_64-unknown-linux-gnu.tar.xz", rust_version),
                            "cd /tmp && tar xvJf rust.tar.xz -C . --strip-components=1 --exclude=rust-docs".to_string(),
                            "cd /tmp && /bin/bash install.sh --without=rust-docs".to_string(),
                            "apt remove -y curl".to_string()
                        ];
                        lang_deps
                    }
                    LanguageEnv::Go(config) => {
                        let go_version = &config.go_version;
                        let install = vec![
                            "apt install -y curl".to_string(),
                            format!("cd /tmp && curl -o go.tar.gz -L https://go.dev/dl/go{}.linux-amd64.tar.gz", go_version),
                            "cd /tmp && rm -rf /usr/local/go && mkdir /usr/local/go && tar -C /usr/local -xzf go.tar.gz".to_string(),
                            "ln -s /usr/local/go/bin/go /usr/bin/go".to_string(),
                            "go version".to_string(),
                            "apt remove -y curl".to_string(),
                        ];
                        install
                    }
                    LanguageEnv::JavaScript(config) | LanguageEnv::TypeScript(config) => {
                        // TODO node version
                        // from nodesource
                        // TODO switch from nodesource to actual binary without repository
                        let mut install = vec![
                            "curl -fsSL https://deb.nodesource.com/setup_lts.x | bash - && apt-get install -y nodejs npm".to_string(),
                            "node --version".to_string(),
                            "npm --version".to_string(),
                        ];
                        if let Some(yarn_version) = &config.yarn_version {
                            install.push(format!("npm install --global yarn@{}", yarn_version));
                            install.push("yarn --version".to_string());
                        }
                        install
                    }
                    LanguageEnv::Java(config) => {
                        let is_oracle = config.is_oracle;
                        if is_oracle {
                            let jdk_version = &config.jdk_version;
                            let install = vec![
                                "apt install -y wget".to_string(),
                                format!("mkdir -p /opt/lib/jvm/jdk-{version}-oracle && mkdir -p /usr/lib/jvm", version = jdk_version),
                                format!("cd /tmp && wget -q https://download.oracle.com/java/{version}/latest/jdk-{version}_linux-x64_bin.tar.gz", version = jdk_version),
                                format!("cd /tmp && tar -zxf jdk-{version}_linux-x64_bin.tar.gz -C /opt/lib/jvm/jdk-{version}-oracle --strip-components=1", version = jdk_version),
                                format!("ln -s /opt/lib/jvm/jdk-{version}-oracle/bin/java  /usr/bin/java", version = jdk_version),
                                format!("ln -s /opt/lib/jvm/jdk-{version}-oracle/bin/javac  /usr/bin/javac", version = jdk_version),
                                "java -version".to_string(),
                                "apt remove -y wget".to_string(),
                            ];
                            return install;
                        }
                        vec![]
                    }
                    LanguageEnv::CSharp(config) => {
                        let dotnet_version = &config.dotnet_version;
                        let install = vec![
                            "apt install -y wget".to_string(),
                            "cd /tmp && wget https://packages.microsoft.com/config/debian/12/packages-microsoft-prod.deb -O packages-microsoft-prod.deb".to_string(),
                            "cd /tmp && dpkg -i packages-microsoft-prod.deb ".to_string(),
                            "apt-get update -y".to_string(),
                            format!("apt-get install -y dotnet-sdk-{}", dotnet_version),
                            "dotnet --version".to_string(),
                            "apt remove -y wget".to_string(),
                        ];
                        install
                    }
                    LanguageEnv::Nim(config) => {
                        let nim_version = &config.nim_version;
                        let nim_binary_url = &config.nim_binary_url;
                        let nim_version_checksum = &config.nim_version_checksum;
                        let install = vec![
                            "apt install -y wget".to_string(),
                            format!("rm -rf /tmp/nim-{version} && rm -rf /usr/lib/nim/nim-{version}&& rm -rf /opt/lib/nim/nim-{version} && mkdir /tmp/nim-{version}", version = nim_version),
                            "mkdir -p /opt/lib/nim && mkdir -p /usr/lib/nim".to_string(),
                            format!("cd /tmp && wget -q {}", nim_binary_url),
                            format!("cd /tmp && echo {} >> hash_file.txt && cat hash_file.txt", nim_version_checksum),
                            "cd /tmp && sha256sum -c hash_file.txt".to_string(),
                            format!("cd /tmp && tar xJf nim-{version}-linux_x64.tar.xz -C nim-{version} --strip-components=1", version = nim_version),
                            format!("cd /tmp  && mv nim-{version} /opt/lib/nim", version = nim_version),
                            format!("ln -s /opt/lib/nim/nim-{version}/bin/nim  /usr/bin/nim", version = nim_version),
                            // equality check not working
                            //  format!("installed_version=`nim --version | head -n 1 | awk '{{print $4}}'` && echo \"installed version: $installed_version\" && [ \"$installed_version\" != \"{}\" ] && exit 1", nim_version),
                            "nim --version".to_string(),
                            "apt remove -y wget".to_string(),
                        ];
                        install
                    }
                };
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
                //     additional_deps.extend(install);
                additional_deps.extend(lang_deps);

                // additional_deps.push(format!("apt remove -y {}", additional_build_deps_for_langs));
                additional_deps
            }
        };
        additional_deps
    }

    fn get_cache_dir(&self) -> String {
        let dir = "~/.cache/sbuild/bookworm-amd64.tar.gz";
        if dir.starts_with('~') {
            let expanded_path = shellexpand::tilde(dir).to_string();
            expanded_path
        } else {
            let parent_dir = env::current_dir().unwrap();
            let dir = parent_dir.join(dir);
            let path = fs::canonicalize(dir.clone()).unwrap();
            let path = path.to_str().unwrap().to_string();
            path
        }
    }
}

impl BackendBuildEnv for Sbuild {
    fn clean(&self) -> Result<()> {
        let cache_dir = self.get_cache_dir();
        info!("Cleaning cached build: {}", cache_dir);
        remove_file_or_directory(&cache_dir, false)?;

        Ok(())
    }

    fn create(&self) -> Result<()> {
        let mut temp_dir = env::temp_dir();
        let dir_name = format!("temp_{}", random::<u32>().to_string());
        temp_dir.push(dir_name);
        fs::create_dir(&temp_dir)?;

        let cache_dir = self.get_cache_dir();

        let create_result = Command::new("sbuild-createchroot")
            .arg("--chroot-mode=unshare")
            .arg("--make-sbuild-tarball")
            .arg(cache_dir)
            .arg(&self.config.build_env.codename)
            .arg(temp_dir)
            .arg("http://deb.debian.org/debian")
            .status();

        if let Err(err) = create_result {
            return Err(eyre!(format!("Failed to create new chroot: {}", err)));
        }
        Ok(())
    }
    fn build(&self) -> Result<()> {
        let mut cmd_args = vec![
            "-d".to_string(),
            self.config.build_env.codename.to_string(),
            "-A".to_string(),                    // build_arch_all
            "-s".to_string(),                    // build source
            "--source-only-changes".to_string(), // source_only_changes
            "-v".to_string(),                    // verbose
            "--chroot-mode=unshare".to_string(),
        ];

        let lang_deps = self.get_additional_deps();

        for action in lang_deps.iter() {
            cmd_args.push(format!("--chroot-setup-commands={}", action))
        }
        cmd_args.push("--chroot-setup-commands=apt dist-upgrade".to_string());
        cmd_args.push("--chroot-setup-commands=apt autoremove -y && cat".to_string());

        if let Some(true) = self.config.build_env.run_lintian {
        } else {
            cmd_args.push("--no-run-lintian".to_string());
        }
        if let Some(true) = self.config.build_env.run_autopkgtest {
        } else {
            cmd_args.push("--no-run-autopkgtest".to_string());
        }
        if let Some(true) = self.config.build_env.run_piuparts {
        } else {
            cmd_args.push("--no-run-piuparts".to_string());
        }
        println!(
            "Building package by invoking: sbuild {}",
            cmd_args.join(" ")
        );

        let mut child = Command::new("sbuild")
            .current_dir(self.build_files_dir.to_string())
            .args(&cmd_args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);

            for line in reader.lines() {
                let line = line?;
                println!("{}", line);
            }
        }
        io::stdout().flush()?;

        child.wait().map_err(|err| eyre!(err.to_string()))?;

        Ok(())
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
    #[ignore]
    fn test_clean_sbuild_env() {
        setup();
        // let build_env = Sbuild::new(
        //     BuildConfig::new("bookworm", "", None, &"".to_string()),
        //     SbuildBuildOptions::default(),
        // );
    }

    #[test]
    #[ignore]
    fn test_create_sbuild_env() {
        setup();

        unreachable!("Test case not implemented yet");
    }

    #[test]
    #[ignore]
    fn test_build_virtual_package_in_sbuild_env() {
        setup();

        unreachable!("Test case not implemented yet");
    }
    #[test]
    #[ignore]

    fn test_build_rust_package_in_sbuild_env() {
        setup();

        unreachable!("Test case not implemented yet");
    }
    #[test]
    #[ignore]

    fn test_build_go_package_in_sbuild_env() {
        setup();

        unreachable!("Test case not implemented yet");
    }

    #[test]
    #[ignore]

    fn test_build_javascript_package_in_sbuild_env() {
        setup();

        unreachable!("Test case not implemented yet");
    }

    #[test]
    #[ignore]

    fn test_build_java_package_in_sbuild_env() {
        setup();

        unreachable!("Test case not implemented yet");
    }

    #[test]
    #[ignore]

    fn test_build_csharp_package_in_sbuild_env() {
        setup();

        unreachable!("Test case not implemented yet");
    }

    #[test]
    #[ignore]

    fn test_build_typescript_package_in_sbuild_env() {
        setup();

        unreachable!("Test case not implemented yet");
    }

    #[test]
    #[ignore]

    fn test_build_nim_package_in_sbuild_env() {
        setup();
        unreachable!("Test case not implemented yet");
    }
}
