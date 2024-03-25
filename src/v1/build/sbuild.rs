use crate::v1::packager::{BackendBuildEnv, BuildConfig, LanguageEnv};
use log::info;
use std::fs;
use std::path::Path;
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
        let chroot_prefix = self.get_build_name();
        info!(
            "Cleaning up sbuild directories with prefix: {}",
            chroot_prefix
        );

        remove_dir_recursive(&format!("/etc/sbuild/chroot/{}", chroot_prefix))
            .map_err(|err| err.to_string())?;
        remove_dir_recursive(&format!("/etc/schroot/chroot.d/{}*", chroot_prefix))
            .map_err(|err| err.to_string())?;
        remove_dir_recursive(&format!("/srv/chroot/{}", chroot_prefix))
            .map_err(|err| err.to_string())?;

        Ok(())
    }

    fn create(&self) -> Result<(), String> {
        let chroot_prefix = self.get_build_name();
        info!("Creating new sbuild env: {}", chroot_prefix);

        let command_name = "sbuild_createchroot";
        let output = Command::new("which")
            .arg(command_name)
            .output()
            .expect(&format!("{} is not installed", command_name));

        if !output.status.success() {
            return Err(format!(
                "{} is not installed. Please install it",
                command_name
            ));
        }
        // Create new chroot
        let output = Command::new(command_name)
            .arg("--merged-usr")
            .arg("--chroot-prefix")
            .arg(&chroot_prefix)
            .arg(&self.config().codename())
            .arg(&format!("/srv/chroot/{}", chroot_prefix))
            .arg("http://deb.debian.org/debian")
            .output()
            .map_err(|err| err.to_string())?;

        if !output.status.success() {
            return Err(format!("Failed to create new sbuild {:?}", output));
        }

        Ok(())
    }
    fn build(&self) -> Result<(), String> {
        // Run in chroot
        let output = Command::new("sbuild")
            .arg("-c")
            .arg(self.get_build_name())
            .arg(self.config().codename())
            .output()
            .map_err(|err| err.to_string())?;

        if !output.status.success() {
            return Err(format!("Failed to build package {:?}", output));
        }

        Ok(())
    }
}

fn remove_dir_recursive(dir_path: &str) -> Result<(), std::io::Error> {
    if Path::new(dir_path).exists() {
        fs::remove_dir_all(dir_path)?;
        info!("Removed directory: {}", dir_path);
    }
    Ok(())
}
