use std::{
    convert::TryFrom,
    path::{Path, PathBuf},
};

use types::{defaults::WORKDIR_ROOT, distribution::Distribution, version::Version};

use crate::{
    configs::{
        autopkgtest_version::AutopkgtestVersion,
        pkg_config::{LanguageEnv, PackageType, PkgConfig},
        sbuild_version::SbuildVersion,
    },
    installers::language_installer::LanguageInstaller,
    misc::{build_pipeline::BuildContext, utils::expand_path},
    tools::{
        autopkgtest_tool::AutopkgtestToolArgs, lintian_tool::LintianToolArgs,
        piuparts_tool::PiupartsToolArgs, sbuild_tool::SbuildToolArgs,
    },
};

#[derive(Debug, Clone)]
pub struct SbuildArgs {
    cache_dir: PathBuf,
    build_files_dir: PathBuf,
    codename: Distribution,
    run_lintian: bool,
    run_autopkgtests: bool,
    run_piuparts: bool,
    lintian_version: Version,
    piuparts_version: Version,
    autopkgtest_version: AutopkgtestVersion,
    sbuild_version: SbuildVersion,
    package_type: PackageType,
    arch: String,
    context: BuildContext,
    deb_dir: PathBuf,
    deb_name: PathBuf,
    changes_file: PathBuf,
    cache_file: PathBuf,
    language_env: Option<LanguageEnv>,
    test_deps_not_in_debian: Vec<String>,
    chroot_setup_commands: Vec<String>,
}

impl TryFrom<PkgConfig> for SbuildArgs {
    type Error = std::io::Error;

    fn try_from(config: PkgConfig) -> Result<Self, Self::Error> {
        let config_root = config.config_root;
        // Cache directory
        let cache_dir = config
            .build_env
            .sbuild_cache_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from("~/.cache/sbuild"));

        let cache_dir = expand_path(&cache_dir, None);

        // Build context
        let package_fields = &config.package_fields;
        let config_root_path = PathBuf::from(config_root.clone());
        let source_to_patch_from_path = config_root_path.join("src").to_str().unwrap().to_string();

        let mut workdir = config.build_env.workdir.clone();
        let mut default_work_dir = PathBuf::from(WORKDIR_ROOT);
        default_work_dir.push(config.build_env.codename.as_ref());
        if workdir.as_os_str().is_empty() {
            workdir = default_work_dir;
        }
        let workdir = expand_path(&workdir, None);

        let build_artifacts_dir = {
            let build_artifacts_dir = format!(
                "{}/{}-{}-{}",
                workdir.display().to_string(),
                &package_fields.package_name,
                package_fields.version_number.as_str(),
                &package_fields.revision_number
            );
            PathBuf::from(build_artifacts_dir)
        };

        let debian_orig_tarball_path = {
            let tarball_path = format!(
                "{}/{}_{}.orig.tar.gz",
                &build_artifacts_dir.display().to_string(),
                &package_fields.package_name,
                &package_fields.version_number
            );
            PathBuf::from(tarball_path)
        };

        let build_files_dir = {
            let build_files_dir = format!(
                "{}/{}-{}",
                build_artifacts_dir.display().to_string(),
                &package_fields.package_name,
                &package_fields.version_number
            );
            PathBuf::from(build_files_dir)
        };

        let mut context = BuildContext {
            build_artifacts_dir,
            build_files_dir: build_files_dir.clone(),
            debcrafter_version: config.build_env.debcrafter_version.as_str().to_string(),
            homepage: package_fields.homepage.clone(),
            spec_file: package_fields.spec_file.clone(),
            tarball_hash: String::new(),
            tarball_url: String::new(),
            src_dir: source_to_patch_from_path.into(),
            tarball_path: debian_orig_tarball_path,
            package_name: package_fields.package_name.clone(),
            git_tag: String::new(),
            git_url: String::new(),
            submodules: vec![],
        };

