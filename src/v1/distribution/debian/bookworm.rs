use std::fs;
use std::io::BufRead;
use std::io::BufReader;

use crate::v1::build::*;
use crate::v1::debcrafter_helper;
use crate::v1::packager::{BackendBuildEnv, Packager};

use crate::v1::distribution::debian::bookworm_config_builder::{
    BookwormPackagerConfig, PackageType,
};
use log::info;
use log::warn;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

pub struct BookwormPackager {
    config: BookwormPackagerConfig,
}

#[derive(Clone)]
pub struct BookwormBuildVariables {
    packaging_dir: String,
    tarball_path: String,
    package_source: String,
}

impl Packager for BookwormPackager {
    type Config = BookwormPackagerConfig;

    fn new(config: Self::Config) -> Self {
        BookwormPackager { config }
    }

    fn package(&self) -> Result<(), String> {
        let build_variables = BookwormBuildVariables {
            package_source: self.config.package_source(),
            tarball_path: self.config.tarball_path(),
            packaging_dir: self.config.packaging_dir(),
        };
        let pre_build: Result<(), String> = match &self.config.package_type() {
            PackageType::Default { tarball_url, .. } => {
                create_package_dir(&build_variables.packaging_dir.clone())?;
                download_source(&build_variables.tarball_path, tarball_url)?;
                extract_source(&build_variables)?;
                create_debian_dir(
                    &build_variables.package_source.clone(),
                    self.config.build_env().debcrafter_version(),
                    self.config.package_fields().spec_file(),
                )?;
                patch_source(
                    &build_variables.package_source.clone(),
                    self.config.package_fields().homepage(),
                )?;

                Ok(())
            }
            PackageType::Git { git_source, .. } => {
                create_package_dir(&build_variables.packaging_dir.clone())?;
                download_git(
                    &build_variables.packaging_dir,
                    &build_variables.tarball_path,
                    git_source,
                )?;
                extract_source(&build_variables)?;
                create_debian_dir(
                    &build_variables.package_source.clone(),
                    self.config.build_env().debcrafter_version(),
                    self.config.package_fields().spec_file(),
                )?;
                patch_source(
                    &build_variables.package_source.clone(),
                    self.config.package_fields().homepage(),
                )?;

                Ok(())
            }
            PackageType::Virtual => {
                info!("creating virtual package");
                create_package_dir(&build_variables.packaging_dir.clone())?;
                create_empty_tar(
                    &build_variables.packaging_dir,
                    &build_variables.tarball_path,
                )?;
                extract_source(&build_variables)?;
                create_debian_dir(
                    &build_variables.package_source.clone(),
                    self.config.build_env().debcrafter_version(),
                    self.config.package_fields().spec_file(),
                )?;
                patch_source(
                    &build_variables.package_source.clone(),
                    self.config.package_fields().homepage(),
                )?;
                Ok(())
            }
        };
        pre_build?;
        let build_env = self.get_build_env().unwrap();
        build_env.build()?;
        Ok(())
    }

    fn get_build_env(&self) -> Result<Box<dyn BackendBuildEnv>, String> {
        let backend_build_env = Box::new(Sbuild::new(self.config.clone()));
        Ok(backend_build_env)
    }
}

fn create_package_dir(packaging_dir: &String) -> Result<(), String> {
    info!("Creating package folder {}", &packaging_dir);
    fs::create_dir_all(packaging_dir).map_err(|err| err.to_string())?;
    Ok(())
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
        .args(["czvf", tarball_path, "--files-from", "/dev/null"])
        .current_dir(packaging_dir)
        .output()
        .map_err(|err| err.to_string())?;
    if !output.status.success() {
        return Err("Virtual package .tar.gz creation failed".to_string());
    }

    Ok(())
}

fn extract_source(build_variables: &BookwormBuildVariables) -> Result<(), String> {
    info!("Extracting source {}", &build_variables.packaging_dir);
    let output = Command::new("tar")
        .args([
            "zxvf",
            &build_variables.tarball_path,
            "-C",
            &build_variables.packaging_dir,
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
    info!(
        "Extracted source to package_source: {:?}",
        build_variables.package_source
    );

    Ok(())
}

fn create_debian_dir(
    package_source: &String,
    debcrafter_version: &String,
    spec_file: &str,
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

    debcrafter_helper::create_debian_dir(spec_file, package_source)?;
    info!(
        "Created /debian dir under package_source folder: {:?}",
        package_source
    );
    Ok(())
}

fn patch_source(package_source: &String, homepage: &String) -> Result<(), String> {
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
    fs::create_dir_all(format!("{}/.pc", &package_source))
        .map_err(|_| "Could not create .pc dir".to_string())?;
    let mut pc_version_file = fs::File::create(pc_version_path).map_err(|err| err.to_string())?;
    writeln!(pc_version_file, "2").map_err(|err| err.to_string())?;

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
    let homepage_line = format!("Homepage: {}", homepage);
    if !has_standards_version {
        let mut insert_index = 0;
        for (i, line) in original_content.iter().enumerate() {
            if line.starts_with("Priority:") {
                insert_index = i + 1;
                break;
            }
        }

        let mut updated_content = original_content.clone();
        updated_content.insert(insert_index, standards_version_line.to_string());
        updated_content.insert(insert_index + 1, homepage_line.to_string());

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
    fn test_create_package_dir_if_already_exists() {
        unreachable!("Test case not implemented yet");
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

        let result = download_source(tarball_path.to_str().unwrap(), &tarball_url);

        assert!(result.is_ok());
        assert!(tarball_path.exists());
    }

    #[test]
    fn test_download_source_with_git_package() {
        // TODO: Write test case for downloading source for a Git package
        unreachable!("Test case not implemented yet");
    }

    #[test]
    fn test_patch_src_dir() {
        // src patching is not implemented yet
        unreachable!("Test case not implemented yet");
    }

    #[test]
    fn test_patch_standards_version() {
        // src patching is not implemented yet
        unreachable!("Test case not implemented yet");
    }

    #[test]
    fn test_patch_homepage() {
        // src patching is not implemented yet
        unreachable!("Test case not implemented yet");
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
        let build_variables = BookwormBuildVariables {
            packaging_dir,
            package_source: packaging_source,
            tarball_path: tarball_path.to_string_lossy().to_string(),
        };
        let result = extract_source(&build_variables);

        assert!(result.is_ok(), "{:?}", result);

        let empty_file_path = PathBuf::from(build_variables.packaging_dir)
            .join(package_name)
            .join("empty_file.txt");

        assert!(
            empty_file_path.exists(),
            "Empty file not found after extraction"
        );
    }
}
