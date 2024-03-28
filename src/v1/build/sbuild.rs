use crate::v1::packager::{BackendBuildEnv, BuildConfig, LanguageEnv};
use log::info;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::{fs, io};

pub struct SbuildBuildOptions {
    run_lintian: bool,
    run_piuparts: bool,
    run_autopkgtest: bool,
}
impl SbuildBuildOptions {
    pub fn default() -> Self {
        SbuildBuildOptions {
            run_autopkgtest: false,
            run_piuparts: false,
            run_lintian: false,
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
        check_if_root()?;

        let build_prefix = self.get_build_name();
        info!(
            "Cleaning up sbuild directories with prefix: {}",
            build_prefix
        );

        remove_dir_recursive(&format!("/etc/sbuild/chroot/{}", build_prefix))
            .map_err(|err| err.to_string())?;
        remove_dir_recursive(&format!("/etc/schroot/chroot.d/{}*", build_prefix))
            .map_err(|err| err.to_string())?;
        remove_dir_recursive(&format!("/srv/chroot/{}", build_prefix))
            .map_err(|err| err.to_string())?;

        Ok(())
    }

    fn create(&self) -> Result<(), String> {
        let build_prefix = self.get_build_name();

        check_if_root()?;

        let create_result = Command::new("sbuild-createchroot")
            .arg("--merged-usr")
            .arg("--chroot-prefix")
            .arg(&build_prefix)
            .arg(&self.config().codename())
            .arg(&format!("/srv/chroot/{}", &build_prefix))
            .arg("http://deb.debian.org/debian")
            .status();

        if let Err(err) = create_result {
            return Err(format!("Failed to create new chroot: {}", err));
        }

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

fn check_if_root() -> Result<(), String> {
    return if let Some(user) = std::env::var("USER").ok() {
        if user == "root" {
            Ok(())
        } else {
            Err("This program was not invoked with sudo.".to_string())
        }
    } else {
        Err("The USER environment variable is not set.".to_string())
    };
}

fn remove_dir_recursive(dir_path: &str) -> Result<(), std::io::Error> {
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
        assert!(false, "Test case not implemented yet");
    }

    #[test]
    fn test_create_sbuild_env() {
        setup();

        assert!(false, "Test case not implemented yet");
    }

    #[test]
    fn test_build_virtualpackage_in_sbuild_env() {
        setup();

        assert!(false, "Test case not implemented yet");
    }
    #[test]
    fn test_build_rust_package_in_sbuild_env() {
        setup();

        assert!(false, "Test case not implemented yet");
    }
    #[test]
    fn test_build_go_package_in_sbuild_env() {
        setup();

        assert!(false, "Test case not implemented yet");
    }

    #[test]
    fn test_build_javascript_package_in_sbuild_env() {
        setup();

        assert!(false, "Test case not implemented yet");
    }

    #[test]
    fn test_build_java_package_in_sbuild_env() {
        setup();

        assert!(false, "Test case not implemented yet");
    }

    #[test]
    fn test_build_csharp_package_in_sbuild_env() {
        setup();

        assert!(false, "Test case not implemented yet");
    }

    #[test]
    fn test_build_typescript_package_in_sbuild_env() {
        setup();

        assert!(false, "Test case not implemented yet");
    }

    #[test]
    fn test_build_zig_package_in_sbuild_env() {
        setup();
        assert!(false, "Test case not implemented yet");
    }
}
