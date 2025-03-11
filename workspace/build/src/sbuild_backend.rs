use crate::sbuild::normalize_codename;
use cargo_metadata::semver::Version;
use common::build::BackendBuildEnv;
use common::pkg_config::{LanguageEnv, PackageType};
use common::pkg_config_verify::PkgVerifyConfig;
use debian::autopkgtest::Autopkgtest;
use debian::autopkgtest_image::AutopkgtestImageBuilder;
use debian::execute::Execute;
use debian::lintian::Lintian;
use debian::piuparts::Piuparts;
use debian::sbuild::SbuildBuilder;
use debian::sbuild_create_chroot::SbuildCreateChroot;
use eyre::{eyre, Context, Result};
use log::{info, warn};
use rand::random;
use sha1::{Digest, Sha1};
use std::fs::{self, create_dir_all};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, vec};

use super::sbuild::Sbuild;

impl BackendBuildEnv for Sbuild {
    fn clean(&self) -> Result<()> {
        let cache_file = self.get_cache_file();
        info!("Cleaning cached build: {}", cache_file);
        remove_file_or_directory(&cache_file, false)?;
        Ok(())
    }

    fn create(&self) -> Result<()> {
        let temp_dir = env::temp_dir().join(format!("temp_{}", random::<u32>()));
        fs::create_dir(&temp_dir)?;

        let cache_file = self.get_cache_file();
        ensure_parent_dir(&cache_file)?;

        let codename = normalize_codename(&self.config.build_env.codename)?;
        let repo_url = get_repo_url(&self.config.build_env.codename)?;

        SbuildCreateChroot::new()
            .chroot_mode("unshare")
            .make_tarball()
            .cache_file(&cache_file)
            .codename(codename)
            .temp_dir(&temp_dir)
            .repo_url(repo_url)
            .execute()?;

        Ok(())
    }

    fn package(&self) -> Result<()> {
        let codename = normalize_codename(&self.config.build_env.codename)?;
        let cache_file = self.get_cache_file();
        let build_chroot_setup_commands = self.build_chroot_setup_commands();
        let run_lintian = self.config.build_env.run_lintian.unwrap_or(false);
        let run_piuparts = self.config.build_env.run_piuparts.unwrap_or(false);
        let run_autopkgtest = self.config.build_env.run_autopkgtest.unwrap_or(false);
        let builder = SbuildBuilder::new()
            .distribution(codename)
            .build_arch_all()
            .build_source()
            .cache_file(&cache_file)
            .verbose()
            .chroot_mode_unshare()
            .setup_commands(&build_chroot_setup_commands)
            .no_run_piuparts()
            .no_apt_upgrades()
            .run_lintian(run_lintian)
            .no_run_autopkgtest()
            .working_dir(Path::new(&self.build_files_dir));

        builder.execute()?;

        if run_piuparts {
            self.run_piuparts()?;
        }
        if run_autopkgtest {
            self.run_autopkgtests()?;
        }
        Ok(())
    }

    fn verify(&self, verify_config: PkgVerifyConfig) -> Result<()> {
        let output_dir = Path::new(&self.build_files_dir)
            .parent()
            .ok_or(eyre!("Invalid build files dir"))?;

        let errors: Vec<_> = verify_config
            .verify
            .package_hash
            .iter()
            .filter_map(|output| {
                let file_path = output_dir.join(&output.name);
                if !file_path.exists() {
                    return Some(eyre!("Verification file missing: {}", output.name));
                }

                let buffer = std::fs::read(&file_path).ok()?;
                let actual_sha1 = calculate_sha1(&buffer).ok()?;

                (actual_sha1 != output.hash).then(|| {
                    eyre!(
                        "SHA1 mismatch for {}: expected {}, got {}",
                        output.name,
                        output.hash,
                        actual_sha1
                    )
                })
            })
            .collect();

        if errors.is_empty() {
            info!("Verification successful!");
            Ok(())
        } else {
            Err(errors
                .into_iter()
                .fold(eyre!("Verification failed"), |acc, err| acc.wrap_err(err)))
        }
    }

    fn run_lintian(&self) -> Result<()> {
        check_tool_version("lintian", &self.config.build_env.lintian_version)?;

        let changes_file = self.get_changes_file();
        let codename = normalize_codename(&self.config.build_env.codename)?;

        Lintian::new()
            .suppress_tag("bad-distribution-in-changes-file")
            .info()
            .extended_info()
            .changes_file(changes_file)
            .tag_display_limit(0)
            .fail_on_warning()
            .fail_on_error()
            .suppress_tag("debug-file-with-no-debug-symbols")
            .with_codename(codename)
            .execute()?;
        Ok(())
    }

