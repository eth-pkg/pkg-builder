use crate::v1::build::sbuild::normalize_codename;
use crate::v1::packager::BackendBuildEnv;
use crate::v1::pkg_config::{LanguageEnv, PackageType};
use crate::v1::pkg_config_verify::PkgVerifyConfig;
use cargo_metadata::semver::Version;
use eyre::{eyre, Result};
use log::{info, warn};
use rand::random;
use sha1::{Digest, Sha1};
use std::ffi::OsStr;
use std::fs::{self, create_dir_all, File};
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
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

        execute_command(
            "sbuild-createchroot",
            &[
                "--chroot-mode=unshare",
                "--make-sbuild-tarball",
                &cache_file,
                &codename,
                temp_dir.to_str().ok_or(eyre!("Invalid temp dir path"))?,
                repo_url,
            ],
            None,
        )?;
        Ok(())
    }

    fn package(&self) -> Result<()> {
        let codename = normalize_codename(&self.config.build_env.codename)?;
        let cache_file = self.get_cache_file();

        let mut args = vec![
            "-d",
            codename,
            "-A", // Build architecture all
            "-s", // Build source
            "--source-only-changes",
            "-c",
            &cache_file,
            "-v", // Verbose
            "--chroot-mode=unshare",
        ];
        let build_chroot_setup_commands = self.build_chroot_setup_commands();
        args.extend(
            build_chroot_setup_commands
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>(),
        );
        args.extend([
            "--no-run-piuparts",
            "--no-apt-upgrade",
            "--no-apt-distupgrade",
        ]);

        if self.config.build_env.run_lintian.unwrap_or(false) {
            args.extend([
                "--run-lintian",
                "--lintian-opt=-i",
                "--lintian-opt=--I",
                "--lintian-opt=--suppress-tags",
                "--lintian-opt=bad-distribution-in-changes-file",
                "--lintian-opt=--suppress-tags",
                "--lintian-opt=debug-file-with-no-debug-symbols",
                "--lintian-opt=--tag-display-limit=0",
                "--lintian-opts=--fail-on=error",
                "--lintian-opts=--fail-on=warning",
            ]);
        } else {
            args.push("--no-run-lintian");
        }
        args.push("--no-run-autopkgtest");

        info!("Building package with: sbuild {}", args.join(" "));
        execute_command("sbuild", &args, Some(Path::new(&self.build_files_dir)))?;

        if self.config.build_env.run_piuparts.unwrap_or(false) {
            self.run_piuparts()?;
        }
        if self.config.build_env.run_autopkgtest.unwrap_or(false) {
            self.run_autopkgtests()?;
        }
        Ok(())
    }

    fn verify(&self, verify_config: PkgVerifyConfig) -> Result<()> {
        let output_dir = Path::new(&self.build_files_dir)
            .parent()
            .ok_or(eyre!("Invalid build files dir"))?;
        let mut errors = Vec::new();

        for output in verify_config.verify.package_hash {
            let file_path = output_dir.join(&output.name);
            if !file_path.exists() {
                return Err(eyre!("Verification file missing: {}", output.name));
            }

            let mut file = File::open(&file_path)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            let actual_sha1 = calculate_sha1(&buffer)?;

            if actual_sha1 != output.hash {
                errors.push(eyre!(
                    "SHA1 mismatch for {}: expected {}, got {}",
                    output.name,
                    output.hash,
                    actual_sha1
                ));
            }
        }

        if errors.is_empty() {
            println!("Verification successful!");
            Ok(())
        } else {
            Err(errors
                .into_iter()
                .fold(eyre!("Verification failed"), |acc, err| acc.wrap_err(err)))
        }
    }

    fn run_lintian(&self) -> Result<()> {
        info!("Running lintian (standalone mode)...");
        check_tool_version("lintian", &self.config.build_env.lintian_version)?;

        let changes_file = self.get_changes_file();
        let codename = normalize_codename(&self.config.build_env.codename)?;
        let mut args = vec![
            "--suppress-tags".to_string(),
            "bad-distribution-in-changes-file".to_string(),
            "-i".to_string(),
            "--I".to_string(),
            format!("{:?}", changes_file),
            "--tag-display-limit=0".to_string(),
            "--fail-on=warning".to_string(),
            "--fail-on=error".to_string(),
            "--suppress-tags".to_string(),
            "debug-file-with-no-debug-symbols".to_string(),
        ];

        if codename == "jammy" || codename == "noble" {
            args.extend([
                "--suppress-tags".to_string(),
                "malformed-deb-archive".to_string(),
            ]);
        }

        info!("Running: lintian {}", args.join(" "));
        execute_command("lintian", args, None)
    }

    fn run_piuparts(&self) -> Result<()> {
        info!("Running piuparts (requires sudo)...");
        check_tool_version("piuparts", &self.config.build_env.piuparts_version)?;

        let codename = normalize_codename(&self.config.build_env.codename)?;
        let repo_url = get_repo_url(&self.config.build_env.codename)?;
        let keyring = get_keyring(&self.config.build_env.codename)?;
        let keyring_opt = format!("--keyring={}", keyring);
        let mut args = vec![
            "-d".to_string(),
            codename.to_string(),
            "-m".to_string(),
            repo_url.to_string(),
            "--bindmount=/dev".to_string(),
            keyring_opt,
            "--verbose".to_string(),
        ];

        if let Some(LanguageEnv::Dotnet(_)) = self.language_env() {
            if self.config.build_env.codename == "bookworm"
                || self.config.build_env.codename == "jammy jellyfish"
            {
                let repo = format!(
                    "--extra-repo=deb https://packages.microsoft.com/debian/12/prod {} main",
                    self.config.build_env.codename
                );
                args.push(repo);
                args.push("--do-not-verify-signatures".to_string());
            }
        }

        let deb_name = self.get_deb_name();
        info!(
            "Running: sudo -S piuparts {} {}",
            args.join(" "),
            deb_name.display()
        );
        execute_command_with_sudo("piuparts", args, &deb_name, Some(self.get_deb_dir()))
    }

    fn run_autopkgtests(&self) -> Result<()> {
        info!("Running autopkgtests...");
        check_tool_version("autopkgtest", &self.config.build_env.autopkgtest_version)?;

        let codename = normalize_codename(&self.config.build_env.codename)?;
        let image_path = self.prepare_autopkgtest_image(&codename)?;
        let changes_file = self.get_changes_file();

        let mut args = vec![
            changes_file
                .to_str()
                .ok_or(eyre!("Invalid changes file path"))?,
            "--no-built-binaries",
            "--apt-upgrade",
        ];
        let debian_test_deps: Vec<String> = self
            .get_test_deps_not_in_debian()
            .iter()
            .map(|dep| format!("--setup-commands={}", dep))
            .collect();
        args.extend(
            debian_test_deps
                .iter()
                .map(|dep| dep.as_str())
                .collect::<Vec<&str>>(),
        );
        args.extend(&[
            "--",
            "qemu",
            image_path.to_str().ok_or(eyre!("Invalid image path"))?,
        ]);

        info!("Running: autopkgtest {}", args.join(" "));
        execute_command("autopkgtest", &args, Some(&self.get_deb_dir()))
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
        let image_name = format!(
            "autopkgtest-{}-{}.img",
            codename, self.config.build_env.arch
        );
        let cache_dir = shellexpand::tilde(&self.cache_dir).to_string();
        let image_path = Path::new(&cache_dir).join(&image_name);

        if !image_path.exists() {
            info!("Creating autopkgtest image: {}", image_name);
            ensure_parent_dir(image_path.clone())?;
            let repo_url = get_repo_url(&self.config.build_env.codename)?;

            let args = match codename {
                "bookworm" => vec![
                    codename.to_string(),
                    format!("{:?}", image_path),
                    format!("--mirror={}", repo_url),
                    format!("--arch={}", self.config.build_env.arch),
                ],
                "noble numbat" | "jammy jellyfish" => vec![
                    format!("--release={}", codename),
                    format!("--mirror={}", repo_url),
                    format!("--arch={}", self.config.build_env.arch),
                    "-v".to_string(),
                ],
                _ => return Err(eyre!("Unsupported codename: {}", codename)),
            };

            execute_command_with_sudo(
                if codename == "bookworm" {
                    "autopkgtest-build-qemu"
                } else {
                    "autopkgtest-buildvm-ubuntu-cloud"
                },
                args,
                &image_path,
                Some(image_path.parent().unwrap()),
            )?;
        }
        Ok(image_path)
    }
}

