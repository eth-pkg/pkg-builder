use git2::Repository;
use log::info;
use std::fs;
use std::io;
use std::path::Path;
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

pub fn create_debian_dir(specification_file: &str, target_dir: &str) -> Result<(), String> {
    let debcrafter_dir = tempdir().expect("Failed to create temporary directory");

    let spec_file_path = fs::canonicalize(PathBuf::from(specification_file)).unwrap();
    let spec_dir = spec_file_path.parent().unwrap();
    if !spec_file_path.exists() {
        return Err(format!("{} spec_file doesn't exist", specification_file));
    }
    let spec_file_name = spec_file_path.file_name().unwrap();
    info!("Spec directory: {:?}", spec_dir.to_str().unwrap());
    info!("Spec file: {:?}", spec_file_name);
    info!("Debcrafter directory: {:?}", debcrafter_dir);
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
    if let Some(first_directory) = get_first_directory(debcrafter_dir.path()) {
        let tmp_debian_dir = first_directory.join("debian");
        let dest_dir = Path::new(target_dir).join("debian");
        copy_dir_contents_recursive(&tmp_debian_dir, &dest_dir)
            .map_err(|err| err.to_string())?;
    } else {
        return Err("Unable to create debian dir.".to_string());
    }
    Ok(())
}

fn copy_dir_contents_recursive(src_dir: &Path, dest_dir: &Path) -> io::Result<()> {
    info!(
        "Copying directory: {:?} to {:?}",
        src_dir.display(),
        dest_dir.display()
    );

    if src_dir.is_dir() {
        if !dest_dir.exists() {
            fs::create_dir_all(dest_dir)?;
        }

        for entry in fs::read_dir(src_dir)? {
            let entry = entry?;
            let src_path = entry.path();
            let dest_path = dest_dir.join(entry.file_name());

            if src_path.is_dir() {
                copy_dir_contents_recursive(&src_path, &dest_path)?;
            } else {
                fs::copy(&src_path, &dest_path)?;
            }
        }
    }

    Ok(())
}

fn get_first_directory(dir: &Path) -> Option<PathBuf> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir).ok()? {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    return Some(path);
                }
            }
        }
    }
    None
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
