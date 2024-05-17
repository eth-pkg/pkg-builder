use crate::v1::packager::BackendBuildEnv;
use crate::v1::pkg_config::{LanguageEnv, PackageType, PkgConfig};
use eyre::{eyre, Report, Result};
use log::{info, warn};
use rand::random;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::{env, fs, io};
use std::fs::{create_dir_all};
use cargo_metadata::semver::Version;
use crate::v1::pkg_config_verify::PkgVerifyConfig;
use sha1::{Digest, Sha1}; // Import from the sha1 crate

pub struct Sbuild {
    config: PkgConfig,
    build_files_dir: String,
    cache_dir: String,
}

impl Sbuild {
    pub fn new(config: PkgConfig, build_files_dir: String) -> Sbuild {
        Sbuild {
            cache_dir: config
                .build_env
                .sbuild_cache_dir
                .clone()
                .unwrap_or("~/.cache/sbuild".to_string()),
            config,
            build_files_dir,
        }
    }

    fn get_build_deps_based_on_langenv(&self, lang_env: &LanguageEnv) -> Vec<String> {
        match lang_env {
            LanguageEnv::C => {
                let lang_deps = vec![];
                lang_deps
            }
            LanguageEnv::Rust(config) => {
                // TODO
                // let rust_version = &config.rust_version;
                let rust_binary_url = &config.rust_binary_url;
                let rust_binary_gpg_asc = &config.rust_binary_gpg_asc;
                let lang_deps = vec![
                    "apt install -y curl gpg gpg-agent".to_string(),
                    format!("cd /tmp && curl -o rust.tar.xz -L {}", rust_binary_url),
                    format!("cd /tmp && echo \"{}\" >> rust.tar.xz.asc && cat rust.tar.xz.asc ", rust_binary_gpg_asc),
                    "curl https://keybase.io/rust/pgp_keys.asc | gpg --import".to_string(),
                    "cd /tmp && gpg --verify rust.tar.xz.asc rust.tar.xz".to_string(),
                    "cd /tmp && tar xvJf rust.tar.xz -C . --strip-components=1 --exclude=rust-docs".to_string(),
                    "cd /tmp && /bin/bash install.sh --without=rust-docs".to_string(),
                    "apt remove -y curl gpg gpg-agent".to_string(),
                ];
                lang_deps
            }
            LanguageEnv::Go(config) => {
                // TODO
                //let go_version = &config.go_version;
                let go_binary_url = &config.go_binary_url;
                let go_binary_checksum = &config.go_binary_checksum;
                let install = vec![
                    "apt install -y curl".to_string(),
                    format!("cd /tmp && curl -o go.tar.gz -L {}", go_binary_url),
                    format!("cd /tmp && echo \"{} go.tar.gz\" >> hash_file.txt && cat hash_file.txt", go_binary_checksum),
                    "cd /tmp && sha256sum -c hash_file.txt".to_string(),
                    "cd /tmp && rm -rf /usr/local/go && mkdir /usr/local/go && tar -C /usr/local -xzf go.tar.gz".to_string(),
                    "ln -s /usr/local/go/bin/go /usr/bin/go".to_string(),
                    "go version".to_string(),
                    // add write permission, this is a chroot env, with one user, should be fine
                    "chmod -R a+rwx /usr/local/go/pkg".to_string(),
                    "apt remove -y curl".to_string(),
                ];
                install
            }
            LanguageEnv::JavaScript(config) | LanguageEnv::TypeScript(config) => {
                // let node_version = &config.go_version;
                let node_binary_url = &config.node_binary_url;
                let node_binary_checksum = &config.node_binary_checksum;
                let mut install = vec![
                    "apt install -y curl".to_string(),
                    format!("cd /tmp && curl -o node.tar.gz -L {}", node_binary_url),
                    format!("cd /tmp && echo \"{} node.tar.gz\" >> hash_file.txt && cat hash_file.txt", node_binary_checksum),
                    "cd /tmp && sha256sum -c hash_file.txt".to_string(),
                    "cd /tmp && rm -rf /usr/share/node && mkdir /usr/share/node && tar -C /usr/share/node -xzf node.tar.gz --strip-components=1".to_string(),
                    "ls -l /usr/share/node/bin".to_string(),
                    "ln -s /usr/share/node/bin/node /usr/bin/node".to_string(),
                    "ln -s /usr/share/node/bin/npm /usr/bin/npm".to_string(),
                    "ln -s /usr/share/node/bin/npx /usr/bin/npx".to_string(),
                    "ln -s /usr/share/node/bin/corepack /usr/bin/corepack".to_string(),
                    "apt remove -y curl".to_string(),
                    "node --version".to_string(),
                    "npm --version".to_string(),
                ];
                if let Some(yarn_version) = &config.yarn_version {
                    install.push(format!("npm install --global yarn@{}", yarn_version));
                    install.push("ln -s /usr/share/node/bin/yarn /usr/bin/yarn".to_string());
                    install.push("yarn --version".to_string());
                }
                install
            }
            LanguageEnv::Java(config) => {
                let is_oracle = config.is_oracle;
                if is_oracle {
                    let jdk_version = &config.jdk_version;
                    let jdk_binary_url = &config.jdk_binary_url;
                    let jdk_binary_checksum = &config.jdk_binary_checksum;
                    let mut install = vec![
                        "apt install -y wget".to_string(),
                        format!("mkdir -p /opt/lib/jvm/jdk-{version}-oracle && mkdir -p /usr/lib/jvm", version = jdk_version),
                        format!("cd /tmp && wget -q --output-document jdk.tar.gz {}", jdk_binary_url),
                        format!("cd /tmp && echo \"{} jdk.tar.gz\" >> hash_file.txt && cat hash_file.txt", jdk_binary_checksum),
                        "cd /tmp && sha256sum -c hash_file.txt".to_string(),
                        format!("cd /tmp && tar -zxf jdk.tar.gz -C /opt/lib/jvm/jdk-{version}-oracle --strip-components=1", version = jdk_version),
                        format!("ln -s /opt/lib/jvm/jdk-{version}-oracle/bin/java  /usr/bin/java", version = jdk_version),
                        format!("ln -s /opt/lib/jvm/jdk-{version}-oracle/bin/javac  /usr/bin/javac", version = jdk_version),
                        "java -version".to_string(),
                        "apt remove -y wget".to_string(),
                    ];
                    if let Some(gradle_config) = &config.gradle {
                        let gradle_version = &gradle_config.gradle_version;
                        let gradle_binary_url = &gradle_config.gradle_binary_url;
                        let gradle_binary_checksum = &gradle_config.gradle_binary_checksum;

                        install.push("apt install -y wget unzip".to_string());
                        install.push(format!("mkdir -p /opt/lib/gradle-{version}", version = gradle_version));
                        install.push(format!("cd /tmp && wget -q --output-document gradle.tar.gz {}", gradle_binary_url));
                        install.push(format!("cd /tmp && echo \"{} gradle.tar.gz\" > hash_file.txt && cat hash_file.txt", gradle_binary_checksum));
                        install.push("cd /tmp && sha256sum -c hash_file.txt".to_string());
                        install.push(format!("cd /tmp && unzip gradle.tar.gz && mv gradle-{version} /opt/lib", version = gradle_version));
                        install.push(format!("ln -s /opt/lib/gradle-{version}/bin/gradle  /usr/bin/gradle", version = gradle_version));
                        install.push("gradle -version".to_string());
                        install.push("apt remove -y wget".to_string());
                    }
                    return install;
                }
                vec![]
            }
            LanguageEnv::Dotnet(config) => {
                let dotnet_version = &config.dotnet_version;
                let dotnet_full_version = &config.dotnet_full_version;
                if self.config.build_env.codename == "bookworm" ||
                    self.config.build_env.codename == "jammy jellyfish" {
                    let install = vec![
                        "apt install -y wget".to_string(),
                        "cd /tmp && wget https://packages.microsoft.com/config/debian/12/packages-microsoft-prod.deb -O packages-microsoft-prod.deb".to_string(),
                        "cd /tmp && dpkg -i packages-microsoft-prod.deb ".to_string(),
                        "apt-get update -y".to_string(),
                        format!("apt-cache madison dotnet-sdk-{}", dotnet_version),
                        format!("apt-get install -y dotnet-sdk-{}={}", dotnet_version, dotnet_full_version),
                        "dotnet --version".to_string(),
                        "apt remove -y wget".to_string(),
                    ];
                    install
                } else if self.config.build_env.codename == "noble numbat" {
                    let install = vec![
                        format!("apt-cache madison dotnet{}", dotnet_version),
                        format!("apt-get install -y dotnet{}={}", dotnet_version, dotnet_full_version),
                        "dotnet --version".to_string(),
                        "apt remove -y wget".to_string(),
                    ];
                    install
                } else {
                    return vec![];
                }
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
        }
    }
    fn get_build_deps_not_in_debian(&self) -> Vec<String> {
        let package_type = &self.config.package_type;
        let lang_env = match package_type {
            PackageType::Default(config) => Some(&config.language_env),
            PackageType::Git(config) => Some(&config.language_env),
            PackageType::Virtual => None,
        };
        match lang_env {
            None => {
                vec![]
            }
            Some(lang_env) => {
                self.get_build_deps_based_on_langenv(lang_env)
            }
        }
    }
    fn get_test_deps_based_on_langenv(&self, lang_env: &LanguageEnv) -> Vec<String> {
        match lang_env {
            LanguageEnv::C => {
                let lang_deps = vec![];
                lang_deps
            }
            LanguageEnv::Rust(_) => {
                // rust compiles to binary, no need to install under test_bed
                let lang_deps = vec![];
                lang_deps
            }
            LanguageEnv::Go(_) => {
                // go compiles to binary, no need to install under test_bed
                let lang_deps = vec![];
                lang_deps
            }
            LanguageEnv::JavaScript(_) | LanguageEnv::TypeScript(_) => {
                // do not install node, as we cannot depend on it, make the testbed install it
                // let node_version = &config.go_version;
                let lang_deps = vec![];
                lang_deps
            }
            LanguageEnv::Java(_) => {
                // do not install jdk, or gradle, as we cannot depend on it, make the testbed install it
                // let node_version = &config.go_version;
                let lang_deps = vec![];
                lang_deps
            }
            LanguageEnv::Dotnet(_) => {
                // add ms repo, but do not install dotnet, let test_bed add it as intall dependency
                if self.config.build_env.codename == "bookworm" ||
                    self.config.build_env.codename == "jammy jellyfish"
                {
                    let install = vec![
                        "apt install -y wget".to_string(),
                        "cd /tmp && wget https://packages.microsoft.com/config/debian/12/packages-microsoft-prod.deb -O packages-microsoft-prod.deb".to_string(),
                        "cd /tmp && dpkg -i packages-microsoft-prod.deb ".to_string(),
                        "apt-get update -y".to_string(),
                        "apt remove -y wget".to_string(),
                    ];
                    install
                } else if self.config.build_env.codename == "noble numbat" {
                    return vec![];
                } else {
                    return vec![];
                }
            }
            LanguageEnv::Nim(_) => {
                // nim compiles to binary, no need to install under test_bed
                let lang_deps = vec![];
                lang_deps
            }
        }
    }
    fn get_test_deps_not_in_debian(&self) -> Vec<String> {
        let package_type = &self.config.package_type;
        let lang_env = match package_type {
            PackageType::Default(config) => Some(&config.language_env),
            PackageType::Git(config) => Some(&config.language_env),
            PackageType::Virtual => None,
        };
        match lang_env {
            None => {
                vec![]
            }
            Some(lang_env) => {
                self.get_test_deps_based_on_langenv(lang_env)
            }
        }
    }

    pub fn get_cache_file(&self) -> String {
        let dir = &self.cache_dir;
        let expanded_path = if dir.starts_with('~') {
            let expanded_path = shellexpand::tilde(dir).to_string();
            expanded_path
        } else if dir.starts_with('/') {
            self.cache_dir.clone()
        } else {
            let parent_dir = env::current_dir().unwrap();
            let dir = parent_dir.join(dir);
            let path = fs::canonicalize(dir.clone()).unwrap();
            let path = path.to_str().unwrap().to_string();
            path
        };

        let codename = normalize_codename(&self.config.build_env.codename).unwrap();
        let cache_file_name = format!(
            "{}-{}.tar.gz",
            codename, self.config.build_env.arch
        )
            .to_string();
        let path = Path::new(&expanded_path);
        let cache_file = path.join(cache_file_name);
        cache_file.to_str().unwrap().to_string()
    }

    pub fn get_deb_dir(&self) -> &Path {
        let deb_dir = Path::new(&self.build_files_dir).parent().unwrap();
        deb_dir
    }
    pub fn get_deb_name(&self) -> PathBuf {
        let deb_dir = self.get_deb_dir();
        let deb_file_name = format!("{}_{}-{}_{}.deb",
                                    self.config.package_fields.package_name,
                                    self.config.package_fields.version_number,
                                    self.config.package_fields.revision_number,
                                    self.config.build_env.arch);
        let deb_name = deb_dir.join(deb_file_name);
        deb_name
    }

    //hello-world_1.0.0-1_amd64.changes
    pub fn get_changes_file(&self) -> PathBuf {
        let deb_dir = self.get_deb_dir();
        let deb_file_name = format!("{}_{}-{}_{}.changes",
                                    self.config.package_fields.package_name,
                                    self.config.package_fields.version_number,
                                    self.config.package_fields.revision_number,
                                    self.config.build_env.arch);
        let deb_name = deb_dir.join(deb_file_name);
        deb_name
    }
}

impl BackendBuildEnv for Sbuild {
    fn clean(&self) -> Result<()> {
        let cache_file = self.get_cache_file();
        info!("Cleaning cached build: {}", cache_file);
        let path = Path::new(&cache_file);
        if path.exists() {
            remove_file_or_directory(&cache_file, false)
                .map_err(|_| eyre!("Could not remove previous cache file!"))?;
        }
        Ok(())
    }