    fn run_piuparts(&self) -> Result<()> {
        info!("Running piuparts (requires sudo)...");
        check_tool_version("piuparts", &self.config.build_env.piuparts_version)?;

        let codename = normalize_codename(&self.config.build_env.codename)?;
        let repo_url = get_repo_url(&self.config.build_env.codename)?;
        let keyring = get_keyring(&self.config.build_env.codename)?;
        let deb_name = self.get_deb_name();

        Piuparts::new()
            .distribution(codename)
            .mirror(repo_url)
            .bindmount_dev()
            .keyring(keyring)
            .verbose()
            .with_dotnet_env(
                matches!(self.language_env(), Some(LanguageEnv::Dotnet(_))),
                codename,
            )
            .deb_file(&deb_name)
            .deb_path(self.get_deb_dir())
            .execute()?;

        Ok(())
    }

    fn run_autopkgtests(&self) -> Result<()> {
        info!("Running autopkgtests...");
        check_tool_version("autopkgtest", &self.config.build_env.autopkgtest_version)?;

        let codename = normalize_codename(&self.config.build_env.codename)?;
        let image_path = self.prepare_autopkgtest_image(&codename)?;
        let changes_file = self.get_changes_file();

        Autopkgtest::new()
            .changes_file(
                changes_file
                    .to_str()
                    .ok_or(eyre!("Invalid changes file path"))?,
            )
            .no_built_binaries()
            .apt_upgrade()
            .test_deps_not_in_debian(&self.get_build_deps_not_in_debian())
            .qemu(image_path.to_str().ok_or(eyre!("Invalid image path"))?)
            .working_dir(self.get_deb_dir())
            .execute()?;
        Ok(())
    }
}

// Helper Methods for Sbuild
impl Sbuild {
    fn build_chroot_setup_commands(&self) -> Vec<String> {
        let mut deps = self.get_build_deps_not_in_debian();
        if self.config.build_env.codename == "noble numbat" {
            deps.extend(vec![
                "apt install -y software-properties-common".to_string(),
                "add-apt-repository universe".to_string(),
                "add-apt-repository restricted".to_string(),
                "add-apt-repository multiverse".to_string(),
                "apt update".to_string(),
            ]);
        }
        deps.into_iter()
            .map(|dep| format!("--chroot-setup-commands={}", dep))
            .collect()
    }

    fn language_env(&self) -> Option<&LanguageEnv> {
        match &self.config.package_type {
            PackageType::Default(config) => Some(&config.language_env),
            PackageType::Git(config) => Some(&config.language_env),
            PackageType::Virtual => None,
        }
    }

    fn prepare_autopkgtest_image(&self, codename: &str) -> Result<PathBuf> {
        info!("Running prepare_autopkgtest_image");
        let repo_url = get_repo_url(&self.config.build_env.codename)?;
        let builder = AutopkgtestImageBuilder::new()
            .codename(codename)?
            .image_path(&self.cache_dir, codename, &self.config.build_env.arch)
            .mirror(repo_url)
            .arch(&self.config.build_env.arch);
        let image_path = builder.get_image_path().unwrap();
        let image_path_parent = image_path.parent().unwrap();
        if image_path.exists(){
            return Ok(image_path.clone());
        }
        create_dir_all(image_path_parent)?;

        builder.execute()?;
        Ok(image_path.clone())
    }
}

// Utility Functions

fn check_tool_version(tool: &str, expected_version: &str) -> Result<()> {
    let (cmd, args) = match tool {
        "lintian" | "piuparts" => (tool, vec!["--version"]),
        "autopkgtest" => ("apt", vec!["list", "--installed", "autopkgtest"]),
        _ => return Err(eyre!("Unsupported tool: {}", tool)),
    };

    let output = Command::new(cmd).args(args).output()?;
    if !output.status.success() {
        return Err(eyre!("Failed to check {} version", tool));
    }

    let version_str = String::from_utf8_lossy(&output.stdout);
    let actual_version = match tool {
        "lintian" => version_str
            .replace("Lintian v", "")
            .split("ubuntu")
            .next()
            .unwrap_or("")
            .trim()
            .to_string(),
        "piuparts" => version_str.replace("piuparts ", "").trim().to_string(),
        "autopkgtest" => version_str
            .split_whitespace()
            .find(|s| s.chars().next().unwrap_or(' ').is_digit(10))
            .map(|v| {
                v.chars()
                    .take_while(|c| c.is_digit(10) || *c == '.')
                    .collect()
            })
            .unwrap_or_default(),
        _ => unreachable!(),
    };
    info!(
        "versions: expected:{} actual:{}",
        expected_version, actual_version
    );
    warn_compare_versions(expected_version, &actual_version.trim(), tool)?;
    Ok(())
}