// Utility Functions
fn execute_command<I, S>(cmd: &str, args: I, dir: Option<&Path>) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(cmd);
    if let Some(dir) = dir {
        command.current_dir(dir);
    }
    command
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());

    run_command(&mut command, cmd)
}

fn execute_command_with_sudo(
    cmd: &str,
    args: Vec<String>,
    target: &Path,
    dir: Option<&Path>,
) -> Result<()> {
    let mut command = Command::new("sudo");
    command
        .arg("-S")
        .arg(cmd)
        .args(args)
        .arg(target)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());
    if let Some(dir) = dir {
        command.current_dir(dir);
    }

    run_command(&mut command, &format!("sudo -S {}", cmd))
}

fn run_command(command: &mut Command, cmd_name: &str) -> Result<()> {
    let mut child = command.spawn()?;
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            info!("{}", line?);
        }
    }
    let status = child.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(eyre!(
            "Command '{}' failed with status: {}",
            cmd_name,
            status
        ))
    }
}

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

    warn_compare_versions(expected_version.to_string(), &actual_version, tool)?;
    Ok(())
}

fn warn_compare_versions(expected: String, actual: &str, tool: &str) -> Result<()> {
    let expected_ver = Version::parse(&expected)?;
    let actual_ver = Version::parse(actual)?;
    match expected_ver.cmp(&actual_ver) {
        std::cmp::Ordering::Less => warn!("Using newer {} version than expected", tool),
        std::cmp::Ordering::Greater => warn!("Using older {} version", tool),
        std::cmp::Ordering::Equal => info!("{} versions match", tool),
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
    use crate::v1::pkg_config::PkgConfig;
    use env_logger::Env;
    use std::sync::Once;
    use tempfile::tempdir;

    static INIT: Once = Once::new();

    fn setup() {
        INIT.call_once(|| {
            env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
        });
    }

    #[test]
    fn test_clean_sbuild_env_when_file_does_not_exist() {
        setup();
        let mut pkg_config = PkgConfig::default();
        let build_files_dir = tempdir().unwrap().path().to_str().unwrap().to_string();
        pkg_config.build_env.codename = "bookworm".to_string();
        pkg_config.build_env.arch = "amd64".to_string();
        let sbuild_cache_dir = tempdir().unwrap().path().to_str().unwrap().to_string();
        pkg_config.build_env.sbuild_cache_dir = Some(sbuild_cache_dir);
        let build_env = Sbuild::new(pkg_config, build_files_dir);
        let result = build_env.clean();
        assert!(result.is_ok());
        let cache_file = build_env.get_cache_file();
        assert!(!Path::new(&cache_file).exists());
    }

    #[test]
    fn test_clean_sbuild_env() {
        setup();
        let mut pkg_config = PkgConfig::default();
        let build_files_dir = tempdir().unwrap().path().to_str().unwrap().to_string();
        pkg_config.build_env.codename = "bookworm".to_string();
        pkg_config.build_env.arch = "amd64".to_string();
        let sbuild_cache = tempdir().unwrap();
        create_dir_all(sbuild_cache.path()).unwrap();
        let sbuild_cache_dir = sbuild_cache.path().to_str().unwrap().to_string();
        pkg_config.build_env.sbuild_cache_dir = Some(sbuild_cache_dir.clone());
        let build_env = Sbuild::new(pkg_config, build_files_dir);
        let cache_file = build_env.get_cache_file();
        File::create(&cache_file).unwrap();
        assert!(Path::new(&cache_file).exists());
        let result = build_env.clean();
        assert!(result.is_ok());
        assert!(!Path::new(&cache_file).exists());
    }

    #[test]
    fn test_create_sbuild_env() {
        setup();
        let mut pkg_config = PkgConfig::default();
        pkg_config.build_env.codename = "bookworm".to_string();
        pkg_config.build_env.arch = "amd64".to_string();
        let sbuild_cache_dir = tempdir().unwrap().path().to_str().unwrap().to_string();
        pkg_config.build_env.sbuild_cache_dir = Some(sbuild_cache_dir);
        let build_files_dir = tempdir().unwrap().path().to_str().unwrap().to_string();
        let build_env = Sbuild::new(pkg_config, build_files_dir.clone());
        build_env.clean().unwrap();
        let cache_file = build_env.get_cache_file();
        assert!(!Path::new(&cache_file).exists());
        let result = build_env.create();
        assert!(result.is_ok());
        assert!(Path::new(&cache_file).exists());
    }
}
