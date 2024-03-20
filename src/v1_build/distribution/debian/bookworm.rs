use std::fs;

use crate::v1_build::distribution::build::SbuildGo;
use crate::v1_build::distribution::build::SbuildJava;
use crate::v1_build::distribution::build::SbuildNode;
use crate::v1_build::distribution::build::SbuildRust;
use crate::v1_build::distribution::build::SbuildDotnet;
use crate::v1_build::distribution::build::SbuildZig;
use crate::v1_build::distribution::packager::{
    BuildConfig, BackendBuildEnv, LanguageEnv, Packager, PackagerConfig,
};
use std::io::Write;
use std::path::Path;
use std::process::Command;

pub struct BookwormPackager {
    config: BookwormPackagerConfig,
    options: BookwormPackagerOptions,
}

pub struct BookwormPackagerOptions {
    work_dir: String,
    verbose: bool,
}
pub struct BookwormPackagerConfig {
    arch: String,
    package_name: String,
    version_number: String,
    tarball_url: String,
    git_source: String,
    is_virtual_package: bool,
    is_git: bool,
    lang_env: LanguageEnv,
}

impl PackagerConfig for BookwormPackagerConfig {}
pub struct BookwormPackagerConfigBuilder {
    arch: Option<String>,
    package_name: Option<String>,
    version_number: Option<String>,
    tarball_url: Option<String>,
    git_source: Option<String>,
    is_virtual_package: bool,
    is_git: bool,
    lang_env: Option<LanguageEnv>,
}

impl BookwormPackagerConfigBuilder {
    pub fn new() -> Self {
        BookwormPackagerConfigBuilder {
            arch: None,
            package_name: None,
            version_number: None,
            tarball_url: None,
            git_source: None,
            is_virtual_package: false,
            is_git: false,
            lang_env: None,
        }
    }

    pub fn arch(mut self, arch: String) -> Self {
        self.arch = Some(arch);
        self
    }

    pub fn package_name(mut self, package_name: String) -> Self {
        self.package_name = Some(package_name);
        self
    }

    pub fn version_number(mut self, version_number: String) -> Self {
        self.version_number = Some(version_number);
        self
    }

    pub fn tarball_url(mut self, tarball_url: String) -> Self {
        self.tarball_url = Some(tarball_url);
        self
    }

    pub fn git_source(mut self, git_source: String) -> Self {
        self.git_source = Some(git_source);
        self
    }

    pub fn is_virtual_package(mut self, is_virtual_package: bool) -> Self {
        self.is_virtual_package = is_virtual_package;
        self
    }

    pub fn is_git(mut self, is_git: bool) -> Self {
        self.is_git = is_git;
        self
    }

    pub fn lang_env(mut self, lang_env: String) -> Self {
        self.lang_env = LanguageEnv::from_string(&lang_env);
        self
    }

    pub fn config(self) -> Result<BookwormPackagerConfig, String> {
        let arch = self.arch.ok_or_else(|| "Missing arch field".to_string())?;
        let package_name = self
            .package_name
            .ok_or_else(|| "Missing package_name field".to_string())?;
        let version_number = self
            .version_number
            .ok_or_else(|| "Missing version_number field".to_string())?;
        let tarball_url = self
            .tarball_url
            .ok_or_else(|| "Missing tarball_url field".to_string())?;
        let git_source = self
            .git_source
            .ok_or_else(|| "Missing git_source field".to_string())?;
        let lang_env = self
            .lang_env
            .ok_or_else(|| "Missing lang_env field".to_string())?;

        Ok(BookwormPackagerConfig {
            arch,
            package_name,
            version_number,
            tarball_url,
            git_source,
            is_virtual_package: self.is_virtual_package,
            is_git: self.is_git,
            lang_env,
        })
    }
}

impl Packager for BookwormPackager {
    type Config = BookwormPackagerConfig;

    fn new(config: Self::Config) -> Self {
        return BookwormPackager {
            config,
            options: BookwormPackagerOptions {
                work_dir: "/tmp/debian".to_string(),
                verbose: true,
            },
        };
    }
    fn create_build_env(&self) -> Result<Box<dyn BackendBuildEnv>, String> {
        let build_config = BuildConfig::new("bookworm", &self.config.arch, self.config.lang_env);
        let backend_build_env: Box<dyn BackendBuildEnv> = match self.config.lang_env {
            LanguageEnv::Rust => Box::new(SbuildRust::new(build_config)),
            LanguageEnv::Go => Box::new(SbuildGo::new(build_config)),
            LanguageEnv::JavaScript => Box::new(SbuildNode::new(build_config)),
            LanguageEnv::Java => Box::new(SbuildJava::new(build_config)),
            LanguageEnv::CSharp => Box::new(SbuildDotnet::new(build_config)),
            LanguageEnv::TypeScript => Box::new(SbuildNode::new(build_config)),
            LanguageEnv::Zig => Box::new(SbuildZig::new(build_config)),
        };
        Ok(backend_build_env)
    }
    fn package(&self) -> Result<bool, String> {
        let packaging_dir = self.create_package_dir()?;
        self.download_source(&packaging_dir)?;
        self.extract_source(&packaging_dir)?;
        self.create_debian_dir(&packaging_dir)?;
        self.patch_source(&packaging_dir)?;

        let backend_build_env = self.create_build_env()?;
        backend_build_env.build()?;
        Ok(true)
    }
}