fn warn_compare_versions(expected: &str, actual: &str, tool: &str) -> Result<()> {
    // Normalize version strings for parsing only, when not using semver
    let expected_normalized = if expected.matches('.').count() == 1 {
        format!("{}.0", expected)
    } else {
        expected.to_string()
    };

    let actual_normalized = if actual.matches('.').count() == 1 {
        format!("{}.0", actual)
    } else {
        actual.to_string()
    };

    let expected_ver =
        Version::parse(&expected_normalized).context("Failed parsing expected version")?;
    let actual_ver =
        Version::parse(&actual_normalized).context("Failed to parse actual version")?;

    match expected_ver.cmp(&actual_ver) {
        std::cmp::Ordering::Less => warn!(
            "Using newer {} version ({}) than expected ({})",
            tool, actual, expected
        ),
        std::cmp::Ordering::Greater => warn!(
            "Using older {} version ({}) than expected ({})",
            tool, actual, expected
        ),
        std::cmp::Ordering::Equal => info!("{} versions match ({})", tool, expected),
    }

    Ok(())
}

fn get_keyring(codename: &str) -> Result<&str> {
    match codename {
        "bookworm" => Ok("/usr/share/keyrings/debian-archive-keyring.gpg"),
        "noble numbat" | "jammy jellyfish" => Ok("/usr/share/keyrings/ubuntu-archive-keyring.gpg"),
        _ => Err(eyre!("Unsupported codename: {}", codename)),
    }
}

fn get_repo_url(codename: &str) -> Result<&str> {
    match codename {
        "bookworm" => Ok("http://deb.debian.org/debian"),
        "noble numbat" | "jammy jellyfish" => Ok("http://archive.ubuntu.com/ubuntu"),
        _ => Err(eyre!("Unsupported codename: {}", codename)),
    }
}

fn calculate_sha1(data: &[u8]) -> Result<String> {
    let mut hasher = Sha1::new();
    hasher.update(data);
    Ok(hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect())
}

fn ensure_parent_dir<T: AsRef<Path>>(path: T) -> Result<()> {
    let parent = Path::new(path.as_ref())
        .parent()
        .ok_or(eyre!("Invalid path: {:?}", path.as_ref()))?;
    create_dir_all(parent)?;
    Ok(())
}

fn remove_file_or_directory(path: &str, is_dir: bool) -> Result<()> {
    if Path::new(path).exists() {
        if is_dir {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::pkg_config::PkgConfig;
    use env_logger::Env;
    use std::fs::{create_dir_all, File};
    use std::path::Path;
    use std::sync::Once;
    use tempfile::tempdir;

    static INIT: Once = Once::new();

    // Initialize logger once for all tests
    fn setup() {
        INIT.call_once(|| {
            env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
        });
    }

    fn create_base_config() -> (PkgConfig, String, String) {
        let mut config = PkgConfig::default();
        config.build_env.codename = "bookworm".to_string();
        config.build_env.arch = "amd64".to_string();

        let build_files_dir = tempdir().unwrap().path().to_str().unwrap().to_string();
        let cache_dir = tempdir().unwrap().path().to_str().unwrap().to_string();
        config.build_env.sbuild_cache_dir = Some(cache_dir.clone());

        (config, build_files_dir, cache_dir)
    }

    #[test]
    fn test_clean_when_file_missing() {
        setup();
        let (config, build_files_dir, _) = create_base_config();
        let build_env = Sbuild::new(config, build_files_dir);

        let result = build_env.clean();
        let cache_file = build_env.get_cache_file();

        assert!(result.is_ok());
        assert!(!Path::new(&cache_file).exists());
    }

    #[test]
    fn test_clean_with_existing_file() {
        setup();
        let (config, build_files_dir, cache_dir) = create_base_config();

        let build_env = Sbuild::new(config, build_files_dir);
        let cache_file = build_env.get_cache_file();

        create_dir_all(cache_dir).unwrap();
        File::create(&cache_file).unwrap();
        assert!(Path::new(&cache_file).exists());

        let result = build_env.clean();
        assert!(result.is_ok());
        assert!(!Path::new(&cache_file).exists());
    }

    #[test]
    fn test_create_environment() {
        setup();
        let (config, build_files_dir, _) = create_base_config();
        let build_env = Sbuild::new(config, build_files_dir);

        build_env.clean().unwrap();
        let cache_file = build_env.get_cache_file();
        assert!(!Path::new(&cache_file).exists());

        let result = build_env.create();
        assert!(result.is_ok());
        assert!(Path::new(&cache_file).exists());
    }
}