        match &config.package_type {
            PackageType::Default(default_config) => {
                context.tarball_url = {
                    let tarball_url = default_config.tarball_url.as_str();
                    if tarball_url.starts_with("http") {
                        tarball_url.to_string()
                    } else {
                        expand_path(
                            &PathBuf::from(tarball_url),
                            Some(&PathBuf::from(&config_root)),
                        )
                        .display()
                        .to_string()
                    }
                };

                if let Some(hash) = &default_config.tarball_hash {
                    context.tarball_hash = hash.clone();
                }
            }
            PackageType::Git(git_config) => {
                context.git_tag = git_config.git_tag.clone();
                context.git_url = git_config.git_url.as_str().to_string();
                context.submodules = git_config.submodules.clone();
            }
            PackageType::Virtual => {
                // Virtual packages already have the correct default values
            }
        }

        // Extract other fields from config
        let run_lintian = config.build_env.run_lintian.unwrap_or(false);
        let run_autopkgtests = config.build_env.run_piuparts.unwrap_or(false);
        let run_piuparts = config.build_env.run_piuparts.unwrap_or(false);
        let lintian_version = config.build_env.lintian_version.clone();
        let piuparts_version = config.build_env.piuparts_version.clone();
        let autopkgtest_version = config.build_env.autopkgtest_version.clone();
        let sbuild_version = config.build_env.sbuild_version.clone();
        let package_type = config.package_type.clone();
        let arch = config.build_env.arch.clone();
        let package_name = package_fields.package_name.clone();
        let package_version_number = package_fields.version_number.clone();
        let revision_number = package_fields.revision_number.clone();

        // Generate derived paths
        let deb_dir = PathBuf::from(&build_files_dir)
            .parent()
            .unwrap()
            .to_path_buf();

        let deb_name = {
            let filename = format!(
                "{}_{}-{}_{}.{}",
                &package_name, &package_version_number, &revision_number, &arch, "deb"
            );
            deb_dir.join(filename)
        };

        let changes_file = {
            let filename = format!(
                "{}_{}-{}_{}.{}",
                &package_name, &package_version_number, &revision_number, &arch, "changes"
            );
            deb_dir.join(filename)
        };

        let cache_file = {
            let dir = shellexpand::tilde(&cache_dir.display().to_string()).to_string();
            let codename = &config.build_env.codename.as_short();
            let cache_file_name = format!("{}-{}.tar.gz", codename, &arch);
            Path::new(&dir).join(cache_file_name)
        };

        // Get language environment
        let language_env = match &package_type {
            PackageType::Default(config) => Some(config.language_env.clone()),
            PackageType::Git(config) => Some(config.language_env.clone()),
            PackageType::Virtual => None,
        };

        // Build deps
        let build_deps_not_in_debian = match &language_env {
            Some(env) => {
                let installer: Box<dyn LanguageInstaller> = env.clone().into();
                installer.get_build_deps(&arch, &config.build_env.codename)
            }
            None => vec![],
        };

        // Test deps
        let test_deps_not_in_debian = match &language_env {
            Some(env) => {
                let installer: Box<dyn LanguageInstaller> = env.clone().into();
                installer.get_test_deps(&config.build_env.codename)
            }
            None => vec![],
        };

        // Chroot setup commands
        let mut chroot_setup_commands = build_deps_not_in_debian.clone();
        if config.build_env.codename == Distribution::noble() {
            chroot_setup_commands.extend(vec![
                "apt install -y software-properties-common".to_string(),
                "add-apt-repository universe".to_string(),
                "add-apt-repository restricted".to_string(),
                "add-apt-repository multiverse".to_string(),
                "apt update".to_string(),
            ]);
        }
        let chroot_setup_commands = chroot_setup_commands
            .into_iter()
            .map(|dep| format!("--chroot-setup-commands={}", dep))
            .collect();

        Ok(SbuildArgs {
            cache_dir,
            build_files_dir,
            codename: config.build_env.codename,
            run_lintian,
            run_autopkgtests,
            run_piuparts,
            lintian_version,
            piuparts_version,
            autopkgtest_version,
            sbuild_version,
            package_type,
            arch,
            context,
            deb_dir,
            deb_name,
            changes_file,
            cache_file,
            language_env,
            test_deps_not_in_debian,
            chroot_setup_commands,
        })
    }
}