impl BookwormPackager {
    fn create_package_dir(&self) -> Result<String, String> {
        let packaging_dir = format!("{}/{}", self.options.work_dir, self.config.package_name);
        if self.options.verbose {
            println!("Creating package folder {}", &packaging_dir);
        }
        match fs::create_dir_all(&packaging_dir) {
            Ok(_) => Ok(packaging_dir),
            Err(err) => Err(err.to_string()), // Convert the error to a string
        }
    }
    fn download_source(&self, packaging_dir: &String) -> Result<bool, String> {
        let tarball_path = format!(
            "{}/{}.orig.tar.gz",
            &packaging_dir, self.config.package_name
        );
        if !self.config.is_virtual_package {
            if self.config.is_git {
                todo!()
            } else {
                if self.options.verbose {
                    println!("Downloading source {}", &tarball_path);
                }
                let status = Command::new("wget")
                    .arg("-O")
                    .arg(&tarball_path)
                    .arg(&self.config.tarball_url)
                    .status()
                    .map_err(|err| err.to_string())?;
                if !status.success() {
                    return Err("Download failed".to_string());
                }
            }
            Ok(true)
        } else {
            if self.options.verbose {
                println!("Creating empty .tar.gz for virtual package");
            }
            let output = Command::new("tar")
                .args(&["czvf", &tarball_path, "--files-from", "/dev/null"])
                .current_dir(&packaging_dir)
                .output()
                .map_err(|err| err.to_string())?;
            if !output.status.success() {
                return Err("Extract failed".to_string());
            }
            Ok(true)
        }
    }
    fn extract_source(&self, packaging_dir: &String) -> Result<bool, String> {
        let tarball_path = format!(
            "{}/{}.orig.tar.gz",
            &packaging_dir, self.config.package_name
        );
        println!("Extracting source {}", &packaging_dir);
        let output = Command::new("tar")
            .args(&[
                "zxvf",
                &tarball_path,
                "-C",
                &packaging_dir,
                "--strip-components=1",
            ])
            .output()
            .map_err(|err| err.to_string())?;
        if !output.status.success() {
            return Err("Extraction failed".to_string());
        }
        Ok(true)
    }
    fn create_debian_dir(&self, packaging_dir: &String) -> Result<bool, String> {
        let output = Command::new("debcrafter")
            .arg(format!("{}.sss", self.config.package_name))
            .arg("/tmp")
            .output()
            .map_err(|err| err.to_string())?;

        if !output.status.success() {
            return Err("Debcrafter failed".to_string());
        }

        let tmp_debian_dir = format!(
            "/tmp/{}-{}",
            self.config.package_name, self.config.version_number
        );
        let packaging_dir_debian: String = format!("{}/debian", packaging_dir);

        println!(
            "Copying debian directory from {} to {}",
            &tmp_debian_dir, &packaging_dir_debian
        );
        fs::copy(&tmp_debian_dir, &packaging_dir_debian).map_err(|err| err.to_string())?;

        Ok(true)
    }
    fn patch_source(&self, packaging_dir: &String) -> Result<bool, String> {
        let packaging_dir_debian: String = format!("{}/debian", packaging_dir);

        // Patch quilt
        let debian_source_format_path = format!("{}/debian/source/format", packaging_dir_debian);
        println!("Setting up quilt format for patching");
        fs::write(&debian_source_format_path, "3.0 (quilt)\n").map_err(|err| err.to_string())?;

        // Patch .pc dir setup
        let pc_version_path = format!("{}/.pc/.version", &packaging_dir_debian);
        println!("Creating necessary directories for patching");
        fs::create_dir_all(&format!("{}/.pc", &packaging_dir_debian))
            .map_err(|err| err.to_string())?;
        let mut pc_version_file =
            fs::File::create(&pc_version_path).map_err(|err| err.to_string())?;
        write!(pc_version_file, "2\n").map_err(|err| err.to_string())?;

        // Patch .pc patch version number
        let debian_control_path = format!("{}/debian/control", &packaging_dir_debian);
        println!("Adding Standards-Version to the control file");
        Command::new("bash")
            .arg("-c")
            .arg(format!("cd {} && head -n 3 control; echo \"Standards-Version: 4.5.1\"; tail -n +4 control; > control.new", &packaging_dir_debian))
            .output().map_err(|err| err.to_string())?;
        fs::rename(
            format!("{}/debian/control.new", &packaging_dir_debian),
            &debian_control_path,
        )
        .map_err(|err| err.to_string())?;

        let src_dir = "src";
        if fs::metadata(src_dir).is_err() {
            println!("Source directory 'src' not found. Skipping copy.");
        } else {
            println!(
                "Copying source directory {} to {}",
                src_dir, &packaging_dir_debian
            );
            // Copy the contents of `src` directory to `PACKAGING_DIR/debian`
            for entry in fs::read_dir(src_dir).map_err(|err| err.to_string())? {
                let entry = entry.map_err(|err| err.to_string())?;
                let src_path = entry.path();
                let dst_path = Path::new(&packaging_dir_debian).join(entry.file_name());
                fs::copy(&src_path, &dst_path)
                    .map(|_| ())
                    .map_err(|err| err.to_string())?
            }
        }

        Ok(true)
    }
}
