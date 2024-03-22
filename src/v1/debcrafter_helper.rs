use git2::Repository;
use log::info;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

// TODO use from crates.io
pub fn check_if_dpkg_parsechangelog_installed() -> Result<(), String> {
    let output = Command::new("which")
        .arg("dpkg-parsechangelog")
        .output()
        .expect("dpkg-parsechangelog is not installed");

    if !output.status.success() {
        return Err(format!(
            "dpkg-parsechangelog is not installed. Please install it"
        ));
    }
    Ok(())
}

pub fn check_if_installed() -> bool {
    match Command::new("which").arg("debcrafter").output() {
        Ok(output) => output.status.success(),
        Err(_) => false, // Assuming debcrafter is not installed if an error occurs
    }
}

pub fn install() -> Result<(), String> {
    let repo_dir = tempdir().expect("Failed to create temporary directory");

    // Path to the temporary directory
    let repo_dir_path = repo_dir.path();

    // Clone the Git repository into the temporary directory
    let repo_url = "https://github.com/Kixunil/debcrafter.git";
    Repository::clone(repo_url, repo_dir_path).map_err(|op| op.to_string())?;

    // Build the project
    let output = Command::new("cargo")
        .arg("build")
        .current_dir(repo_dir_path)
        .output()
        .expect("Failed to execute 'cargo build'");

    if !output.status.success() {
        return Err(format!("Cargo build failed: {:?}", output));
    }

    // Install the binary
    let output = Command::new("cargo")
        .arg("install")
        .arg("--path")
        .arg(repo_dir_path)
        .output()
        .expect("Failed to execute 'cargo install'");

    if !output.status.success() {
        return Err(format!("Cargo install failed: {:?}", output));
    }
    Ok(())
}

// pub fn check_version_compatibility(debcrafter_version: &str) -> Result<(), String> {
//     let output = Command::new("debcrafter")
//         .arg("--version")
//         .output()
//         .map_err(|_| "Failed to execute debcrafter --version")?;

//     let output_str = String::from_utf8_lossy(&output.stdout);

//     if !output.status.success() && output_str.contains(debcrafter_version) {
//         return Err("debcrafter does not have the required version".to_string());
//     }
//     Ok(())
// }

pub fn create_debian_dir(specification_file: &str) -> Result<PathBuf, String> {
    let debcrafter_dir = tempdir().expect("Failed to create temporary directory");

    let spec_file_path = fs::canonicalize(PathBuf::from(specification_file)).unwrap();
    let spec_dir = spec_file_path.parent().unwrap();
    if !spec_file_path.exists() {
        return Err(format!("{} spec_file doesn't exist", specification_file));
    }
    let spec_file_name = spec_file_path.file_name().unwrap();
    info!("spec dir{:?}", spec_dir.to_str().unwrap());
    info!("spec file{:?}", spec_file_name);
    let output = Command::new("debcrafter")
        .arg(spec_file_name)
        .current_dir(spec_dir)
        .arg(debcrafter_dir.path())
        .output()
        .map_err(|err| format!("Error invoking debcrafter command: {:?}", err))?;

    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        return Err(error_message.to_string());
    }
    let file_stem = spec_file_path.file_stem().ok_or("File stem not found")?;
    let debian_dir = debcrafter_dir.path().join(file_stem);

    Ok(debian_dir)
}

pub fn copy_debian_dir(tmp_debian_dir: PathBuf, target_dir: &str) -> Result<(), String> {
    info!(
        "Copying debian directory from {} to {}",
        tmp_debian_dir.to_str().unwrap(), &target_dir
    );
    fs::copy(tmp_debian_dir, &target_dir).map_err(|err| err.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // fn test_debcrafter_installed() {
    //     let result = check_if_installed();
    //     assert!(result.is_err());
    //     assert_eq!(result.unwrap_err(), "debcrafter is not installed");
    // }

    // #[test]
    // fn test_debcrafter_version_incombability() {
    //     let result = check_version_compatibility("1.0.0");
    //     assert!(result.is_err());
    //     assert_eq!(
    //         result.unwrap_err(),
    //         "debcrafter does not have the required version"
    //     );
    // }
}
