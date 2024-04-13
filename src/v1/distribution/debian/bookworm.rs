use std::io::BufRead;
use std::io::BufReader;
use std::{env, fs, io};

use crate::v1::build::sbuild::Sbuild;
use crate::v1::debcrafter_helper;
use crate::v1::packager::{BackendBuildEnv, Packager};

use eyre::{eyre, Result};

use crate::v1::pkg_config::{PackageType, PkgConfig};
use dirs::home_dir;
use log::info;
use log::warn;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

pub struct BookwormPackager {
    config: PkgConfig,
    source_to_patch_from_path: String,
    debian_artifacts_dir: String,
    debian_orig_tarball_path: String,
    build_files_dir: String,
    config_root: String,
}

impl Packager for BookwormPackager {
    type BuildEnv = Sbuild;

    fn new(config: PkgConfig, config_root: String) -> Self {
        let package_fields = config.package_fields.clone();
        let config_root_path = PathBuf::from(&config_root);
        let source_to_patch_from_path = config_root_path.join("src").to_str().unwrap().to_string();
        let workdir = config
            .build_env
            .workdir
            .clone()
            .unwrap_or("~/.pkg-builder/packages/bookworm".to_string());
        let workdir = expand_path(&workdir, None);
        let debian_artifacts_dir = get_build_artifacts_dir(&package_fields.package_name, &workdir);
        let debian_orig_tarball_path = get_tarball_path(
            &package_fields.package_name,
            &package_fields.version_number,
            &debian_artifacts_dir,
        );
        let build_files_dir = get_build_files_dir(
            &package_fields.package_name,
            &package_fields.version_number,
            &debian_artifacts_dir,
        );
        let mut updated_config = BookwormPackager {
            config,
            source_to_patch_from_path,
            build_files_dir,
            debian_artifacts_dir,
            debian_orig_tarball_path,
            config_root,
        };
        updated_config.config.build_env.workdir = Some(workdir);
        let spec_file = package_fields.spec_file;
        let spec_file_canonical = config_root_path.join(spec_file);
        updated_config.config.package_fields.spec_file =
            spec_file_canonical.to_str().unwrap().to_string();
        updated_config
    }

    fn package(&self) -> Result<()> {
        let pre_build: Result<()> = match &self.config.package_type {
            PackageType::Default(config) => {
                create_package_dir(&self.debian_artifacts_dir.clone())?;
                download_source(
                    &self.debian_orig_tarball_path,
                    &config.tarball_url,
                    &self.config_root,
                )?;
                extract_source(&self.debian_orig_tarball_path, &self.build_files_dir)?;
                create_debian_dir(
                    &self.build_files_dir.clone(),
                    &self.config.build_env.debcrafter_version,
                    &self.config.package_fields.spec_file,
                )?;
                patch_source(
                    &self.build_files_dir.clone(),
                    &self.config.package_fields.homepage,
                    &self.source_to_patch_from_path,
                )?;
                setup_sbuild()?;
                Ok(())
            }
            PackageType::Git(config) => {
                create_package_dir(&self.debian_artifacts_dir.clone())?;
                download_git(
                    &self.debian_artifacts_dir,
                    &self.debian_orig_tarball_path,
                    &config.git_url,
                )?;
                extract_source(&self.debian_orig_tarball_path, &self.build_files_dir)?;
                create_debian_dir(
                    &self.build_files_dir.clone(),
                    &self.config.build_env.debcrafter_version,
                    &self.config.package_fields.spec_file,
                )?;
                patch_source(
                    &self.build_files_dir.clone(),
                    &self.config.package_fields.homepage,
                    &self.source_to_patch_from_path,
                )?;
                setup_sbuild()?;
                Ok(())
            }
            PackageType::Virtual => {
                info!("creating virtual package");
                create_package_dir(&self.debian_artifacts_dir.clone())?;
                create_empty_tar(&self.debian_artifacts_dir, &self.debian_orig_tarball_path)?;
                extract_source(&self.debian_orig_tarball_path, &self.build_files_dir)?;
                create_debian_dir(
                    &self.build_files_dir.clone(),
                    &self.config.build_env.debcrafter_version,
                    &self.config.package_fields.spec_file,
                )?;
                patch_source(
                    &self.build_files_dir.clone(),
                    &self.config.package_fields.homepage,
                    &self.source_to_patch_from_path,
                )?;
                setup_sbuild()?;
                Ok(())
            }
        };
        pre_build?;
        let build_env = self.get_build_env().unwrap();
        build_env.build()?;
        Ok(())
    }