    fn create(&self) -> Result<()> {
        let mut temp_dir = env::temp_dir();
        let dir_name = format!("temp_{}", random::<u32>());
        temp_dir.push(dir_name);
        fs::create_dir(&temp_dir)?;

        let cache_file = self.get_cache_file();
        let cache_dir = Path::new(&cache_file).parent().unwrap();
        create_dir_all(cache_dir).map_err(|_| eyre!("Failed to create cache_dir"))?;
        let codename = normalize_codename(&self.config.build_env.codename)?;

        let repo_url = get_repo_url(&self.config.build_env.codename.as_str())?;
        let create_result = Command::new("sbuild-createchroot")
            .arg("--chroot-mode=unshare")
            .arg("--make-sbuild-tarball")
            .arg(cache_file)
            .arg(codename)
            .arg(temp_dir)
            .arg(repo_url)
            .status();

        if let Err(err) = create_result {
            return Err(eyre!(format!("Failed to create new chroot: {}", err)));
        }
        Ok(())
    }
    fn package(&self) -> Result<()> {
        let codename = normalize_codename(&self.config.build_env.codename)?;

        let mut cmd_args = vec![
            "-d".to_string(),
            codename.to_string(),
            "-A".to_string(),                    // build_arch_all
            "-s".to_string(),                    // build source
            "--source-only-changes".to_string(), // source_only_changes
            "-c".to_string(),                    // override cache file location, default is ~/.cache/sbuild both by sbuild and pkg-builder
            self.get_cache_file(),
            "-v".to_string(),                    // verbose
            "--chroot-mode=unshare".to_string(),
        ];


        let lang_deps = self.get_build_deps_not_in_debian();

        for action in lang_deps.iter() {
            cmd_args.push(format!("--chroot-setup-commands={}", action))
        }

        cmd_args.push("--no-run-piuparts".to_string());
        cmd_args.push("--no-apt-upgrade".to_string());
        cmd_args.push("--no-apt-distupgrade".to_string());

        if let Some(true) = self.config.build_env.run_lintian {
            cmd_args.push("--run-lintian".to_string());
            cmd_args.push("--lintian-opt=-i".to_string());
            cmd_args.push("--lintian-opt=--I".to_string());
            cmd_args.push("--lintian-opt=--suppress-tags".to_string());
            cmd_args.push("--lintian-opt=bad-distribution-in-changes-file".to_string());
            cmd_args.push("--lintian-opt=--suppress-tags".to_string());
            cmd_args.push("--lintian-opt=debug-file-with-no-debug-symbols".to_string());
            cmd_args.push("--lintian-opt=--tag-display-limit=0".to_string());
            cmd_args.push("--lintian-opts=--fail-on=error".to_string());
            cmd_args.push("--lintian-opts=--fail-on=warning".to_string());
        } else {
            cmd_args.push("--no-run-lintian".to_string());

        }

        cmd_args.push("--no-run-autopkgtest".to_string());

        info!(
            "Building package by invoking: sbuild {}",
            cmd_args.join(" ")
        );

        let mut cmd = Command::new("sbuild")
            .current_dir(self.build_files_dir.clone())
            .args(&cmd_args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;
        run_process(&mut cmd)?;

        if let Some(true) = self.config.build_env.run_piuparts {
            self.run_piuparts()?;
        };

        if let Some(true) = self.config.build_env.run_autopkgtest {
           self.run_autopkgtests()?;
        }

        Ok(())
    }

    fn verify(&self, verify_config: PkgVerifyConfig) -> Result<()> {
        let output_dir = Path::new(&self.build_files_dir).parent().unwrap();
        let package_hash = verify_config.verify.package_hash;
        let mut errors: Vec<Report> = vec![];
        for output in package_hash.iter() {
            let file = output_dir.join(output.name.clone());
            if !file.exists() {
                return Err(eyre!(format!("File to be verified does not exist {}", output.name)));
            }
            let mut file = fs::File::open(file).map_err(|_| eyre!("Could not open file."))?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).map_err(|_| eyre!("Could not read file."))?;
            let actual_sha1 = calculate_sha1(&*buffer.clone()).unwrap_or_default();
            if actual_sha1 != output.hash {
                errors.push(eyre!(format!("file {} actual sha1 is {}", output.name,  &actual_sha1)));
            }
        }
        let result = if errors.is_empty() {
            println!("Verify is successful!");
            Ok(())
        } else {
            let mut combined_report = errors.pop().unwrap_or_else(|| Report::msg("No errors found"));

            for report in errors.into_iter() {
                combined_report = combined_report.wrap_err(report);
            }
            Err(combined_report)
        };
        result
    }

