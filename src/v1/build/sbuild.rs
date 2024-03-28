use crate::v1::packager::{BackendBuildEnv, BuildConfig, LanguageEnv};
use log::info;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::process::{Command, Stdio};
use std::{fs, io};

pub struct SbuildBuildOptions {
    run_lintian: bool,
    run_piuparts: bool,
    run_autopkgtest: bool,
}
impl SbuildBuildOptions{
    pub fn default() -> Self {
        SbuildBuildOptions {
            run_autopkgtest: false,
            run_piuparts: false,
            run_lintian: false
        }
    }
}

pub enum Sbuild {
    Rust(BuildConfig, SbuildBuildOptions),
    Go(BuildConfig, SbuildBuildOptions),
    JavaScript(BuildConfig, SbuildBuildOptions),
    Java(BuildConfig, SbuildBuildOptions),
    CSharp(BuildConfig, SbuildBuildOptions),
    TypeScript(BuildConfig, SbuildBuildOptions),
    Zig(BuildConfig, SbuildBuildOptions),

    // No dependency
    EmptyEnv(BuildConfig, SbuildBuildOptions),
}

impl Sbuild {
    pub fn new(build_config: BuildConfig, build_options: SbuildBuildOptions) -> Sbuild {
        match build_config.lang_env() {
            Some(LanguageEnv::Rust) => Sbuild::Rust(build_config, build_options),
            Some(LanguageEnv::Go) => Sbuild::Go(build_config, build_options),
            Some(LanguageEnv::JavaScript) => Sbuild::JavaScript(build_config, build_options),
            Some(LanguageEnv::Java) => Sbuild::Java(build_config, build_options),
            Some(LanguageEnv::CSharp) => Sbuild::CSharp(build_config, build_options),
            Some(LanguageEnv::TypeScript) => Sbuild::TypeScript(build_config, build_options),
            Some(LanguageEnv::Zig) => Sbuild::Zig(build_config, build_options),
            None => Sbuild::EmptyEnv(build_config, build_options),
        }
    }
    pub fn config(&self) -> &BuildConfig {
        match self {
            Sbuild::Rust(build_config, _) => build_config,
            Sbuild::Go(build_config, _) => build_config,
            Sbuild::JavaScript(build_config, _) => build_config,
            Sbuild::Java(build_config, _) => build_config,
            Sbuild::CSharp(build_config, _) => build_config,
            Sbuild::TypeScript(build_config, _) => build_config,
            Sbuild::Zig(build_config, _) => build_config,
            Sbuild::EmptyEnv(build_config, _) => build_config,
        }
    }
    pub fn build_options(&self) -> &SbuildBuildOptions {
        match self {
            Sbuild::Rust(_, build_options) => build_options,
            Sbuild::Go(_, build_options) => build_options,
            Sbuild::JavaScript(_, build_options) => build_options,
            Sbuild::Java(_, build_options) => build_options,
            Sbuild::CSharp(_, build_options) => build_options,
            Sbuild::TypeScript(_, build_options) => build_options,
            Sbuild::Zig(_, build_options) => build_options,
            Sbuild::EmptyEnv(_, build_options) => build_options,
        }
    }
    fn get_build_name(&self) -> String {
        return format!(
            "{}-{}-{}",
            self.config().codename(),
            self.config().arch(),
            self.config()
                .lang_env()
                .map_or("empty-env".to_string(), |v| v.to_string())
        );
    }
}

impl BackendBuildEnv for Sbuild {
    fn clean(&self) -> Result<(), String> {
        let build_prefix = self.get_build_name();
        println!(
            "To clean up sbuild directories with prefix: {}, execute the following commands:",
            build_prefix
        );

        // Construct the directory paths
        let etc_sbuild_dir = format!("/etc/sbuild/chroot/{}", build_prefix);
        let etc_schroot_dir = format!("/etc/schroot/chroot.d/{}", build_prefix);
        let srv_chroot_dir = format!("/srv/chroot/{}", build_prefix);

        // Print out the commands to remove directories
        println!("sudo rm -rf {}", etc_sbuild_dir);
        println!("sudo rm -rf {}", etc_schroot_dir);
        println!("sudo rm -rf {}", srv_chroot_dir);

        Ok(())
    }

    fn create(&self) -> Result<(), String> {
        let build_prefix = self.get_build_name();
        println!("To build the package please create a build environment manually first, by running the following, and rerunning the current command.");
        // Construct the command string
        let command = format!(
            "sudo {} --merged-usr --chroot-prefix {} {} {} {}",
            "sbuild-createchroot",
            &build_prefix,
            &self.config().codename(),
            &format!("/srv/chroot/{}", build_prefix),
            "http://deb.debian.org/debian"
        );

        // Print out the command for the user to execute
        println!("Run the following command to create the sbuild environment:");
        println!("{}", command);

        Ok(())
    }
    fn build(&self) -> Result<(), String> {
        let sbuild_command = format!(
            "sbuild -c {} -d {}",
            self.get_build_name(),
            self.config().codename(),
        );

        // Get the current permissions of the file
        info!(
            "Adding executable permission for {}/debian/rules",
            self.config().package_dir()
        );

        let debian_rules = format!("{}/debian/rules", self.config().package_dir());
        let mut permissions = fs::metadata(debian_rules.clone())
            .map_err(|err| err.to_string())?
            .permissions();
        permissions.set_mode(permissions.mode() | 0o111);
        fs::set_permissions(debian_rules, permissions).map_err(|err| err.to_string())?;

        info!("Building package by invoking: {}", sbuild_command);
        let mut cmd_args = vec![
            "-c".to_string(),
            format!("{}-{}-sbuild", self.get_build_name(), self.config().arch()),
            "-d".to_string(),
            self.config().codename().to_string(),
        ];

        let build_options = self.build_options();
        if !build_options.run_lintian {
            cmd_args.push("--no-run-lintian".to_string());
        }
        if !build_options.run_autopkgtest {
            cmd_args.push("--no-run-autopkgtest".to_string());
        }
        if !build_options.run_piuparts {
            cmd_args.push("--no-run-piuparts".to_string());
        }

        let mut child = Command::new("sbuild")
            .current_dir(self.config().package_dir())
            .args(&cmd_args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|err| format!("Failed to start sbuild: {}", err.to_string()))?;

        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);

            for line in reader.lines() {
                println!(
                    "{}",
                    line.map_err(|err| format!("Failed to log: {}", err.to_string()))?
                );
            }
        }
        io::stdout()
            .flush()
            .map_err(|_| "Failed to flush output of sbuild".to_string())?;

        let status = child
            .wait()
            .map_err(|err| format!("Failed to build package: {}", err.to_string()))?;
        println!("Command exited with: {}", status);

        Ok(())
    }
}
