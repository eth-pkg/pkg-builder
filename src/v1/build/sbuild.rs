use crate::v1::packager::{BackendBuildEnv, BuildConfig, LanguageEnv};
use std::process::Command;

pub enum Sbuild {
    Rust(BuildConfig),
    Go(BuildConfig),
    JavaScript(BuildConfig),
    Java(BuildConfig),
    CSharp(BuildConfig),
    TypeScript(BuildConfig),
    Zig(BuildConfig),
}

impl Sbuild {
    pub fn new(build_config: BuildConfig) -> Sbuild {
        match build_config.lang_env() {
            LanguageEnv::Rust => Sbuild::Rust(build_config),
            LanguageEnv::Go => Sbuild::Go(build_config),
            LanguageEnv::JavaScript => Sbuild::JavaScript(build_config),
            LanguageEnv::Java => Sbuild::Java(build_config),
            LanguageEnv::CSharp => Sbuild::CSharp(build_config),
            LanguageEnv::TypeScript => Sbuild::TypeScript(build_config),
            LanguageEnv::Zig => Sbuild::Zig(build_config),
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
        }
    }
    fn get_build_name(&self) -> String {
        return format!(
            "{}-{}-{}",
            self.config().codename(),
            self.config().arch(),
            self.config().lang_env()
        );
    }
}

impl BackendBuildEnv for Sbuild {
    fn clean(&self) -> Result<(), String> {
        let chroot_prefix = self.get_build_name();

        // Clean up previous chroots
        let cleanup_result = Command::new("sudo")
            .arg("rm")
            .args(&["-rf", &format!("/etc/sbuild/chroot/{}", chroot_prefix)])
            .args(&["-rf", &format!("/etc/schroot/chroot.d/{}*", chroot_prefix)])
            .args(&["-rf", &format!("/srv/chroot/{}", chroot_prefix)])
            .status();

        if let Err(err) = cleanup_result {
            return Err(format!("Failed to clean up previous chroots: {}", err));
        }
        Ok(())
    }

    fn create(&self) -> Result<(), String> {
        let chroot_prefix = self.get_build_name();

        // Create new chroot
        let create_result = Command::new("sudo")
            .arg("sbuild-createchroot")
            .arg("--merged-usr")
            .arg("--chroot-prefix")
            .arg(&chroot_prefix)
            .arg(&self.config().codename())
            .arg(&format!("/srv/chroot/{}", chroot_prefix))
            .arg("http://deb.debian.org/debian")
            .status();

        if let Err(err) = create_result {
            return Err(format!("Failed to create new chroot: {}", err));
        }

        Ok(())
    }
    fn build(&self) -> Result<(), String> {
        // Create new chroot
        let create_result = Command::new("sbuild")
            .arg("-c")
            .arg(self.get_build_name())
            .arg(self.config().codename())
            .status();

        if let Err(err) = create_result {
            return Err(format!("Failed to build package: {}", err));
        }

        Ok(())
    }
}