impl SbuildArgs {
    pub fn get_sbuild_tool_args(&self) -> SbuildToolArgs {
        let version = self.sbuild_version.clone();
        let codename = self.codename.clone();
        let cache_file = self.cache_file.clone();
        let build_chroot_setup_commands = self.chroot_setup_commands.clone();
        let build_files_dir = self.build_files_dir.clone();
        let package_type = self.package_type.clone();
        let context = self.context.clone();
        SbuildToolArgs {
            version,
            codename,
            cache_file,
            build_chroot_setup_commands,
            build_files_dir,
            package_type,
            context,
            run_lintian: self.run_lintian,
        }
    }

    pub fn get_autopkg_tool_args(&self) -> AutopkgtestToolArgs {
        let version = self.autopkgtest_version.clone();
        let changes_file = self.changes_file.clone();
        let codename = self.codename.clone();
        let deb_dir = self.deb_dir.clone();
        let test_deps = self.test_deps_not_in_debian.clone();
        let cache_dir = self.cache_dir.clone();
        let arch = self.arch.clone();

        AutopkgtestToolArgs {
            version,
            changes_file,
            codename,
            deb_dir,
            test_deps,
            image_path: None,
            cache_dir,
            arch,
        }
    }
    pub fn get_lintian_tool_args(&self) -> LintianToolArgs {
        let version = self.lintian_version.clone();
        let changes_file = self.changes_file.clone();
        let codename = self.codename.clone();

        LintianToolArgs {
            version,
            changes_file,
            codename,
        }
    }
    pub fn get_piuparts_tool_args(&self) -> PiupartsToolArgs {
        let version = self.piuparts_version.clone();
        let codename = self.codename.clone();
        let deb_dir = self.deb_dir.clone();
        let deb_name = self.deb_name.clone();
        let language_env = self.language_env.clone();
        PiupartsToolArgs {
            version,
            codename,
            deb_dir,
            language_env,
            deb_name,
        }
    }

    pub fn build_files_dir(&self) -> &PathBuf {
        &self.build_files_dir
    }
    pub fn codename(&self) -> &Distribution {
        &self.codename
    }

    pub fn run_autopkgtests(&self) -> bool {
        self.run_autopkgtests
    }
    pub fn run_piuparts(&self) -> bool {
        self.run_piuparts
    }

