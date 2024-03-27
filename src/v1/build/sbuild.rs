use crate::v1::packager::{BackendBuildEnv, BuildConfig, LanguageEnv};
use log::info;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

pub enum Sbuild {
    Rust(BuildConfig),
    Go(BuildConfig),
    JavaScript(BuildConfig),
    Java(BuildConfig),
    CSharp(BuildConfig),
    TypeScript(BuildConfig),
    Zig(BuildConfig),

    // No dependency
    EmptyEnv(BuildConfig),
}

impl Sbuild {
    pub fn new(build_config: BuildConfig) -> Sbuild {
        match build_config.lang_env() {
            Some(LanguageEnv::Rust) => Sbuild::Rust(build_config),
            Some(LanguageEnv::Go) => Sbuild::Go(build_config),
            Some(LanguageEnv::JavaScript) => Sbuild::JavaScript(build_config),
            Some(LanguageEnv::Java) => Sbuild::Java(build_config),
            Some(LanguageEnv::CSharp) => Sbuild::CSharp(build_config),
            Some(LanguageEnv::TypeScript) => Sbuild::TypeScript(build_config),
            Some(LanguageEnv::Zig) => Sbuild::Zig(build_config),
            None => Sbuild::EmptyEnv(build_config),
        }
    }
    pub fn config(&self) -> &BuildConfig {
        match self {
            Sbuild::Rust(build_config) => build_config,
            Sbuild::Go(build_config) => build_config,
            Sbuild::JavaScript(build_config) => build_config,
            Sbuild::Java(build_config) => build_config,
            Sbuild::CSharp(build_config) => build_config,
            Sbuild::TypeScript(build_config) => build_config,
            Sbuild::Zig(build_config) => build_config,
            Sbuild::EmptyEnv(build_config) => build_config,
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

        let output = Command::new("sbuild")
            .current_dir(self.config().package_dir())
            .arg("-c")
            .arg(format!(
                "{}-{}-sbuild",
                self.get_build_name(),
                self.config().arch()
            ))
            .arg("-d")
            .arg(self.config().codename())
            .output()
            .map_err(|err| err.to_string())?;

        println!("Command Output: {:?}", output);
        if !output.status.success() {
            return Err(format!("Failed to build package {:?}", output));
        }

        Ok(())
    }
}
