use std::path::{Path, PathBuf};

use crate::installers::language_installer::LanguageInstaller;
use crate::pkg_config::{LanguageEnv, PackageType, PkgConfig};
use crate::pkg_config_verify::PkgVerifyConfig;
use cargo_metadata::semver::Version;
use debian::autopkgtest::{Autopkgtest, AutopkgtestError};
use debian::autopkgtest_image::{AutopkgtestImageBuilder, AutopkgtestImageError};
use debian::execute::Execute;
use debian::lintian::{Lintian, LintianError};
use debian::piuparts::{Piuparts, PiupartsError};
use debian::sbuild::{SbuildBuilder, SbuildCmdError};
use debian::sbuild_create_chroot::{SbuildCreateChroot, SbuildCreateChrootError};
use log::{info, warn};
use rand::random;
use sha1::{Digest, Sha1};
use std::fs::{self, create_dir_all};
use std::process::Command;
use std::{env, vec};
use thiserror::Error;
use types::distribution::Distribution;

pub struct Sbuild {
    pub(crate) config: PkgConfig,
    pub(crate) build_files_dir: String,
    pub(crate) cache_dir: PathBuf,
}

impl Sbuild {
    pub fn new(config: PkgConfig, build_files_dir: String) -> Self {
        let cache_dir = config
            .build_env
            .sbuild_cache_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from("~/.cache/sbuild"));

        Self {
            cache_dir,
            config,
            build_files_dir,
        }
    }
}

