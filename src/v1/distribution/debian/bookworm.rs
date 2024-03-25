use std::fs;
use std::io::BufRead;
use std::io::BufReader;

use crate::v1::build::*;
use crate::v1::debcrafter_helper;
use crate::v1::packager::{BackendBuildEnv, BuildConfig, LanguageEnv, Packager, PackagerConfig};

use log::info;
use log::warn;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
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
pub struct NormalPackageConfig {
    arch: String,
    package_name: String,
    version_number: String,
    tarball_url: String,
    lang_env: LanguageEnv,
    debcrafter_version: String,
    spec_file: String,
}
pub struct GitPackageConfig {
    arch: String,
    package_name: String,
    version_number: String,
    git_source: String,
    lang_env: LanguageEnv,
    debcrafter_version: String,
    spec_file: String,
}

pub struct VirtualPackageConfig {
    arch: String,
    package_name: String,
    version_number: String,
    debcrafter_version: String,
    spec_file: String,
}

pub enum BookwormPackagerConfig {
    InvalidCombination,
    NormalPackage(NormalPackageConfig),
    GitPackage(GitPackageConfig),
    VirtualPackage(VirtualPackageConfig),
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
    spec_file: Option<String>,
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
            spec_file: None,
        }
    }

    pub fn arch(mut self, arch: Option<String>) -> Self {
        self.arch = arch;
        self
    }

    pub fn package_name(mut self, package_name: Option<String>) -> Self {
        self.package_name = package_name;
        self
    }

    pub fn version_number(mut self, version_number: Option<String>) -> Self {
        self.version_number = version_number;
        self
    }

    pub fn tarball_url(mut self, tarball_url: Option<String>) -> Self {
        self.tarball_url = tarball_url;
        self
    }

    pub fn git_source(mut self, git_source: Option<String>) -> Self {
        self.git_source = git_source;
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

    pub fn lang_env(mut self, lang_env: Option<String>) -> Self {
        self.lang_env = LanguageEnv::from_string(&lang_env.unwrap_or_default());
        self
    }

    pub fn debcrafter_version(mut self, debcrafter_version: Option<String>) -> Self {
        self.debcrafter_version = debcrafter_version;
        self
    }

    pub fn spec_file(mut self, spec_file: Option<String>) -> Self {
        self.spec_file = spec_file;
        self
    }

    pub fn config(self) -> Result<BookwormPackagerConfig, String> {
        if self.is_virtual_package && self.is_git {
            return Ok(BookwormPackagerConfig::InvalidCombination);
        }
        let arch = self.arch.ok_or_else(|| "Missing arch field".to_string())?;
        let package_name = self
            .package_name
            .ok_or_else(|| "Missing package_name field".to_string())?;
        let version_number = self
            .version_number
            .ok_or_else(|| "Missing version_number field".to_string())?;

        let debcrafter_version = self
            .debcrafter_version
            .ok_or_else(|| "Missing debcrafter_version field".to_string())?;
        let spec_file = self
            .spec_file
            .ok_or_else(|| "Missing spec_file field".to_string())?;
        if self.is_virtual_package {
            let config = VirtualPackageConfig {
                arch,
                package_name,
                version_number,
                debcrafter_version,
                spec_file,
            };
            return Ok(BookwormPackagerConfig::VirtualPackage(config));
        }
        let lang_env = self
            .lang_env
            .ok_or_else(|| "Missing lang_env field".to_string())?;
        if self.is_git {
            let git_source = self
                .git_source
                .ok_or_else(|| "Missing git_source field".to_string())?;
            let config = GitPackageConfig {
                arch,
                package_name,
                version_number,
                lang_env,
                debcrafter_version,
                git_source: git_source,
                spec_file,
            };
            return Ok(BookwormPackagerConfig::GitPackage(config));
        } else {
            let tarball_url = self
                .tarball_url
                .ok_or_else(|| "Missing tarball_url field".to_string())?;
            let config = NormalPackageConfig {
                arch,
                package_name,
                version_number,
                tarball_url,
                lang_env,
                debcrafter_version,
                spec_file,
            };
            Ok(BookwormPackagerConfig::NormalPackage(config))
        }
    }
}

impl Packager for BookwormPackager {
    type Config = BookwormPackagerConfig;

    fn new(config: Self::Config) -> Self {
        return BookwormPackager {
            config,
            options: BookwormPackagerOptions {
                work_dir: "/tmp/pkg-builder".to_string(),
            },
        };
    }

