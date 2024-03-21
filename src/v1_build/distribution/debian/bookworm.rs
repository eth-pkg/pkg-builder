use std::fs;

use crate::v1_build::build::SbuildDotnet;
use crate::v1_build::build::SbuildGo;
use crate::v1_build::build::SbuildJava;
use crate::v1_build::build::SbuildNode;
use crate::v1_build::build::SbuildRust;
use crate::v1_build::build::SbuildZig;
use crate::v1_build::debcrafter_helper;
use crate::v1_build::packager::{
    BackendBuildEnv, BuildConfig, LanguageEnv, Packager, PackagerConfig
};

use log::info;
use std::io::Write;
use std::path::Path;
use std::process::Command;

pub struct BookwormPackager {
    // changing config will change the built package
    config: BookwormPackagerConfig,
    // options effect the process of building, but doesn't change the output
    // for example which directory you build your source
    // this flag don't affect the reproducibility
    options: BookwormPackagerOptions,
}

pub struct BookwormPackagerOptions {
    work_dir: String,
}
#[allow(dead_code)]
pub struct BookwormPackagerConfig {
    arch: String,
    package_name: String,
    version_number: String,
    tarball_url: String,
    git_source: String,
    is_virtual_package: bool,
    is_git: bool,
    lang_env: LanguageEnv,
    debcrafter_version: String,
}

#[derive(Debug, PartialEq)]
pub enum PackageType {
    InvalidCombination,
    NormalPackage,
    GitPackage,
    VirtualPackage,
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
    debcrafter_version: Option<String>,
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
            debcrafter_version: None,
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