    fn get_build_env(&self) -> Result<Self::BuildEnv> {
        let backend_build_env = Sbuild::new(self.config.clone(), self.build_files_dir.clone());
        Ok(backend_build_env)
    }
}

fn create_package_dir(build_artifacts_dir: &String) -> Result<()> {
    if fs::metadata(build_artifacts_dir).is_ok() {
        info!("Remove previous package folder {}", &build_artifacts_dir);
        fs::remove_dir_all(build_artifacts_dir)?;
    }
    info!("Creating package folder {}", &build_artifacts_dir);
    fs::create_dir_all(build_artifacts_dir)?;
    Ok(())
}

fn download_source(tarball_path: &str, tarball_url: &str, config_root: &str) -> Result<()> {
    info!("Downloading source {}", tarball_path);
    let is_web = tarball_url.starts_with("http");
    let tarball_url = get_tarball_url(tarball_url, config_root);
    if is_web {
        info!(
            "Downloading tar: {} to location: {}",
            tarball_url, tarball_path
        );
        let status = Command::new("wget")
            .arg("-O")
            .arg(tarball_path)
            .arg(tarball_url)
            .status()?;
        if !status.success() {
            return Err(eyre!("Download failed".to_string()));
        }
    } else {
        info!("Copying tar: {} to location: {}", tarball_url, tarball_path);
        fs::copy(tarball_url, tarball_path)?;
    }
    Ok(())
}

#[allow(unused_variables)]
fn download_git(build_artifacts_dir: &str, tarball_path: &str, git_source: &str) -> Result<()> {
    todo!()
}

fn create_empty_tar(build_artifacts_dir: &str, tarball_path: &str) -> Result<()> {
    info!("Creating empty .tar.gz for virtual package");
    let output = Command::new("tar")
        .args(["czvf", tarball_path, "--files-from", "/dev/null"])
        .current_dir(build_artifacts_dir)
        .output()?;
    if !output.status.success() {
        return Err(eyre!("Virtual package .tar.gz creation failed".to_string(),));
    }

    Ok(())
}

fn extract_source(tarball_path: &str, build_files_dir: &str) -> Result<()> {
    info!("Extracting source {}", &build_files_dir);
    fs::create_dir_all(build_files_dir)?;

    let mut args = vec!["zxvf", &tarball_path, "-C", &build_files_dir];
    let numbers_to_strip = components_to_strip(tarball_path.to_string().clone());
    let numbers_to_strip = numbers_to_strip.unwrap_or_default();
    let strip = format!("--strip-components={}", numbers_to_strip);
    if numbers_to_strip > 0 {
        args.push(&strip);
    }
    info!("Stripping components: {} {:?}", numbers_to_strip, args);
    let output = Command::new("tar").args(args).output()?;
    if !output.status.success() {
        let error_message = String::from_utf8(output.stderr)
            .unwrap_or_else(|_| "Unknown error occurred during extraction".to_string());
        return Err(eyre!(error_message));
    }
    info!("Extracted source to build_files_dir: {:?}", build_files_dir);

    Ok(())
}

fn create_debian_dir(
    build_files_dir: &String,
    debcrafter_version: &String,
    spec_file: &str,
) -> Result<()> {
    debcrafter_helper::check_if_dpkg_parsechangelog_installed()?;
    if !debcrafter_helper::check_if_installed() {
        debcrafter_helper::install()?;
    }
    warn!(
        "Debcrafter version number is not checked! Expecting version number of: {}",
        debcrafter_version
    );
    // debcrafter_helper::check_version_compatibility(debcrafter_version)?;

    debcrafter_helper::create_debian_dir(spec_file, build_files_dir)?;
    info!(
        "Created /debian dir under build_files_dir folder: {:?}",
        build_files_dir
    );
    Ok(())
}