    fn package(&self) -> Result<(), String> {
        match &self.config {
            BookwormPackagerConfig::InvalidCombination => {
                return Err(
                    "Invalid combination is_git and is_virtual_package is not supported"
                        .to_string(),
                )
            }
            BookwormPackagerConfig::NormalPackage(config) => {
                let packaging_dir = format!("{}/{}", self.options.work_dir, config.package_name);
                let tarball_path =
                    format!("{}/{}.orig.tar.gz", &packaging_dir, config.package_name);
                let package_source = format!(
                    "{}/{}-{}",
                    packaging_dir, config.package_name, config.version_number
                );
                let result =
                    build_normal_package(config, &packaging_dir, &tarball_path, &package_source);
                result
            }
            BookwormPackagerConfig::GitPackage(config) => {
                let packaging_dir = format!("{}/{}", self.options.work_dir, config.package_name);
                let tarball_path =
                    format!("{}/{}.orig.tar.gz", &packaging_dir, config.package_name);
                let package_source = format!(
                    "{}/{}-{}",
                    packaging_dir, config.package_name, config.version_number
                );
                let result: Result<(), String> =
                    build_git_package(config, &packaging_dir, &tarball_path, &package_source);
                result
            }
            BookwormPackagerConfig::VirtualPackage(config) => {
                let packaging_dir = format!("{}/{}", self.options.work_dir, config.package_name);
                let tarball_path =
                    format!("{}/{}.orig.tar.gz", &packaging_dir, config.package_name);
                let package_source = format!(
                    "{}/{}-{}",
                    packaging_dir, config.package_name, config.version_number
                );
                let result: Result<(), String> =
                    build_virtual_package(config, &packaging_dir, &tarball_path, &package_source);
                result
            }
        }
    }
}

fn build_normal_package(
    config: &NormalPackageConfig,
    packaging_dir: &String,
    tarball_path: &String,
    package_source: &String,
) -> Result<(), String> {
    create_package_dir(&packaging_dir)?;
    download_source(&tarball_path, &config.tarball_url)?;
    extract_source(&packaging_dir, &tarball_path, &package_source)?;
    create_debian_dir(
        package_source,
        &config.debcrafter_version,
        &config.spec_file,
    )?;
    patch_source(&package_source)?;

    let build_config = BuildConfig::new("bookworm", &config.arch, Some(config.lang_env));
    let backend_build_env = Sbuild::new(build_config);
    backend_build_env.build()?;
    return Ok(());
}

fn build_git_package(
    config: &GitPackageConfig,
    packaging_dir: &String,
    tarball_path: &String,
    package_source: &String,
) -> Result<(), String> {
    create_package_dir(&packaging_dir)?;
    download_git(&packaging_dir, &tarball_path, &config.git_source)?;
    extract_source(&packaging_dir, &tarball_path, &package_source)?;
    create_debian_dir(
        package_source,
        &config.debcrafter_version,
        &config.spec_file,
    )?;
    patch_source(&package_source)?;
    let build_config = BuildConfig::new("bookworm", &config.arch, Some(config.lang_env));
    let backend_build_env = Sbuild::new(build_config);
    backend_build_env.build()?;

    return Ok(());
}

fn build_virtual_package(
    config: &VirtualPackageConfig,
    packaging_dir: &String,
    tarball_path: &String,
    package_source: &String,
) -> Result<(), String> {
    info!("creating virtual package");
    create_package_dir(&packaging_dir)?;
    create_empty_tar(&packaging_dir, &tarball_path)?;
    extract_source(&packaging_dir, &tarball_path, &package_source)?;
    create_debian_dir(
        package_source,
        &config.debcrafter_version,
        &config.spec_file,
    )?;
    patch_source(&package_source)?;
    let build_config = BuildConfig::new("bookworm", &config.arch, None);
    let backend_build_env = Sbuild::new(build_config);
    backend_build_env.clean()?;
    backend_build_env.create()?;
    backend_build_env.build()?;
    return Ok(());
}

fn create_package_dir(packaging_dir: &String) -> Result<(), String> {
    info!("Creating package folder {}", &packaging_dir);
    return fs::create_dir_all(&packaging_dir).map_err(|err| err.to_string());
}

fn download_source(tarball_path: &str, tarball_url: &str) -> Result<(), String> {
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
    Ok(())
}
#[allow(unused_variables)]
fn download_git(packaging_dir: &str, tarball_path: &str, git_source: &str) -> Result<(), String> {
    todo!()
}
fn create_empty_tar(packaging_dir: &str, tarball_path: &str) -> Result<(), String> {
    info!("Creating empty .tar.gz for virtual package");
    let output = Command::new("tar")
        .args(&["czvf", tarball_path, "--files-from", "/dev/null"])
        .current_dir(packaging_dir)
        .output()
        .map_err(|err| err.to_string())?;
    if !output.status.success() {
        return Err("Virtual package .tar.gz creation failed".to_string());
    }

    Ok(())
}