    pub fn debcrafter_version(mut self, debcrafter_version: String) -> Self {
        self.debcrafter_version = Some(debcrafter_version);
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
        let debcrafter_version = self
            .debcrafter_version
            .ok_or_else(|| "Missing debcrafter_version field".to_string())?;

        Ok(BookwormPackagerConfig {
            arch,
            package_name,
            version_number,
            tarball_url,
            git_source,
            is_virtual_package: self.is_virtual_package,
            is_git: self.is_git,
            lang_env,
            debcrafter_version,
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
    fn package(&self) -> Result<(), String> {
        let packaging_dir = format!("{}/{}", self.options.work_dir, self.config.package_name);
        let tarball_path = format!(
            "{}/{}.orig.tar.gz",
            &packaging_dir, self.config.package_name
        );

        create_package_dir(&packaging_dir)?;
        download_source(
            &packaging_dir,
            &tarball_path,
            &self.config.tarball_url,
            self.config.is_virtual_package,
            self.config.is_git,
        )?;
        extract_source(&packaging_dir, &tarball_path)?;
        create_debian_dir(
            &packaging_dir,
            &self.config.debcrafter_version,
            &self.config.package_name,
            &self.config.version_number,
        )?;
        patch_source(&packaging_dir)?;

        let backend_build_env = self.create_build_env()?;
        backend_build_env.build()?;
        Ok(())
    }
}

fn create_package_dir(packaging_dir: &String) -> Result<(), String> {
    info!("Creating package folder {}", &packaging_dir);
    return fs::create_dir_all(&packaging_dir).map_err(|err| err.to_string());
}

pub fn download_source(
    packaging_dir: &str,
    tarball_path: &str,
    tarball_url: &str,
    is_virtual_package: bool,
    is_git: bool,
) -> Result<(), String> {
    let package_type = match (is_git, is_virtual_package) {
        (true, true) => PackageType::InvalidCombination,
        (false, false) => PackageType::NormalPackage,
        (true, false) => PackageType::GitPackage,
        (false, true) => PackageType::VirtualPackage,
    };

    match package_type {
        PackageType::InvalidCombination => {
            return Err("Invalid combination of package types".to_string());
        }
        PackageType::NormalPackage => {
            info!("Downloading source {}", tarball_path);
            let status = Command::new("wget")
                .arg("-O")
                .arg(tarball_path)
                .arg(tarball_url)
                .status()
                .map_err(|err| err.to_string())?;
            if !status.success() {
                return Err("Download failed".to_string());
            }
        }
        PackageType::GitPackage => todo!(),
        PackageType::VirtualPackage => {
            info!("Creating empty .tar.gz for virtual package");
            let output = Command::new("tar")
                .args(&["czvf", tarball_path, "--files-from", "/dev/null"])
                .current_dir(packaging_dir)
                .output()
                .map_err(|err| err.to_string())?;
            if !output.status.success() {
                return Err("Virtual package .tar.gz creation failed".to_string());
            }
        }
    }
    Ok(())
}

fn extract_source(packaging_dir: &String, tarball_path: &String) -> Result<(), String> {
    info!("Extracting source {}", &packaging_dir);
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
        let error_message = String::from_utf8(output.stderr)
            .unwrap_or_else(|_| "Unknown error occurred during extraction".to_string());
        return Err(format!("Extraction failed: {}", error_message));
    }
    println!("{:?}", output.status);
    Ok(())
}
fn create_debian_dir(
    packaging_dir: &String,
    debcrafter_version: &String,
    package_name: &String,
    version_number: &String,
) -> Result<(), String> {
    debcrafter_helper::check_if_installed()?;
    debcrafter_helper::check_version_compatibility(debcrafter_version)?;
    let tmp_debian_dir = debcrafter_helper::create_debian_dir(package_name)?;
    let target_dir = format!("{}/{}-{}/debian", packaging_dir, package_name, version_number);
    debcrafter_helper::copy_debian_dir(tmp_debian_dir, &target_dir)?;

    Ok(())
}
fn patch_source(packaging_dir: &String) -> Result<(), String> {
    let packaging_dir_debian: String = format!("{}/debian", packaging_dir);

    // Patch quilt
    let debian_source_format_path = format!("{}/debian/source/format", packaging_dir_debian);
    info!("Setting up quilt format for patching");
    fs::write(&debian_source_format_path, "3.0 (quilt)\n").map_err(|err| err.to_string())?;

    // Patch .pc dir setup
    let pc_version_path = format!("{}/.pc/.version", &packaging_dir_debian);
    info!("Creating necessary directories for patching");
    fs::create_dir_all(&format!("{}/.pc", &packaging_dir_debian)).map_err(|err| err.to_string())?;
    let mut pc_version_file = fs::File::create(&pc_version_path).map_err(|err| err.to_string())?;
    write!(pc_version_file, "2\n").map_err(|err| err.to_string())?;

    // Patch .pc patch version number
    let debian_control_path = format!("{}/debian/control", &packaging_dir_debian);
    info!("Adding Standards-Version to the control file");
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
        info!("Source directory 'src' not found. Skipping copy.");
    } else {
        info!(
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

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn setup_mock_server() -> MockServer {
        // Start the mock server
        let server = MockServer::start();

        // Mock the endpoint to serve the tarball file
        server.mock(|when, then| {
            when.method(GET).path("/test_package.tar.gz");
            then.status(200)
                .header("Content-Type", "application/octet-stream")
                .body_from_file("tests/test_package.tar.gz");
        });

        server
    }

    #[test]
    fn test_create_package_dir() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        let packaging_dir = temp_dir.path().join("test_package");

        let result = create_package_dir(&String::from(packaging_dir.to_str().unwrap()));

        assert!(result.is_ok());
        assert!(packaging_dir.exists());
    }

    #[test]
    fn test_download_source_virtual_package() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        let packaging_dir = String::from(temp_dir.path().to_str().unwrap());
        let tarball_name = "test_package.tar.gz";
        let tarball_path = temp_dir.path().join(tarball_name);
        let tarball_path_str = String::from(temp_dir.path().join(tarball_name).to_str().unwrap());
        let tarball_url = "http://example.com/test_package.tar.gz".to_string();
        let is_virtual_package = true;
        let is_git = false;

        let result = download_source(
            &packaging_dir,
            &tarball_path_str,
            &tarball_url,
            is_virtual_package,
            is_git,
        );

        assert!(result.is_ok());
        assert!(tarball_path.exists());
    }

    #[test]
    fn test_download_source_non_virtual_package() {
        let server = setup_mock_server();

        let temp_dir = tempdir().expect("Failed to create temporary directory");

        let packaging_dir = temp_dir.path().join("test_package");
        let tarball_name = "test_package.tar.gz";
        let tarball_path = temp_dir.path().join(tarball_name);
        let tarball_url = format!("{}/{}", server.base_url(), tarball_name);
        let is_virtual_package = false;
        let is_git = false;

        let result = download_source(
            &packaging_dir.to_string_lossy().to_string(),
            &tarball_path.to_string_lossy().to_string(),
            &tarball_url,
            is_virtual_package,
            is_git,
        );

        assert!(result.is_ok());
        assert!(tarball_path.exists());
    }

    #[test]
    fn test_download_source_with_invalid_combination() {
        let packaging_dir = "/tmp/test_package".to_string();
        let tarball_path = "/tmp/test_package.tar.gz".to_string();
        let tarball_url = "http://example.com/test_package.tar.gz".to_string();
        let is_virtual_package = true;
        let is_git = true;

        let result = download_source(
            &packaging_dir,
            &tarball_path,
            &tarball_url,
            is_virtual_package,
            is_git,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_download_source_with_git_package() {
        // TODO: Write test case for downloading source for a Git package
        assert!(false, "Test case not implemented yet");
    }

    #[test]
    fn test_extract_source() {
        let package_name = "test_package";
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let tarball_path: PathBuf = PathBuf::from("tests/test_package.tar.gz");

        let packaging_dir = temp_dir.path().to_string_lossy().to_string();

        assert!(tarball_path.exists());

        let result = extract_source(&packaging_dir, &tarball_path.to_string_lossy().to_string());

        assert!(result.is_ok(), "{:?}", result);

        let empty_file_path = PathBuf::from(packaging_dir)
            .join(package_name)
            .join("empty_file.txt");

        assert!(
            empty_file_path.exists(),
            "Empty file not found after extraction"
        );
    }
}