fn patch_source(build_files_dir: &String, homepage: &String, src_dir: &String) -> Result<()> {
    // Patch quilt
    let debian_source_format_path = format!("{}/debian/source/format", build_files_dir);
    info!(
        "Setting up quilt format for patching. Debian source format path: {}",
        debian_source_format_path
    );
    let debian_source_dir = PathBuf::from(&build_files_dir).join("debian/source");
    if !debian_source_dir.exists() {
        fs::create_dir_all(&debian_source_dir)?;
        info!(
            "Created debian/source directory at: {:?}",
            debian_source_dir
        );
    }

    if !Path::new(&debian_source_format_path).exists() {
        fs::write(&debian_source_format_path, "3.0 (quilt)\n")?;
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
    let pc_version_path = format!("{}/.pc/.version", &build_files_dir);
    info!("Creating necessary directories for patching");
    fs::create_dir_all(format!("{}/.pc", &build_files_dir))?;
    let mut pc_version_file = fs::File::create(pc_version_path)?;
    writeln!(pc_version_file, "2")?;

    // Patch .pc patch version number
    let debian_control_path = format!("{}/debian/control", build_files_dir);
    info!(
        "Adding Standards-Version to the control file. Debian control path: {}",
        debian_control_path
    );
    let input_file = fs::File::open(&debian_control_path)?;
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

        let mut output_file = fs::File::create(&debian_control_path)?;
        for line in updated_content {
            writeln!(output_file, "{}", line)?;
        }

        info!("Standards-Version added to the control file.");
    } else {
        info!("Standards-Version already exists in the control file. No changes made.");
    }

    // Only copy if src dir exists
    if fs::metadata(src_dir).is_ok() {
        copy_directory_recursive(Path::new(src_dir), Path::new(&build_files_dir))?;
    }
    // Get the current permissions of the file
    info!(
        "Adding executable permission for {}/debian/rules",
        build_files_dir
    );

    let debian_rules = format!("{}/debian/rules", build_files_dir);
    let mut permissions = fs::metadata(debian_rules.clone())?.permissions();
    permissions.set_mode(permissions.mode() | 0o111);
    fs::set_permissions(debian_rules, permissions)?;

    info!("Patching finished successfully!");
    Ok(())
}

fn setup_sbuild() -> Result<()> {
    let src_path = Path::new("overrides/.sbuildrc");
    let home_dir = home_dir().expect("Home dir is empty");
    let dest_path = home_dir.join(".sbuildrc");
    let contents = fs::read_to_string(src_path)?;

    if dest_path.exists() {
        let existing_contents = fs::read_to_string(&dest_path)?;
        return if existing_contents != contents {
            Err(eyre!(
                "Existing .sbuildrc file differs from expected content. Please backup your ~/.sbuildrc file. And rerun this script!",
            ))
        } else {
            Ok(())
        };
    }
    let mut home_dir = home_dir.to_str().unwrap_or("/home/runner").to_string();
    if home_dir == *"/home/runner" {
        home_dir = "/home/runner".to_string();
    }
    let replaced_contents = contents.replace("<HOME>", &home_dir);
    let mut file = fs::File::create(&dest_path)?;
    file.write_all(replaced_contents.as_bytes())?;

    Ok(())
}
fn copy_directory_recursive(src_dir: &Path, dest_dir: &Path) -> Result<(), io::Error> {
    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let entry_path = entry.path();
        let file_name = entry.file_name();

        let dest_path = dest_dir.join(&file_name);

        if entry_path.is_dir() {
            copy_directory_recursive(&entry_path, &dest_path)?;
        } else {
            fs::copy(&entry_path, &dest_path)?;
        }
    }

    Ok(())
}

fn components_to_strip(tar_gz_file: String) -> Result<usize, io::Error> {
    let output = Command::new("tar")
        .arg("--list")
        .arg("-z")
        .arg("-f")
        .arg(tar_gz_file)
        .output()?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = output_str.lines().filter(|l| !l.ends_with('/')).collect();

    let common_prefix = longest_common_prefix(&lines);

    let components_to_strip = common_prefix.split('/').filter(|&x| !x.is_empty()).count();

    Ok(components_to_strip)
}