fn extract_source(
    packaging_dir: &String,
    tarball_path: &String,
    package_source: &String,
) -> Result<(), String> {
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
    info!("Extracted source to package_source: {:?}", package_source);

    Ok(())
}
fn create_debian_dir(
    package_source: &String,
    debcrafter_version: &String,
    spec_file: &String,
) -> Result<(), String> {
    debcrafter_helper::check_if_dpkg_parsechangelog_installed()?;
    if !debcrafter_helper::check_if_installed() {
        debcrafter_helper::install()?;
    }
    warn!(
        "Debcrafter version number is not checked! Expecting version number of: {}",
        debcrafter_version
    );
    // debcrafter_helper::check_version_compatibility(debcrafter_version)?;

    debcrafter_helper::create_debian_dir(&spec_file, &package_source)?;
    info!(
        "Created /debian dir under package_source folder: {:?}",
        package_source
    );
    Ok(())
}
fn patch_source(package_source: &String) -> Result<(), String> {
    // Patch quilt
    let debian_source_format_path = format!("{}/debian/source/format", package_source);
    info!(
        "Setting up quilt format for patching. Debian source format path: {}",
        debian_source_format_path
    );
    let debian_source_dir = PathBuf::from(&package_source).join("debian/source");
    if !debian_source_dir.exists() {
        fs::create_dir_all(&debian_source_dir)
            .map_err(|_| "Failed to create debian/source dir".to_string())?;
        info!(
            "Created debian/source directory at: {:?}",
            debian_source_dir
        );
    }

    if !Path::new(&debian_source_format_path).exists() {
        fs::write(&debian_source_format_path, "3.0 (quilt)\n")
            .map_err(|err| format!("Error writing file: {}", err))?;
        info!(
            "Quilt format file created at: {}",
            debian_source_format_path
        );
    } else {
        info!(
            "Quilt format file already exists at: {}",
            debian_source_format_path
        );
    }

    // Patch .pc dir setup
    let pc_version_path = format!("{}/.pc/.version", &package_source);
    info!("Creating necessary directories for patching");
    fs::create_dir_all(&format!("{}/.pc", &package_source))
        .map_err(|_| "Could not create .pc dir".to_string())?;
    let mut pc_version_file = fs::File::create(&pc_version_path).map_err(|err| err.to_string())?;
    write!(pc_version_file, "2\n").map_err(|err| err.to_string())?;

    // Patch .pc patch version number
    let debian_control_path = format!("{}/debian/control", package_source);
    info!(
        "Adding Standards-Version to the control file. Debian control path: {}",
        debian_control_path
    );
    let input_file = fs::File::open(&debian_control_path).map_err(|err| err.to_string())?;
    let reader = BufReader::new(input_file);

    let original_content: Vec<String> = reader.lines().map(|line| line.unwrap()).collect();

    let has_standards_version = original_content
        .iter()
        .any(|line| line.starts_with("Standards-Version"));
    let standards_version_line = "Standards-Version: 4.5.1";
    if !has_standards_version {
        let mut insert_index = 0;
        for (i, line) in original_content.iter().enumerate() {
            if line.starts_with("Package:") {
                insert_index = i + 1;
                break;
            }
        }

        let mut updated_content = original_content.clone();
        updated_content.insert(insert_index, standards_version_line.to_string());

        let mut output_file =
            fs::File::create(&debian_control_path).map_err(|err| err.to_string())?;
        for line in updated_content {
            writeln!(output_file, "{}", line).map_err(|err| err.to_string())?;
        }

        info!("Standards-Version added to the control file.");
    } else {
        info!("Standards-Version already exists in the control file. No changes made.");
    }

    // let src_dir = "src";
    // if fs::metadata(src_dir).is_err() {
    //     info!("Source directory 'src' not found. Skipping copy.");
    // } else {
    //     info!(
    //         "Copying source directory {} to {}",
    //         src_dir, &package_source
    //     );
    //     for entry in fs::read_dir(src_dir).map_err(|err| err.to_string())? {
    //         let entry = entry.map_err(|err| err.to_string())?;
    //         let src_path = entry.path();
    //         let dst_path = Path::new(&package_source).join(entry.file_name());
    //         fs::copy(&src_path, &dst_path)
    //             .map(|_| ())
    //             .map_err(|err| err.to_string())?
    //     }
    // }
    info!("Patching finished successfully!");
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

        let result = create_empty_tar(&packaging_dir, &tarball_path_str);

        assert!(result.is_ok());
        assert!(tarball_path.exists());
    }

    #[test]
    fn test_download_source_non_virtual_package() {
        let server = setup_mock_server();

        let temp_dir = tempdir().expect("Failed to create temporary directory");

        let tarball_name = "test_package.tar.gz";
        let tarball_path = temp_dir.path().join(tarball_name);
        let tarball_url = format!("{}/{}", server.base_url(), tarball_name);

        let result = download_source(&tarball_path.to_string_lossy().to_string(), &tarball_url);

        assert!(result.is_ok());
        assert!(tarball_path.exists());
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
        let packaging_source = temp_dir
            .path()
            .join("test_package")
            .to_string_lossy()
            .to_string();

        assert!(tarball_path.exists());

        let result = extract_source(
            &packaging_dir,
            &tarball_path.to_string_lossy().to_string(),
            &packaging_source,
        );

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