    fn run_lintian(&self) -> Result<()> {
        info!(
            "Running lintian outside, not as same as on CI..",
        );
        check_lintian_version(self.config.build_env.lintian_version.clone())?;
        // let deb_dir = self.get_deb_dir();
        let changes_file = self.get_changes_file();
        let changes_file = changes_file.to_str().unwrap();
        let mut cmd_args = vec![
            "--suppress-tags".to_string(),
            "bad-distribution-in-changes-file".to_string(),
            "-i".to_string(),
            "--I".to_string(),
            changes_file.to_string(),
            "--tag-display-limit=0".to_string(),
            "--fail-on=warning".to_string(), // fail on warning
            "--fail-on=error".to_string(), // fail on error
            "--suppress-tags".to_string(), // overrides fails for this message
            "debug-file-with-no-debug-symbols".to_string(),
        ];
        let codename = normalize_codename(&self.config.build_env.codename)?;

        if codename == "jammy".to_string() || codename == "noble".to_string() {
            // changed a format of .deb packages on ubuntu, it's not a bug
            // but some lintian will report as such
            cmd_args.push("--suppress-tags".to_string());
            cmd_args.push("malformed-deb-archive".to_string());
        }

        info!(
            "Testing package by invoking: lintian {}",
            cmd_args.join(" ")
        );

        let mut cmd = Command::new("lintian")
            // for CI
            .args(&cmd_args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;
        run_process(&mut cmd)

    }


    fn run_piuparts(&self) -> Result<()> {
        info!(
            "Running piuparts command with elevated privileges..",
        );
        info!(
            "Piuparts must run as root user through sudo, please provide your password, if prompted."
        );
        check_piuparts_version(self.config.build_env.piuparts_version.clone())?;

        let repo_url = get_repo_url(&self.config.build_env.codename.as_str())?;
        let keyring = get_keyring(&self.config.build_env.codename)?;
        let codename = normalize_codename(&self.config.build_env.codename)?;

        let mut cmd_args = vec![
            "-d".to_string(),
            codename.to_string(),
            "-m".to_string(),
            repo_url.to_string(),
            "--bindmount=/dev".to_string(),
            format!("--keyring={}", keyring),
            "--verbose".to_string(),
        ];
        let package_type = &self.config.package_type;

        let lang_env = match package_type {
            PackageType::Default(config) => Some(&config.language_env),
            PackageType::Git(config) => Some(&config.language_env),
            PackageType::Virtual => None,
        };
        if let Some(env) = lang_env {
            match env {
                LanguageEnv::Dotnet(_) => {
                    if self.config.build_env.codename == "bookworm" ||
                        self.config.build_env.codename == "jammy jellyfish" {
                        let ms_repo = format!("deb https://packages.microsoft.com/debian/12/prod {} main", self.config.build_env.codename);
                        cmd_args.push(format!("--extra-repo={}", ms_repo));
                        cmd_args.push("--do-not-verify-signatures".to_string());
                    } else if self.config.build_env.codename == "noble numbat" {
                    }
                }
                _ => {
                    // no other package repositories supported
                    // might supply my own, but not for now
                }
            }
        }
        let deb_dir = self.get_deb_dir();
        let deb_name = self.get_deb_name();
        info!(
            "Testing package by invoking: sudo -S piuparts {} {}",
            cmd_args.join(" "),
            deb_name.to_str().unwrap()
        );
        info!("Note this command run inside of directory: {}", deb_dir.display());

        let mut cmd = Command::new("sudo")
            .current_dir(deb_dir)
            // for CI
            .arg("-S")
            .arg("piuparts")
            .args(&cmd_args)
            .arg(deb_name)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;
        run_process(&mut cmd)
    }

    fn run_autopkgtests(&self) -> Result<()> {
        info!(
            "Running autopkgtests command outside of build env.",
        );
        check_autopkgtest_version(self.config.build_env.autopkgtest_version.clone())?;
        let codename = normalize_codename(&self.config.build_env.codename)?;

        let image_name = format!("autopkgtest-{}-{}.img", codename, self.config.build_env.arch);
        let mut cache_dir = self.cache_dir.clone();
        if cache_dir.starts_with('~') {
            cache_dir = shellexpand::tilde(&cache_dir).to_string()
        }
        let image_path = Path::new(&cache_dir).join(image_name.clone());
        create_autopkgtest_image(image_path.clone(), 
                                self.config.build_env.codename.to_string(), 
                                self.config.build_env.arch.to_string())?;

        let deb_dir = self.get_deb_dir();
        //  let deb_name = self.get_deb_name();
        let changes_file = self.get_changes_file();
        let mut cmd_args = vec![
            changes_file.to_str().unwrap().to_string(),
            // this will not going rebuild the package, which we want to avoid
            // as some packages can take an hour to build,
            // we don't want to build for 2 hours
            "--no-built-binaries".to_string(),
            // needed dist-upgrade as testbed is outdated, when new version of distribution released
            "--apt-upgrade".to_string(),
        ];
        let lang_deps = self.get_test_deps_not_in_debian();

        for action in lang_deps.iter() {
            cmd_args.push(format!("--setup-commands={}", action))
        }
        cmd_args.push("--".to_string());
        cmd_args.push("qemu".to_string());
        cmd_args.push(image_path.to_str().unwrap().to_string());
        info!(
            "Testing package by invoking: autopkgtest {}",
            cmd_args.join(" ")
        );
        info!("Note this command run inside of directory: {}", deb_dir.display());
        let mut cmd = Command::new("autopkgtest")
            .current_dir(deb_dir)
            .args(&cmd_args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;
        run_process(&mut cmd)
    }
}

fn check_lintian_version(expected_version: String) -> Result<()> {
    let output = Command::new("lintian")
        .arg("--version")
        .output()?;

    if output.status.success() {
        let mut output_str = String::from_utf8_lossy(&output.stdout).to_string()
            .replace("Lintian v", "")
            .replace("\n", "")
            .trim()
            .to_string();
        if let Some(pos) = output_str.find("ubuntu") {
            output_str.truncate(pos);
            output_str = output_str.trim().to_string();
        }
        warn_compare_versions(expected_version, &output_str, "lintian")?;
        Ok(())
    } else {
        Err(eyre!("Failed to execute lintian --version"))
    }
}

fn check_piuparts_version(expected_version: String) -> Result<()> {
    let output = Command::new("piuparts")
        .arg("--version")
        .output()?;

    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout)
            .to_string()
            .replace("piuparts ", "")
            .replace("\n", "")
            .trim()
            .to_string();
        warn_compare_versions(expected_version, &output_str, "piuparts")?;
        Ok(())
    } else {
        Err(eyre!("Failed to execute piuparts --version"))
    }
}

fn check_autopkgtest_version(expected_version: String) -> Result<()> {
    let output = Command::new("apt")
        .arg("list")
        .arg("--installed")
        .arg("autopkgtest")
        .output()?;

    //autopkgtest/jammy-updates,now 5.32ubuntu3~22.04.1 all [installed]
    if output.status.success() {
        let mut output_str = String::from_utf8_lossy(&output.stdout).to_string()
            .replace("Listing...", "")
            .replace("\n", "")
            .replace("autopkgtest/stable,now ", "")
            .replace("autopkgtest/jammy-updates,now ", "")
            .replace("autopkgtest/jammy,now ", "")
            .replace("ubuntu3~22.04.1", "")
            .trim()
            .to_string();
        if let Some(pos) = output_str.find("all ") {
            output_str.truncate(pos);
            output_str = output_str.trim().to_string();
        }
        info!("autopkgtest version {}", output_str);
        // append versions, to it looks like semver
        let expected_version = format!("{}.0", expected_version);
        let actual_version = format!("{}.0", output_str);
        warn_compare_versions(expected_version, &actual_version, "autopkgtest")?;
        Ok(())
    } else {
        Err(eyre!("Failed to execute apt list --installed autopkgtest"))
    }
}

pub fn warn_compare_versions(expected_version: String, actual_version: &str, program_name: &str) -> Result<()> {
    let expected_version = Version::parse(&expected_version).unwrap();
    let actual_version = Version::parse(actual_version).unwrap();
    match expected_version.cmp(&actual_version) {
        std::cmp::Ordering::Less => {
            warn!("Warning: using newer versions than expected version.");
            Ok(())
        }
        std::cmp::Ordering::Greater => {
            warn!("Using older version of {}", program_name);
            Ok(())
        }
        std::cmp::Ordering::Equal => {
            info!("Versions match. Proceeding.");
            Ok(())
        }
    }
}

pub fn normalize_codename(codename: &str) -> Result<&str> {
    match codename {
        "bookworm" => {
            Ok("bookworm")
        }
        "noble numbat" => {
            Ok("noble")
        }
        "jammy jellyfish" => {
            Ok("jammy")
        }
        _ => {
            Err(eyre!("Not supported distribution"))
        }
    }
}

pub fn get_keyring(codename: &str) -> Result<&str> {
    match codename {
        "bookworm" => {
            Ok("/usr/share/keyrings/debian-archive-keyring.gpg")
        }
        "noble numbat" | "jammy jellyfish" => {
            Ok("/usr/share/keyrings/ubuntu-archive-keyring.gpg")
        }
        _ => {
            Err(eyre!("Not supported distribution"))
        }
    }
}

pub fn get_repo_url(codename: &str) -> Result<&str> {
    match codename {
        "bookworm" => {
            Ok("http://deb.debian.org/debian")
        }
        "noble numbat" | "jammy jellyfish" => {
            Ok("http://archive.ubuntu.com/ubuntu")
        }
        _ => {
            Err(eyre!("Not supported distribution"))
        }
    }
}

pub fn calculate_sha1<R: Read>(mut reader: R) -> Result<String, io::Error> {
    let mut hasher = Sha1::new();
    io::copy(&mut reader, &mut hasher)?;
    let digest_bytes = hasher.finalize();
    let hex_digest = digest_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();

    Ok(hex_digest)
}

fn create_autopkgtest_image(image_path: PathBuf, codename: String, arch: String) -> Result<()> {

    // do not recreate image if exists
    if image_path.exists() {
        return Ok(());
    }
    info!(
        "autopkgtests environment does not exist. Creating it."
    );
    info!(
        "please provide your password through sudo to as autopkgtest env creation requires it."
    );
    create_dir_all(image_path.parent().unwrap())?;
    let repo_url = get_repo_url(&codename)?;



    match codename.as_str() {
        "bookworm" => {
            let codename = normalize_codename(&codename)?;
            let cmd_args = vec![
                codename.to_string(),
                image_path.to_str().unwrap().to_string(),
                format!("--mirror={}", repo_url),
                format!("--arch={}", arch),
            ];
            let mut cmd = Command::new("sudo")
                // for CI
                .arg("-S")
                .arg("autopkgtest-build-qemu")
                .args(&cmd_args)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()?;
            run_process(&mut cmd)
        }
        "noble numbat" | "jammy jellyfish" => {
            let codename = normalize_codename(&codename)?;
            let cmd_args = vec![
                format!("--release={}", codename.to_string()),
                format!("--mirror={}", repo_url),
                format!("--arch={}", arch),
                "-v".to_string(),
            ];
            let mut cmd = Command::new("sudo")
                // for CI
                .arg("-S")
                .arg("autopkgtest-buildvm-ubuntu-cloud")
                .args(&cmd_args)
                .current_dir(image_path.parent().unwrap().to_str().unwrap())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()?;
            run_process(&mut cmd)

        }
        _ => {
            Err(eyre!("Not supported distribution"))
        }
    }
}

fn run_process(child: &mut Child) -> Result<()> {
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);