fn longest_common_prefix(strings: &[&str]) -> String {
    if strings.is_empty() {
        return String::new();
    }

    let first_string = &strings[0];
    let mut prefix = String::new();

    'outer: for (i, c) in first_string.char_indices() {
        for string in &strings[1..] {
            if let Some(next_char) = string.chars().nth(i) {
                if next_char != c {
                    break 'outer;
                }
            } else {
                break 'outer;
            }
        }
        prefix.push(c);
    }

    prefix
}
pub fn get_build_artifacts_dir(package_name: &str, work_dir: &str) -> String {
    let build_artifacts_dir = format!("{}/{}", work_dir, &package_name);
    build_artifacts_dir
}
pub fn get_tarball_path(
    package_name: &str,
    version_number: &str,
    build_artifacts_dir: &str,
) -> String {
    let tarball_path = format!(
        "{}/{}_{}.orig.tar.gz",
        &build_artifacts_dir, &package_name, &version_number
    );
    tarball_path
}
pub fn get_build_files_dir(
    package_name: &str,
    version_number: &str,
    build_artifacts_dir: &str,
) -> String {
    let build_files_dir = format!(
        "{}/{}-{}",
        build_artifacts_dir, &package_name, &version_number
    );
    build_files_dir
}
pub fn get_tarball_url(tarball_url: &str, config_root: &str) -> String {
    if tarball_url.starts_with("http") {
        tarball_url.to_string()
    } else {
        expand_path(tarball_url, Some(config_root))
    }
}
pub fn expand_path(dir: &str, dir_to_expand: Option<&str>) -> String {
    if dir.starts_with('~') {
        let expanded_path = shellexpand::tilde(dir).to_string();
        expanded_path
    } else if dir.starts_with('/') {
        dir.to_string()
    } else {
        let parent_dir = match dir_to_expand {
            None => env::current_dir().unwrap(),
            Some(path) => PathBuf::from(path),
        };
        let dir = parent_dir.join(dir);
        let path = fs::canonicalize(dir.clone()).unwrap();
        let path = path.to_str().unwrap().to_string();
        path
    }
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
    #[ignore]
    fn test_create_package_dir() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        let build_artifacts_dir = temp_dir.path().join("test_package");

        let result = create_package_dir(&String::from(build_artifacts_dir.to_str().unwrap()));

        assert!(result.is_ok());
        assert!(build_artifacts_dir.exists());
    }
    #[test]
    #[ignore]
    fn test_create_package_dir_if_already_exists() {
        unreachable!("Test case not implemented yet");
    }

    #[test]
    #[ignore]
    fn test_download_source_virtual_package() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        let build_artifacts_dir = String::from(temp_dir.path().to_str().unwrap());
        let tarball_name = "test_package.tar.gz";
        let tarball_path = temp_dir.path().join(tarball_name);
        let tarball_path_str = String::from(temp_dir.path().join(tarball_name).to_str().unwrap());

        let result = create_empty_tar(&build_artifacts_dir, &tarball_path_str);

        assert!(result.is_ok());
        assert!(tarball_path.exists());
    }

    #[test]
    #[ignore]
    fn test_download_source_non_virtual_package() {
        let server = setup_mock_server();

        let temp_dir = tempdir().expect("Failed to create temporary directory");

        let tarball_name = "test_package.tar.gz";
        let tarball_path = temp_dir.path().join(tarball_name);
        let tarball_url = format!("{}/{}", server.base_url(), tarball_name);

        let result = download_source(tarball_path.to_str().unwrap(), &tarball_url, "/examples");

        assert!(result.is_ok());
        assert!(tarball_path.exists());
    }

    #[test]
    #[ignore]
    fn test_download_source_with_git_package() {
        // TODO: Write test case for downloading source for a Git package
        unreachable!("Test case not implemented yet");
    }

    #[test]
    #[ignore]
    fn test_patch_src_dir() {
        // src patching is not implemented yet
        unreachable!("Test case not implemented yet");
    }

    #[test]
    #[ignore]
    fn test_patch_standards_version() {
        // src patching is not implemented yet
        unreachable!("Test case not implemented yet");
    }

    #[test]
    #[ignore]

    fn test_patch_homepage() {
        // src patching is not implemented yet
        unreachable!("Test case not implemented yet");
    }

    #[test]
    #[ignore]

    fn test_extract_source() {
        let package_name = "test_package";
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let tarball_path: PathBuf = PathBuf::from("tests/test_package.tar.gz");

        let build_artifacts_dir = temp_dir.path().to_string_lossy().to_string();
        let packaging_source = temp_dir
            .path()
            .join("test_package")
            .to_string_lossy()
            .to_string();

        assert!(tarball_path.exists());

        let result = extract_source(&packaging_source, tarball_path.to_str().unwrap());

        assert!(result.is_ok(), "{:?}", result);

        let empty_file_path = PathBuf::from(build_artifacts_dir)
            .join(package_name)
            .join("empty_file.txt");

        assert!(
            empty_file_path.exists(),
            "Empty file not found after extraction"
        );
    }
}