    pub fn get_cache_file(&self) -> PathBuf {
        self.cache_file.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configs::autopkgtest_version::AutopkgtestVersion;
    use crate::configs::pkg_config::{
        BuildEnv, DefaultPackageTypeConfig, DotnetConfig, LanguageEnv, PackageFields, PackageType,
    };
    use crate::configs::sbuild_version::SbuildVersion;
    use std::convert::TryFrom;
    use std::path::PathBuf;
    use types::distribution::Distribution;
    use types::version::Version;

    fn create_test_pkg_config() -> PkgConfig {
        let config = PkgConfig {
            package_fields: PackageFields {
                spec_file: "hello-world-dotnet.sss".into(),
                package_name: "hello-world".into(),
                version_number: Version::try_from("1.0.0").unwrap(),
                revision_number: "1".into(),
                homepage: "http://example.com".into(),
            },
            package_type: PackageType::Default(DefaultPackageTypeConfig {
                tarball_url: "test.tar.gz".into(),
                tarball_hash: Some("".into()),
                language_env: LanguageEnv::Dotnet(DotnetConfig {
                    dotnet_packages: vec![],
                    use_backup_version: false,
                    deps: None,
                }),
            }),
            build_env: BuildEnv {
                codename: Distribution::bookworm(),
                arch: "amd64".into(),
                pkg_builder_version: Version::try_from("1.0.0").unwrap(),
                debcrafter_version: "1.0.0".into(),
                sbuild_cache_dir: None,
                docker: None,
                run_lintian: None,
                run_piuparts: None,
                run_autopkgtest: None,
                lintian_version: Version::try_from("1.0.0").unwrap(),
                piuparts_version: Version::try_from("1.0.0").unwrap(),
                autopkgtest_version: AutopkgtestVersion::try_from("2.5").unwrap(),
                sbuild_version: SbuildVersion::try_from(
                    "sbuild (Debian sbuild) 0.85.6 (26 February 2024)",
                )
                .unwrap(),
                workdir: PathBuf::from("/tmp/workdir/packages"),
            },
            config_root: "/test/config/root".into(),
        };

        config
    }

    #[test]
    fn test_cache_dir_set_to_default() {
        let config = create_test_pkg_config();
        let args = SbuildArgs::try_from(config.clone()).unwrap();

        // Test all fields
        assert_eq!(
            args.cache_dir.display().to_string(),
            shellexpand::tilde("~/.cache/sbuild").to_string()
        );
    }

    #[test]
    fn test_cache_dir_overriden() {
        let mut config = create_test_pkg_config();
        config.build_env.sbuild_cache_dir = Some(PathBuf::from("/test/cache/dir"));
        let args = SbuildArgs::try_from(config.clone()).unwrap();

        // Test all fields
        assert_eq!(
            args.cache_dir.display().to_string(),
            shellexpand::tilde("/test/cache/dir").to_string()
        );
    }

    #[test]
    fn test_deb_name_construction() {
        let config = create_test_pkg_config();
        let args = SbuildArgs::try_from(config.clone()).unwrap();

        let expected_deb_name = PathBuf::from(
            "/tmp/workdir/packages/hello-world-1.0.0-1/hello-world_1.0.0-1_amd64.deb",
        );
        assert_eq!(
            args.deb_name,
            expected_deb_name,
            "deb_name should be correctly constructed from package name, version, revision, and arch"
        );
    }

    #[test]
    fn test_changes_file_construction() {
        let config = create_test_pkg_config();
        let args = SbuildArgs::try_from(config.clone()).unwrap();

        let expected_changes_file = PathBuf::from(
            "/tmp/workdir/packages/hello-world-1.0.0-1/hello-world_1.0.0-1_amd64.changes",
        );
        assert_eq!(
            args.changes_file,
            expected_changes_file,
            "changes_file should be correctly constructed from package name, version, revision, and arch"
        );
    }

    #[test]
    fn test_cache_file_construction() {
        let config = create_test_pkg_config();
        let args = SbuildArgs::try_from(config.clone()).unwrap();

        let expected_cache_file =
            PathBuf::from(shellexpand::tilde("~/.cache/sbuild/bookworm-amd64.tar.gz").to_string());
        assert_eq!(
            args.get_cache_file(),
            expected_cache_file,
            "cache_file should be constructed from cache_dir, codename, and arch"
        );
    }

    #[test]
    fn test_deb_dir_construction() {
        let config = create_test_pkg_config();
        let args = SbuildArgs::try_from(config.clone()).unwrap();

        let expected_deb_dir = PathBuf::from("/tmp/workdir/packages/hello-world-1.0.0-1");
        assert_eq!(
            args.deb_dir, expected_deb_dir,
            "deb_dir should be the parent directory of build_files_dir"
        );
    }

    #[test]
    fn test_build_files_dir_construction() {
        let config = create_test_pkg_config();
        let args = SbuildArgs::try_from(config.clone()).unwrap();

        let expected_build_files_dir =
            PathBuf::from("/tmp/workdir/packages/hello-world-1.0.0-1/hello-world-1.0.0");
        assert_eq!(
            args.build_files_dir(),
            &expected_build_files_dir,
            "build_files_dir should be constructed from workdir, package name, and version"
        );
    }

    #[test]
    fn test_context_tarball_url_local_path() {
        let config = create_test_pkg_config();
        let args = SbuildArgs::try_from(config.clone()).unwrap();

        let expected_tarball_url = "/test/config/root/test.tar.gz".to_string();
        assert_eq!(
            args.context.tarball_url, expected_tarball_url,
            "tarball_url should be expanded relative to config_root for local paths"
        );
    }

    #[test]
    fn test_context_tarball_url_http() {
        let mut config = create_test_pkg_config();
        if let PackageType::Default(ref mut default_config) = config.package_type {
            default_config.tarball_url = "https://example.com/test.tar.gz".into();
        }
        let args = SbuildArgs::try_from(config.clone()).unwrap();

        let expected_tarball_url = "https://example.com/test.tar.gz".to_string();
        assert_eq!(
            args.context.tarball_url, expected_tarball_url,
            "tarball_url should remain unchanged for HTTP URLs"
        );
    }
}