        for line in reader.lines() {
            let line = line?;
            info!("{}", line);
        }
    }
    io::stdout().flush()?;

    let status = child.wait().map_err(|err| eyre!(err.to_string()))?;
    if status.success() {
        Ok(())
    } else {
        Err(eyre!("Sbuild exited with non-zero status code. Please see build output for potential causes."))
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
    use super::*;
    use env_logger::Env;
    use std::fs::File;
    use std::sync::Once;
    use tempfile::tempdir;

    static INIT: Once = Once::new();

    // Set up logging for tests
    fn setup() {
        INIT.call_once(|| {
            env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
        });
    }

    #[test]
    fn test_clean_sbuild_env_when_file_does_not_exist() {
        setup();
        let mut pkg_config = PkgConfig::default();
        let build_files_dir = tempdir().unwrap().path().to_str().unwrap().to_string();
        pkg_config.build_env.codename = "bookworm".to_string();
        pkg_config.build_env.arch = "amd64".to_string();
        let sbuild_cache_dir = tempdir().unwrap().path().to_str().unwrap().to_string();
        pkg_config.build_env.sbuild_cache_dir = Some(sbuild_cache_dir);
        let build_env = Sbuild::new(pkg_config, build_files_dir);
        let result = build_env.clean();
        assert!(result.is_ok());
        let cache_file = build_env.get_cache_file();
        let cache_file_path = Path::new(&cache_file);
        assert!(!cache_file_path.exists())
    }

    #[test]
    fn test_clean_sbuild_env() {
        setup();
        let mut pkg_config = PkgConfig::default();
        let build_files_dir = tempdir().unwrap().path().to_str().unwrap().to_string();
        pkg_config.build_env.codename = "bookworm".to_string();
        pkg_config.build_env.arch = "amd64".to_string();
        let sbuild_cache = tempdir().unwrap();
        // create dir manually, as it doesn't exist
        create_dir_all(sbuild_cache.path()).expect("Could not create temporary directory for testing.");
        let sbuild_cache_dir = sbuild_cache.path().to_str().unwrap().to_string();
        pkg_config.build_env.sbuild_cache_dir = Some(sbuild_cache_dir.clone());
        let build_env = Sbuild::new(pkg_config, build_files_dir);
        let cache_file = build_env.get_cache_file();
        let cache_file_path = Path::new(&cache_file);

        File::create(cache_file_path)
            .expect("File needs to be created manually before testing deletion.");
        assert!(
            Path::new(&sbuild_cache_dir).exists(),
        );

        assert!(
            cache_file_path.exists(),
            "File should exist before testing deletion."
        );

        let result = build_env.clean();
        assert!(result.is_ok());
        assert!(!cache_file_path.exists())
    }

    #[test]
    fn test_create_sbuild_env() {
        setup();
        let mut pkg_config = PkgConfig::default();
        pkg_config.build_env.codename = "bookworm".to_string();
        pkg_config.build_env.arch = "amd64".to_string();
        let sbuild_cache_dir = tempdir().unwrap().path().to_str().unwrap().to_string();
        pkg_config.build_env.sbuild_cache_dir = Some(sbuild_cache_dir);

        let build_files_dir = tempdir().unwrap().path().to_str().unwrap().to_string();
        let build_env = Sbuild::new(pkg_config, build_files_dir);
        build_env.clean().expect("Could not clean previous env.");
        let cache_file = build_env.get_cache_file();
        let cache_file_path = Path::new(&cache_file);
        assert!(!cache_file_path.exists());
        let result = build_env.create();
        assert!(result.is_ok());
        assert!(cache_file_path.exists())
    }
}
