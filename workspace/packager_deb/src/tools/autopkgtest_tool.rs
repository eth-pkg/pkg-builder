use std::{fs::create_dir_all, path::PathBuf, process::Command};

use debian::{
    autopkgtest::Autopkgtest, autopkgtest_image::AutopkgtestImageBuilder, execute::Execute,
};
use log::{info, warn};
use types::{config::Architecture, distribution::Distribution};

use crate::{
    configs::autopkgtest_version::AutopkgtestVersion, misc::distribution::DistributionTrait,
    sbuild::SbuildError,
};

use super::tool_runner::{BuildTool, ToolRunner};

pub struct AutopkgtestToolArgs {
    pub (crate) version: AutopkgtestVersion,
    pub (crate) changes_file: PathBuf,
    pub (crate) codename: Distribution,
    pub (crate) deb_dir: PathBuf,
    pub (crate) test_deps: Vec<String>,
    pub (crate) image_path: Option<PathBuf>,
    pub (crate) cache_dir: PathBuf,
    pub (crate) arch: Architecture,
}

pub struct AutopkgtestTool {
    args: AutopkgtestToolArgs,
}

impl AutopkgtestTool {
    pub fn new(args: AutopkgtestToolArgs) -> Self {
        AutopkgtestTool { args }
    }
}

impl BuildTool for AutopkgtestTool {
    fn name(&self) -> &str {
        "autopkgtest"
    }
    fn check_tool_version(&self) -> Result<(), SbuildError> {
        let output = Command::new("apt")
            .args(vec!["list", "--installed", "autopkgtest"])
            .output()?;

        if !output.status.success() {
            return Err(SbuildError::GenericError(format!(
                "Failed to check {} version",
                self.name()
            )));
        }

        let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
        let actual_version = extract_version_from_apt_output(&stdout_str)?;

        match self.args.version.cmp(&actual_version) {
            std::cmp::Ordering::Less => warn!(
                "Using newer {} version ({}) than expected ({})",
                self.name(),
                actual_version,
                self.args.version
            ),
            std::cmp::Ordering::Greater => warn!(
                "Using older {} version ({}) than expected ({})",
                self.name(),
                actual_version,
                self.args.version
            ),
            std::cmp::Ordering::Equal => info!("{} versions match ({})", self.name(), self.args.version),
        }

        Ok(())
    }

    fn configure(&mut self, _runner: &mut ToolRunner) -> Result<(), SbuildError> {
        info!("Running prepare_autopkgtest_image");
        let builder = AutopkgtestImageBuilder::new()
            .codename(&self.args.codename)?
            .image_path(
                &self.args.cache_dir.display().to_string(),
                &self.args.codename,
                &self.args.arch,
            )
            .mirror(self.args.codename.get_repo_url())
            .arch(&self.args.arch);
        let image_path = builder.get_image_path().unwrap();
        let image_path_parent = image_path.parent().unwrap();
        if !image_path.exists() {
            create_dir_all(image_path_parent)?;

            builder.execute()?;
        }

        self.args.image_path = Some(image_path.clone());
        Ok(())
    }
    fn execute(&self) -> Result<(), SbuildError> {
        Autopkgtest::new()
            .changes_file(self.args.changes_file.to_str().ok_or(SbuildError::GenericError(
                "Invalid changes file path".to_string(),
            ))?)
            .no_built_binaries()
            .apt_upgrade()
            .test_deps_not_in_debian(&&self.args.test_deps)
            .qemu(self.args.image_path.clone().unwrap())
            .working_dir(&self.args.deb_dir)
            .execute()?;
        Ok(())
    }
}

fn extract_version_from_apt_output(output: &str) -> Result<AutopkgtestVersion, SbuildError> {
    let version = output
        .lines()
        .filter(|line| line.trim_start().starts_with("autopkgtest"))
        .flat_map(|line| line.split_whitespace())
        .find(|&word| {
            let has_digits = word.chars().any(|c| c.is_digit(10));
            let has_dot = word.contains('.');
            has_digits && has_dot
        })
        .ok_or_else(|| SbuildError::GenericError("Could not find version string".to_string()))?;

    // For Ubuntu-style versions (e.g., "5.38ubuntu1~24.04.1"), take only major.minor
    // For Debian-style versions (e.g., "5.20.3-1"), take the full version
    let cleaned_version = if version.contains("ubuntu") || version.contains('~') {
        // Ubuntu-style: take only major.minor
        version
            .split(|c: char| !c.is_digit(10) && c != '.')
            .next()
            .unwrap_or(version)
            .trim()
    } else {
        // Debian-style: take the full version
        version.trim()
    };

    let version = AutopkgtestVersion::try_from(cleaned_version)?;
    Ok(version)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configs::autopkgtest_version::AutopkgtestVersion;

    #[test]
    fn test_extract_version_from_apt_output_debian() {
        // Standard apt list output
        let output = "Listing... Done\nautopkgtest/stable,now 5.20 amd64 [installed]";
        let extracted = extract_version_from_apt_output(output);
        assert!(extracted.is_ok());
        let expected_version = AutopkgtestVersion::try_from("5.20").unwrap();
        assert_eq!(extracted.unwrap(), expected_version);

        // Multiple lines with version in the middle
        let output = "Listing... Done\npkg1/stable 1.0\nautopkgtest/stable,now 5.20.3-1 amd64 [installed]\npkg2/stable 2.0";
        let extracted = extract_version_from_apt_output(output);
        assert!(extracted.is_ok());
        let expected_version = AutopkgtestVersion::try_from("5.20.3-1").unwrap();
        assert_eq!(extracted.unwrap(), expected_version);

        // No version digit
        let output = "Listing... Done\nautopkgtest not installed";
        let extracted = extract_version_from_apt_output(output);
        assert!(extracted.is_err());

        // Empty output
        let output = "";
        let extracted = extract_version_from_apt_output(output);
        assert!(extracted.is_err());

        // Version with multiple dots
        let output = "autopkgtest/stable,now 5.20.3.2 amd64 [installed]";
        let extracted = extract_version_from_apt_output(output);

        assert!(extracted.is_err());
    }

    #[test]
    fn test_extract_version_from_apt_output_ubuntu() {
        // Standard apt list output
        let output =
            "Listing...\nautopkgtest/noble-updates,now 5.38ubuntu1~24.04.1 all [installed]";
        let extracted = extract_version_from_apt_output(output);
        assert!(extracted.is_ok());
        let expected_version = AutopkgtestVersion::try_from("5.38").unwrap();
        assert_eq!(extracted.unwrap(), expected_version);

        // Multiple lines with version in the middle
        let output = "Listing... Done\npkg1/stable 1.0\nListing...\nautopkgtest/noble-updates,now 5.38ubuntu1~24.04.1 all [installed]\npkg2/stable 2.0";
        let extracted = extract_version_from_apt_output(output);
        assert!(extracted.is_ok());
        let expected_version = AutopkgtestVersion::try_from("5.38").unwrap();
        assert_eq!(extracted.unwrap(), expected_version);

        // No version digit
        let output = "Listing... Done\nautopkgtest not installed";
        let extracted = extract_version_from_apt_output(output);
        assert!(extracted.is_err());

        // Empty output
        let output = "";
        let extracted = extract_version_from_apt_output(output);
        assert!(extracted.is_err());
    }
}