#[derive(Debug, Error)]
pub enum SbuildError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    SbuildCreateChrootError(#[from] SbuildCreateChrootError),

    #[error(transparent)]
    AutopkgtestImageError(#[from] AutopkgtestImageError),

    #[error(transparent)]
    PiupartsError(#[from] PiupartsError),

    #[error(transparent)]
    LintianError(#[from] LintianError),

    #[error(transparent)]
    AutopkgtestError(#[from] AutopkgtestError),

    #[error(transparent)]
    SbuildCmdError(#[from] SbuildCmdError),
    // Add an error variant for verification errors
    #[error("Verification error: {0}")]
    VerificationError(String),
    // Add a generic error variant
    #[error("{0}")]
    GenericError(String),
}

impl Sbuild {
    pub fn clean(&self) -> Result<(), SbuildError> {
        let cache_file = self.get_cache_file();
        info!("Cleaning cached build: {}", cache_file);
        remove_file_or_directory(&cache_file, false)?;
        Ok(())
    }

    pub fn create(&self) -> Result<(), SbuildError> {
        let temp_dir = env::temp_dir().join(format!("temp_{}", random::<u32>()));
        fs::create_dir(&temp_dir)?;

        let cache_file = self.get_cache_file();
        ensure_parent_dir(&cache_file)?;

        let codename = &self.config.build_env.codename;
        let repo_url = get_repo_url(&self.config.build_env.codename);

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

    pub fn package(&self) -> Result<(), SbuildError> {
        let codename = &self.config.build_env.codename;
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

    pub fn verify(&self, verify_config: PkgVerifyConfig) -> Result<(), SbuildError> {
        let output_dir =
            Path::new(&self.build_files_dir)
                .parent()
                .ok_or(SbuildError::GenericError(
                    "Invalid build files dir".to_string(),
                ))?;

        let errors: Vec<_> = verify_config
            .verify
            .package_hash
            .iter()
            .filter_map(|output| {
                let file_path = output_dir.join(&output.name);
                if !file_path.exists() {
                    return Some(format!("Verification file missing: {}", output.name));
                }

                let buffer = match std::fs::read(&file_path) {
                    Ok(buf) => buf,
                    Err(_) => return Some(format!("Failed to read file: {}", output.name)),
                };

                let actual_sha1 = match calculate_sha1(&buffer) {
                    Ok(hash) => hash,
                    Err(_) => {
                        return Some(format!("Failed to calculate hash for: {}", output.name))
                    }
                };

                (actual_sha1 != output.hash).then(|| {
                    format!(
                        "SHA1 mismatch for {}: expected {}, got {}",
                        output.name, output.hash, actual_sha1
                    )
                })
            })
            .collect();

        if errors.is_empty() {
            info!("Verification successful!");
            Ok(())
        } else {
            // Convert the error collection to a proper error
            Err(SbuildError::VerificationError(errors.join("; ")))
        }
    }

    pub fn run_lintian(&self) -> Result<(), SbuildError> {
        check_tool_version("lintian", &self.config.build_env.lintian_version)?;

        let changes_file = self.get_changes_file();
        let codename = &self.config.build_env.codename;

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

    pub fn run_piuparts(&self) -> Result<(), SbuildError> {
        info!("Running piuparts (requires sudo)...");
        check_tool_version("piuparts", &self.config.build_env.piuparts_version)?;

        let codename = &self.config.build_env.codename;
        let repo_url = get_repo_url(&codename);
        let keyring = get_keyring(&self.config.build_env.codename);
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

    pub fn run_autopkgtests(&self) -> Result<(), SbuildError> {
        info!("Running autopkgtests...");
        check_tool_version("autopkgtest", &self.config.build_env.autopkgtest_version)?;

        let codename = &self.config.build_env.codename;
        let image_path = self.prepare_autopkgtest_image(&codename)?;
        let changes_file = self.get_changes_file();

        Autopkgtest::new()
            .changes_file(changes_file.to_str().ok_or(SbuildError::GenericError(
                "Invalid changes file path".to_string(),
            ))?)
            .no_built_binaries()
            .apt_upgrade()
            .test_deps_not_in_debian(&&self.get_test_deps_not_in_debian())
            .qemu(
                image_path
                    .to_str()
                    .ok_or(SbuildError::GenericError("Invalid image path".to_string()))?,
            )
            .working_dir(self.get_deb_dir())
            .execute()?;
        Ok(())
    }
}

// Helper Methods for Sbuild
impl Sbuild {
    pub fn get_cache_file(&self) -> String {
        let dir = shellexpand::tilde(&self.cache_dir.display().to_string()).to_string();
        let codename = &self.config.build_env.codename.as_short();
        let cache_file_name = format!("{}-{}.tar.gz", codename, self.config.build_env.arch);
        Path::new(&dir)
            .join(cache_file_name)
            .to_str()
            .unwrap()
            .to_string()
    }

    pub fn get_deb_dir(&self) -> &Path {
        Path::new(&self.build_files_dir).parent().unwrap()
    }

    pub fn get_deb_name(&self) -> PathBuf {
        self.get_package_path_with_extension("deb")
    }

    pub fn get_changes_file(&self) -> PathBuf {
        self.get_package_path_with_extension("changes")
    }

    fn get_package_path_with_extension(&self, ext: &str) -> PathBuf {
        let deb_dir = self.get_deb_dir();
        let filename = format!(
            "{}_{}-{}_{}.{}",
            self.config.package_fields.package_name,
            self.config.package_fields.version_number,
            self.config.package_fields.revision_number,
            self.config.build_env.arch,
            ext
        );
        deb_dir.join(filename)
    }

    pub fn get_build_deps_not_in_debian(&self) -> Vec<String> {
        let lang_env = self.config.package_type.get_language_env();
        match lang_env {
            Some(env) => {
                let installer: Box<dyn LanguageInstaller> = env.into();
                installer
                    .get_build_deps(&self.config.build_env.arch, &self.config.build_env.codename)
            }
            None => vec![],
        }
    }

    pub fn get_test_deps_not_in_debian(&self) -> Vec<String> {
        let lang_env = self.config.package_type.get_language_env();
        match lang_env {
            Some(env) => {
                let installer: Box<dyn LanguageInstaller> = env.into();
                installer.get_test_deps(&self.config.build_env.codename)
            }
            None => vec![],
        }
    }
    fn build_chroot_setup_commands(&self) -> Vec<String> {
        let mut deps = self.get_build_deps_not_in_debian();
        if self.config.build_env.codename == Distribution::noble() {
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

    fn prepare_autopkgtest_image(&self, codename: &Distribution) -> Result<PathBuf, SbuildError> {
        info!("Running prepare_autopkgtest_image");
        let repo_url = get_repo_url(codename);
        let builder = AutopkgtestImageBuilder::new()
            .codename(codename)?
            .image_path(
                &self.cache_dir.display().to_string(),
                codename,
                &self.config.build_env.arch,
            )
            .mirror(repo_url)
            .arch(&self.config.build_env.arch);
        info!("Running prepare_autopkgtest_image 2");

        let image_path = builder.get_image_path().unwrap();
        let image_path_parent = image_path.parent().unwrap();
        if image_path.exists() {
            return Ok(image_path.clone());
        }

        create_dir_all(image_path_parent)?;

        builder.execute()?;
        Ok(image_path.clone())
    }
}

// Utility Functions

fn check_tool_version(tool: &str, expected_version: &Version) -> Result<(), SbuildError> {
    let (cmd, args) = match tool {
        "lintian" | "piuparts" => (tool, vec!["--version"]),
        "autopkgtest" => ("apt", vec!["list", "--installed", "autopkgtest"]),
        _ => {
            return Err(SbuildError::GenericError(format!(
                "Unsupported tool: {}",
                tool
            )))
        }
    };

    let output = Command::new(cmd).args(args).output()?;
    if !output.status.success() {
        return Err(SbuildError::GenericError(format!(
            "Failed to check {} version",
            tool
        )));
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

fn warn_compare_versions(expected: &Version, actual: &str, tool: &str) -> Result<(), SbuildError> {
    // Normalize version strings for parsing only, when not using semver
    // let expected_normalized = if expected.matches('.').count() == 1 {
    //     format!("{}.0", expected)
    // } else {
    //     expected.to_string()
    // };

    let actual_normalized = if actual.matches('.').count() == 1 {
        format!("{}.0", actual)
    } else {
        actual.to_string()
    };

    // let expected_ver = Version::parse(&expected_normalized).map_err(|e| {
    //     SbuildError::GenericError(format!("Failed parsing expected version: {}", e))
    // })?;
    let actual_ver = Version::parse(&actual_normalized)
        .map_err(|e| SbuildError::GenericError(format!("Failed to parse actual version: {}", e)))?;

    match expected.cmp(&actual_ver) {
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

fn get_keyring(codename: &Distribution) -> &str {
    match codename {
        Distribution::Debian(_) => "/usr/share/keyrings/debian-archive-keyring.gpg",
        Distribution::Ubuntu(_) => "/usr/share/keyrings/ubuntu-archive-keyring.gpg",
    }
}

fn get_repo_url(codename: &Distribution) -> &str {
    match codename {
        Distribution::Debian(_) => "http://deb.debian.org/debian",
        Distribution::Ubuntu(_) => "http://archive.ubuntu.com/ubuntu",
    }
}

fn calculate_sha1(data: &[u8]) -> Result<String, SbuildError> {
    let mut hasher = Sha1::new();
    hasher.update(data);
    Ok(hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect())
}

fn ensure_parent_dir<T: AsRef<Path>>(path: T) -> Result<(), SbuildError> {
    let parent = Path::new(path.as_ref())
        .parent()
        .ok_or(SbuildError::GenericError(format!(
            "Invalid path: {:?}",
            path.as_ref()
        )))?;
    create_dir_all(parent)?;
    Ok(())
}

fn remove_file_or_directory(path: &str, is_dir: bool) -> Result<(), SbuildError> {
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
    use crate::pkg_config::{BuildEnv, DefaultPackageTypeConfig, PackageFields};

    use super::*;
    use env_logger::Env;
    use types::url::Url;
    use std::fs::{create_dir_all, File};
    use std::path::Path;
    use std::sync::Once;
    use tempfile::tempdir;
    use types::distribution::Distribution;
    use types::version::Version;

    static INIT: Once = Once::new();

    // Initialize logger once for all tests
    fn setup() {
        INIT.call_once(|| {
            env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
        });
    }

    fn create_base_config() -> (PkgConfig, String, PathBuf) {
        let sbuild_cache_dir = tempdir().unwrap().path().to_path_buf();
        let config = PkgConfig {
            package_fields: PackageFields {
                spec_file: "".into(),
                package_name: "".into(),
                version_number: Version::try_from("1.0.0").unwrap(),
                revision_number: "".into(),
                homepage: "".into(),
            },
            package_type: PackageType::Default(DefaultPackageTypeConfig {
                tarball_url: Url::try_from("http://example.com").unwrap(),
                tarball_hash: Some("".into()),
                language_env: LanguageEnv::C,
            }),
            build_env: BuildEnv {
                codename: Distribution::bookworm(),
                arch: "amd64".into(),
                pkg_builder_version: Version::try_from("1.0.0").unwrap(),
                debcrafter_version:Version::try_from("1.0.0").unwrap(),
                sbuild_cache_dir: Some(sbuild_cache_dir.clone()),
                docker: None,
                run_lintian: None,
                run_piuparts: None,
                run_autopkgtest: None,
                lintian_version: Version::try_from("1.0.0").unwrap(),
                piuparts_version: Version::try_from("1.0.0").unwrap(),
                autopkgtest_version: Version::try_from("1.0.0").unwrap(),
                sbuild_version: Version::try_from("1.0.0").unwrap(),
                workdir: PathBuf::from(""),
            },
        };
        let build_files_dir = tempdir().unwrap().path().to_str().unwrap().to_string();

        (config, build_files_dir, sbuild_cache_dir)
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
    #[ignore = "Only run on CI"]
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
