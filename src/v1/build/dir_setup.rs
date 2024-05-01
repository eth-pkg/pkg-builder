use std::io::BufRead;
use std::io::BufReader;
use std::{env, fs, io};


use eyre::{eyre, Result};

use crate::v1::pkg_config::{SubModule};
use dirs::home_dir;
use log::info;
use log::warn;
use std::io::{Write, Read};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use filetime::FileTime;
use sha2::{Digest, Sha256, Sha512};
use tempfile::tempdir;
use crate::v1::build::debcrafter_helper;

pub fn create_package_dir(build_artifacts_dir: &String) -> Result<()> {
    if fs::metadata(build_artifacts_dir).is_ok() {
        info!("Remove previous package folder {}", &build_artifacts_dir);
        fs::remove_dir_all(build_artifacts_dir)?;
    }
    info!("Creating package folder {}", &build_artifacts_dir);
    fs::create_dir_all(build_artifacts_dir)?;
    Ok(())
}

pub fn download_source(tarball_path: &str, tarball_url: &str, config_root: &str) -> Result<()> {
    info!("Downloading source {}", tarball_path);
    let is_web = tarball_url.starts_with("http");
    let tarball_url = get_tarball_url(tarball_url, config_root);
    if is_web {
        info!(
            "Downloading tar: {} to location: {}",
            tarball_url, tarball_path
        );
        let status = Command::new("wget")
            .arg("-q")
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

pub fn update_submodules(git_submodules: &Vec<SubModule>, current_dir: &str) -> Result<()> {
    // DO not use git2, it has very little git supported functionality
    // Initialize all submodules if they are not already initialized
    // Update submodules to specific commits
    for submodule in git_submodules.clone() {
        let output = Command::new("git")
            .current_dir(Path::new(current_dir).join(submodule.path.clone()))
            .args(&["checkout", &submodule.commit.clone()])
            .output()
            .map_err(|err| eyre!(format!("Failed to checkout submodule {}", err)))?;
        if !output.status.success() {
            return Err(eyre!(
                "Failed to checkout commit {} for submodule {}: {}",
                submodule.commit,
                submodule.path,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }

    Ok(())
}

pub fn clone_and_checkout_tag(git_url: &str, tag_version: &str, path: &str, git_submodules: &Vec<SubModule>) -> Result<()> {
    let output = Command::new("git")
        .args(&["clone", "--depth", "1", "--branch", tag_version, git_url, path])
        .output()
        .expect("Failed to execute git clone command");
    if !output.status.success() {
        return Err(eyre!(
            "Failed to checkout tag {}: {}",
            tag_version,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Initialize submodules
    let output = Command::new("git")
        .current_dir(path)
        .args(&["submodule", "update", "--init", "--recursive"])
        .output()?;

    if !output.status.success() {
        return Err(eyre!(
            "Failed to initialize submodules: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    update_submodules(git_submodules, path)?;

    Ok(())
}

fn set_creation_time<P: AsRef<Path>>(dir_path: P, timestamp: FileTime) -> io::Result<()> {
    let mut stack = vec![PathBuf::from(dir_path.as_ref())];

    while let Some(current) = stack.pop() {
        for entry in fs::read_dir(&current)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let file_path = entry.path();

            if file_type.is_dir() {
                stack.push(file_path);
            } else if file_type.is_file() {
                filetime::set_file_mtime(&file_path, timestamp)?;
            }
        }
    }

    Ok(())
}


pub fn download_git(build_artifacts_dir: &str, tarball_path: &str, git_url: &str, tag_version: &str, git_submodules: &Vec<SubModule>) -> Result<()> {
    let temporary_dir = tempdir()?;
    let path = temporary_dir.path();
    //let path = Path::new("/tmp/nimbus");
    clone_and_checkout_tag(git_url, tag_version, path.to_str().unwrap(), &git_submodules)?;
    // remove .git directory, no need to package it
    fs::remove_dir_all(path.join(".git"))?;

    // // Back in the path for reproducibility: January 1, 2022
    let timestamp = FileTime::from_unix_time(1640995200, 0);
    set_creation_time(path, timestamp)?;

    info!("Creating tar from git repo from {}", path.to_str().unwrap());
    let output = Command::new("tar")
        .args(&[
            "--sort=name",
            "--owner=0",
            "--group=0",
            "--numeric-owner",
            // does not work
            // "--mtime='2019-01-01 00:00'",
          //  "--pax-option=exthdr.name=%d/PaxHeaders/%f,delete=atime,delete=ctime",
            "-czvf", tarball_path, "-C", path.to_str().unwrap(), ".",
        ])
        .current_dir(build_artifacts_dir)
        .output()?;
    if !output.status.success() {
        return Err(eyre!(format!(
            "Failed to create tarball: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
            .into());
    }

    Ok(())
}

pub fn create_empty_tar(build_artifacts_dir: &str, tarball_path: &str) -> Result<()> {
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

pub fn calculate_sha512<R: Read>(mut reader: R) -> Result<String> {
    let mut hasher = Sha512::new();
    io::copy(&mut reader, &mut hasher)?;
    let digest_bytes = hasher.finalize();
    let hex_digest = digest_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();

    Ok(hex_digest)
}

pub fn calculate_sha256<R: Read>(mut reader: R) -> Result<String> {
    let mut hasher = Sha256::new();
    io::copy(&mut reader, &mut hasher)?;
    let digest_bytes = hasher.finalize();
    let hex_digest = digest_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();

    Ok(hex_digest)
}

pub fn verify_tarball_checksum(tarball_path: &str, expected_checksum: &str) -> Result<bool> {
    let mut file = fs::File::open(tarball_path).map_err(|_| eyre!("Could not open tarball."))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(|_| eyre!("Could not read tarball."))?;

    let actual_sha512 = calculate_sha512(&*buffer.clone()).unwrap_or_default();
    info!("sha512 hash {}", &actual_sha512);

    if actual_sha512 == expected_checksum {
        return Ok(true);
    }

    let actual_sha256 = calculate_sha256(&*buffer).unwrap_or_default();
    info!("sha256 hash {}", &actual_sha256);

    if actual_sha256 == expected_checksum {
        return Ok(true);
    }
    Err(eyre!("Hashes do not match."))
}

pub fn verify_hash(tarball_path: &str, expected_checksum: Option<String>) -> Result<()> {
    match expected_checksum {
        Some(tarball_hash) => {
            match verify_tarball_checksum(tarball_path, &tarball_hash) {
                Ok(true) => Ok(()),
                Ok(false) => Err(eyre!("Checksum is invalid.")),
                Err(err) => Err(eyre!("Error checking hash: {}", err)),
            }
        }
        None => Ok(()), // If no checksum is provided, consider it verified
    }
}

pub fn extract_source(tarball_path: &str, build_files_dir: &str) -> Result<()> {
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

pub fn create_debian_dir(
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

pub fn patch_quilt(build_files_dir: &String) -> Result<()> {
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
    Ok(())
}

pub fn patch_pc_dir(build_files_dir: &String) -> Result<()> {
    let pc_version_path = format!("{}/.pc/.version", &build_files_dir);
    info!("Creating necessary directories for patching");
    fs::create_dir_all(format!("{}/.pc", &build_files_dir))?;
    let mut pc_version_file = fs::File::create(pc_version_path)?;
    writeln!(pc_version_file, "2")?;
    Ok(())
}

pub fn patch_standards_version(build_files_dir: &String, homepage: &String) -> Result<()> {
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
    Ok(())
}

pub fn copy_src_dir(build_files_dir: &String, src_dir: &String) -> Result<()> {
    let src_dir_path = Path::new(src_dir);
    if src_dir_path.exists() {
        copy_directory_recursive(Path::new(src_dir), Path::new(&build_files_dir))
            .map_err(|err| eyre!(format!("Failed to copy src directory: {}", err)))?;
    }
    Ok(())
}

pub fn patch_rules_permission(build_files_dir: &str) -> Result<()> {
    info!(
        "Adding executable permission for {}/debian/rules",
        build_files_dir
    );

    let debian_rules = format!("{}/debian/rules", build_files_dir);
    let mut permissions = fs::metadata(debian_rules.clone())
        .map_err(|_| eyre!("Failed to get debian/rules permission."))?.permissions();
    permissions.set_mode(permissions.mode() | 0o111);
    fs::set_permissions(debian_rules, permissions).map_err(|_| eyre!("Failed to set debian/rules permission."))?;
    Ok(())
}

pub fn patch_source(build_files_dir: &String, homepage: &String, src_dir: &String) -> Result<()> {
    // Patch quilt
    patch_quilt(build_files_dir)?;

    // Patch .pc dir setup
    patch_pc_dir(build_files_dir)?;

    // Patch .pc patch version number
    patch_standards_version(build_files_dir, homepage)?;

    // Only copy if src dir exists
    copy_src_dir(build_files_dir, src_dir)?;

    patch_rules_permission(build_files_dir)?;

    info!("Patching finished successfully!");
    Ok(())
}

pub fn setup_sbuild() -> Result<()> {
    let home_dir = home_dir().expect("Home dir is empty");
    let dest_path = home_dir.join(".sbuildrc");
    let content = include_str!(".sbuildrc");
    let home_dir = home_dir.to_str().unwrap_or("/home/runner").to_string();
    let replaced_contents = content.replace("<HOME>", &home_dir);
    let mut file = fs::File::create(dest_path).map_err(|_| eyre!("Failed to create ~/.sbuildrc."))?;
    file.write_all(replaced_contents.as_bytes()).map_err(|_| eyre!("Failed to write ~/.sbuildrc."))?;

    Ok(())
}

pub fn copy_directory_recursive(src_dir: &Path, dest_dir: &Path) -> Result<(), io::Error> {
    if !dest_dir.exists() {
        fs::create_dir_all(dest_dir)?;
    }

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

pub fn components_to_strip(tar_gz_file: String) -> Result<usize, io::Error> {
    let output = Command::new("tar")
        .arg("--list")
        .arg("-z")
        .arg("-f")
        .arg(tar_gz_file)
        .output()?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = output_str.lines().filter(|l| !l.ends_with('/')).collect();

    let common_prefix = longest_common_prefix(&lines.clone());

    let components_to_strip = common_prefix.split('/').filter(|&x| !x.is_empty()).count();

    Ok(components_to_strip)
}

pub fn longest_common_prefix(strings: &[&str]) -> String {
    if strings.is_empty() {
        return String::new();
    }
    if strings.len() == 1 {
        let mut path_buf = PathBuf::from(strings[0]);
        path_buf.pop();
        let common_prefix = path_buf.to_string_lossy().to_string();
        return common_prefix;
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
        let path = fs::canonicalize(dir.clone()).unwrap_or(dir);
        let path = path.to_str().unwrap().to_string();
        path
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use super::*;
    use httpmock::prelude::*;
    use std::path::PathBuf;
    // use std::sync::Once;
    // use env_logger::Env;
    use tempfile::tempdir;
    use crate::v1::pkg_config::{PackageType, PkgConfig};

    // static INIT: Once = Once::new();

    fn setup() {
        // INIT.call_once(|| {
        //     env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
        // });
    }

    fn setup_mock_server() -> MockServer {
        // Start the mock server
        let server = MockServer::start();

        // Mock the endpoint to serve the tarball file
        server.mock(|when, then| {
            when.method(GET).path("/test_package.tar.gz");
            then.status(200)
                .header("Content-Type", "application/octet-stream")
                .body_from_file("tests/misc/test_package.tar.gz");
        });

        server
    }

    #[test]
    fn expand_path_expands_tilde_correctly() {
        setup();
        let result = expand_path("~", None);
        assert_ne!(result, "~");
        assert!(!result.contains('~'));
    }

    #[test]
    fn expand_path_handles_absolute_paths() {
        setup();
        let result = expand_path("/absolute/path", None);
        assert_eq!(result, "/absolute/path");
    }

    #[test]
    fn expand_path_expands_relative_paths_with_parent() {
        setup();
        let result = expand_path("somefile", Some("/tmp"));
        assert_eq!(result, "/tmp/somefile");
    }

    #[test]
    fn expand_path_expands_relative_paths_without_parent() {
        setup();
        let result = expand_path("somefile", None);
        assert!(result.starts_with('/'));
    }

    #[test]
    fn test_create_package_dir() {
        setup();

        let temp_dir = tempdir().expect("Failed to create temporary directory");

        let build_artifacts_dir = temp_dir.path().join("test_package");

        let result = create_package_dir(&String::from(build_artifacts_dir.to_str().unwrap()));

        assert!(result.is_ok());
        assert!(build_artifacts_dir.exists());
    }

    #[test]
    fn test_create_package_dir_if_already_exists() {
        setup();

        let temp_dir = tempdir().expect("Failed to create temporary directory");

        let build_artifacts_dir = temp_dir.path().join("test_package");
        let result = fs::create_dir(build_artifacts_dir.clone());
        assert!(result.is_ok());
        let test_file = build_artifacts_dir.clone().join("test_file");
        File::create(test_file.clone()).expect("Failed to create test_file");
        assert!(test_file.clone().exists());
        let result = create_package_dir(&String::from(build_artifacts_dir.to_str().unwrap()));

        assert!(result.is_ok());
        assert!(!test_file.clone().exists());
        assert!(build_artifacts_dir.exists());
    }

    #[test]
    fn test_download_source_virtual_package() {
        setup();

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
    fn test_download_source_non_virtual_package() {
        setup();

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
    fn test_download_source_with_git_package() {}


    #[test]
    fn test_extract_source() {
        setup();
        let package_name = "test_package";
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let temp_dir = temp_dir.path();
        let tarball_path: PathBuf = PathBuf::from("tests/misc/test_package.tar.gz");

        let build_files_dir = temp_dir
            .join(package_name)
            .to_string_lossy()
            .to_string();

        assert!(tarball_path.exists());
        let result = extract_source(tarball_path.to_str().unwrap(), &build_files_dir);

        assert!(result.is_ok(), "{:?}", result);
        assert!(Path::new(&build_files_dir).exists());

        let test_file_path = PathBuf::from(build_files_dir.clone())
            .join("empty_file.txt");

        assert!(
            test_file_path.exists(),
            "Empty file not found after extraction"
        );
    }

    #[test]
    fn patch_rules_permission_adds_exec_permission() -> Result<(), Box<dyn std::error::Error>> {
        setup();

        let temp_dir = tempdir()?;
        let rules_path = temp_dir.path().join("debian/rules");
        fs::create_dir_all(temp_dir.path().join("debian")).expect("Could not create dir");
        File::create(&rules_path)?;

        patch_rules_permission(temp_dir.path().to_str().unwrap())?;

        let permissions = fs::metadata(&rules_path)?.permissions();
        assert_ne!(permissions.mode() & 0o111, 0);

        Ok(())
    }

    #[test]
    fn patch_rules_permission_handles_nonexistent_directory() {
        setup();

        let result = patch_rules_permission("/nonexistent/dir");

        assert!(result.is_err());
    }

    #[test]
    fn patch_quilt_creates_source_dir_and_format_file() -> Result<(), Box<dyn std::error::Error>> {
        setup();

        let temp_dir = tempdir()?;
        let build_files_dir = temp_dir.path().to_str().unwrap().to_string();

        patch_quilt(&build_files_dir)?;

        let debian_source_dir = temp_dir.path().join("debian/source");
        assert!(debian_source_dir.exists());

        let debian_source_format_path = temp_dir.path().join("debian/source/format");
        let format_content = fs::read_to_string(debian_source_format_path)?;
        assert_eq!(format_content, "3.0 (quilt)\n");

        Ok(())
    }

    #[test]
    fn patch_quilt_skips_creation_if_already_exists() -> Result<(), Box<dyn std::error::Error>> {
        setup();

        let temp_dir = tempdir()?;
        let temp_dir = temp_dir.path();
        let build_files_dir = temp_dir.to_str().unwrap().to_string();

        fs::create_dir_all(temp_dir.join("debian/source")).expect("Failed to create dir for test.");
        File::create(temp_dir.join("debian/source/format")).expect("Failed to create file.");

        let result = patch_quilt(&build_files_dir);
        assert!(result.is_ok());

        let entries: Vec<_> = fs::read_dir(temp_dir)?.collect();
        assert_eq!(entries.len(), 1);

        Ok(())
    }


    #[test]
    fn test_verify_hash_valid_checksum_512() {
        setup();
        let tarball_path = "tests/misc/test_package.tar.gz";
        let expected_checksum = "abd0b8e99f983926dbf60bdcbaef13f83ec7b31d56e68f6252ed05981b237c837044ce768038fc34b71f925e2fb19b7dee451897db512bb4a99e0e1bc96d8ab3";

        let result = verify_hash(tarball_path, Some(expected_checksum.to_string()));

        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_hash_invalid_checksum_512() {
        setup();
        let tarball_path = "tests/misc/test_package.tar.gz";
        let expected_checksum = "abd0b8e99f983926dbf60bdcbaef13f83ec7b31d56e68f6252ed05981b237c837044ce768038fc34b71f925e2fb19b7dee451897db512bb4a99e0e1bc96d8ab2";

        let result = verify_hash(tarball_path, Some(expected_checksum.to_string()));

        assert!(result.is_err());
        assert_eq!(result.err().unwrap().to_string(), "Error checking hash: Hashes do not match.");
    }

    #[test]
    fn test_verify_hash_valid_checksum_256() {
        setup();
        let tarball_path = "tests/misc/test_package.tar.gz";
        let expected_checksum = "b610e83c026d4c465636779240b6ed40a076593a61df5f6b9f9f59f1a929478d";

        let result = verify_hash(tarball_path, Some(expected_checksum.to_string()));

        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_hash_invalid_checksum_256() {
        setup();
        let tarball_path = "tests/misc/test_package.tar.gz";
        let expected_checksum = "b610e83c026d4c465636779240b6ed40a076593a61df5f6b9f9f59f1a929478_";

        let result = verify_hash(tarball_path, Some(expected_checksum.to_string()));

        assert!(result.is_err());
        assert_eq!(result.err().unwrap().to_string(), "Error checking hash: Hashes do not match.");
    }

    #[test]
    fn test_clone_and_checkout_tag() {
        let url = "https://github.com/status-im/nimbus-eth2.git";
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let repo_path = temp_dir.path();
        let repo_path_str = repo_path.to_str().unwrap();
        let tag_version = "v24.3.0";
        let str = fs::read_to_string("examples/bookworm/git-package/nimbus/pkg-builder.toml")
            .expect("File does not exist");
        let config: PkgConfig = toml::from_str(&str)
            .expect("Cannot parse file.");
        match config.package_type {
            PackageType::Git(gitconfig) => {
                let result = clone_and_checkout_tag(url, tag_version, repo_path_str, &gitconfig.submodules);
                assert!(result.is_ok(), "Failed to clone and checkout tag: {:?}", result);
            }
            _ => panic!("Wrong type of file."),
        }


        fs::remove_dir_all(temp_dir).unwrap();
    }
}
