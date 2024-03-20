use std::process::Command;


struct SbuildConfig {
    codename: String,
    arch: String,
    build_env: BuildEnv,
}

pub struct SbuildDefault {
    config: SbuildConfig,
}

pub trait SbuildFactory {
    fn create_sbuild(&self);
}

// Implement the trait for each enum variant
impl SbuildFactory for BuildEnv {
    fn create_sbuild(&self) {
        match self {
            BuildEnv::Rust => println!("Creating sbuild for Rust"),
            BuildEnv::Go => println!("Creating sbuild for Go"),
            BuildEnv::JavaScript => println!("Creating sbuild for JavaScript"),
            BuildEnv::Java => println!("Creating sbuild for Java"),
            BuildEnv::CSharp => println!("Creating sbuild for C#"),
            BuildEnv::TypeScript => println!("Creating sbuild for TypeScript"),
            BuildEnv::Zig => println!("Creating sbuild for Zig"),
        }
    }
}

pub trait Sbuild {
    fn create_sbuild(&self);
    fn clean_sbuild(&self);
}

impl Sbuild for SbuildDefault {
    fn new(codename: &str, arch: &str, build_env: BuildEnv) -> Self {
        let config = SbuildConfig {
            codename: codename.to_string(),
            arch: arch.to_string(),
            build_env: build_env,
        };
        return SbuildDefault {
            config
        }
    }

    fn clean_sbuild(&self) -> Result<(), String> {
        let chroot_prefix = format!("{}-{}-default", self.codename, self.arch);

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
    fn create_sbuild(&self) -> Result<(), String> {
        // Create new chroot
        let create_result = Command::new("sudo")
            .arg("sbuild-createchroot")
            .arg("--merged-usr")
            .arg("--chroot-prefix")
            .arg(&chroot_prefix)
            .arg(&self.codename)
            .arg(&format!("/srv/chroot/{}", chroot_prefix))
            .arg("http://deb.debian.org/debian")
            .status();

        if let Err(err) = create_result {
            return Err(format!("Failed to create new chroot: {}", err));
        }

        Ok(())
    }
}
