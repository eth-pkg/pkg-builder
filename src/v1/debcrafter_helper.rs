use log::info;
use std::fs;
use std::process::Command;

// TODO use from crates.io
pub fn check_if_installed() -> Result<(), String> {
    Command::new("debcrafter")
        .output()
        .map_err(|_| "debcrafter is not installed")?;

    Ok(())
}

pub fn check_version_compatibility(debcrafter_version: &str) -> Result<(), String> {
    let output = Command::new("debcrafter")
        .arg("--version")
        .output()
        .map_err(|_| "Failed to execute debcrafter --version")?;

    let output_str = String::from_utf8_lossy(&output.stdout);

    if !output.status.success() && output_str.contains(debcrafter_version) {
        return Err("debcrafter does not have the required version".to_string());
    }
    Ok(())
}

pub fn create_debian_dir(specification_file: &str) -> Result<&str, String> {
    let output = Command::new("debcrafter")
        .arg(specification_file)
        .arg("/tmp")
        .output()
        .map_err(|err| err.to_string())?;

    if !output.status.success() {
        return Err("Debcrafter failed".to_string());
    }
    Ok("/tmp/package_name")
}

pub fn copy_debian_dir(tmp_debian_dir: &str, target_dir: &str) -> Result<(), String> {
    info!(
        "Copying debian directory from {} to {}",
        &tmp_debian_dir, &target_dir
    );
    fs::copy(tmp_debian_dir, &target_dir).map_err(|err| err.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debcrafter_installed() {
        let result = check_if_installed();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "debcrafter is not installed");
    }

    #[test]
    fn test_debcrafter_version_incombability() {
        let result = check_version_compatibility("1.0.0");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "debcrafter does not have the required version"
        );
    }
}
