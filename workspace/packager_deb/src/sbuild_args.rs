use std::{
    env, fs,
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
    misc::build_pipeline::BuildContext,
};

#[derive(Debug, Clone)]
pub struct SbuildArgs {
    cache_dir: PathBuf,
    config: PkgConfig,
    _config_root: PathBuf,
    context: BuildContext,
}

impl SbuildArgs {
    pub fn new(config: PkgConfig, config_root: PathBuf) -> Self {
        let cache_dir = config
            .build_env
            .sbuild_cache_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from("~/.cache/sbuild"));
        let context = SbuildArgs::build_context(config.clone(), config_root.clone());
        SbuildArgs {
            config,
            cache_dir,
            _config_root: config_root,
            context,
        }
    }

    pub fn cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }
    pub fn build_files_dir(&self) -> &PathBuf {
        &self.context.build_files_dir
    }
    pub fn codename(&self) -> &Distribution {
        &self.config.build_env.codename
    }

    pub fn run_lintian(&self) -> bool {
        self.config.build_env.run_lintian.unwrap_or(false)
    }

    pub fn run_autopkgtests(&self) -> bool {
        self.config.build_env.run_piuparts.unwrap_or(false)
    }

    pub fn run_piuparts(&self) -> bool {
        self.config.build_env.run_piuparts.unwrap_or(false)
    }

    pub fn lintian_version(&self) -> Version {
        self.config.build_env.lintian_version.clone()
    }

    pub fn piuparts_version(&self) -> Version {
        self.config.build_env.piuparts_version.clone()
    }

    pub fn autopkgtest_version(&self) -> AutopkgtestVersion {
        self.config.build_env.autopkgtest_version.clone()
    }

    pub fn sbuild_version(&self) -> SbuildVersion {
        self.config.build_env.sbuild_version.clone()
    }

    pub fn package_type(&self) -> PackageType {
        self.config.package_type.clone()
    }

    pub fn arch(&self) -> String {
        self.config.build_env.arch.clone()
    }

    pub fn context(&self) -> BuildContext {
        self.context.clone()
    }

    pub fn get_cache_file(&self) -> PathBuf {
        let dir = shellexpand::tilde(&self.cache_dir.display().to_string()).to_string();
        let codename = &self.config.build_env.codename.as_short();
        let cache_file_name = format!("{}-{}.tar.gz", codename, self.config.build_env.arch);
        Path::new(&dir).join(cache_file_name)
    }

    pub fn get_deb_dir(&self) -> PathBuf {
        PathBuf::from(&self.build_files_dir())
            .parent()
            .unwrap()
            .to_path_buf()
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
    pub fn build_chroot_setup_commands(&self) -> Vec<String> {
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

    pub fn language_env(&self) -> Option<LanguageEnv> {
        match &self.config.package_type {
            PackageType::Default(config) => Some(config.language_env.clone()),
            PackageType::Git(config) => Some(config.language_env.clone()),
            PackageType::Virtual => None,
        }
    }

    pub fn build_context(config: PkgConfig, config_root: PathBuf) -> BuildContext {
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

        let build_artifacts_dir = get_build_artifacts_dir(
            &package_fields.package_name,
            &workdir.display().to_string(),
            &package_fields.version_number.as_str(),
            &package_fields.revision_number,
        );

        let debian_orig_tarball_path = get_tarball_path(
            &package_fields.package_name,
            &package_fields.version_number.as_str(),
            &build_artifacts_dir,
        );

        let build_files_dir = get_build_files_dir(
            &package_fields.package_name,
            &package_fields.version_number.as_str(),
            &build_artifacts_dir,
        );

        let mut context = BuildContext {
            build_artifacts_dir,
            build_files_dir,
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
                context.tarball_url =
                    get_tarball_url(&default_config.tarball_url.as_str(), &config_root);
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

        context
    }
}

pub fn get_build_artifacts_dir(
    package_name: &str,
    work_dir: &str,
    version_number: &str,
    revision_number: &str,
) -> PathBuf {
    let build_artifacts_dir = format!(
        "{}/{}-{}-{}",
        work_dir, &package_name, version_number, revision_number
    );
    PathBuf::from(build_artifacts_dir)
}

pub fn get_tarball_path(
    package_name: &str,
    version_number: &str,
    build_artifacts_dir: &PathBuf,
) -> PathBuf {
    let tarball_path = format!(
        "{}/{}_{}.orig.tar.gz",
        &build_artifacts_dir.display().to_string(),
        &package_name,
        &version_number
    );
    PathBuf::from(tarball_path)
}

pub fn get_build_files_dir(
    package_name: &str,
    version_number: &str,
    build_artifacts_dir: &PathBuf,
) -> PathBuf {
    let build_files_dir = format!(
        "{}/{}-{}",
        build_artifacts_dir.display().to_string(),
        &package_name,
        &version_number
    );
    PathBuf::from(build_files_dir)
}

pub fn get_tarball_url(tarball_url: &str, config_root: &PathBuf) -> String {
    if tarball_url.starts_with("http") {
        tarball_url.to_string()
    } else {
        expand_path(
            &PathBuf::from(tarball_url),
            Some(&PathBuf::from(config_root)),
        )
        .display()
        .to_string()
    }
}

pub fn expand_path(dir: &PathBuf, dir_to_expand: Option<&PathBuf>) -> PathBuf {
    if dir.to_string_lossy().starts_with('~') {
        let dir_str = dir.to_string_lossy();
        PathBuf::from(shellexpand::tilde(&dir_str).to_string())
    } else if dir.is_absolute() {
        dir.clone()
    } else {
        let parent_dir = match dir_to_expand {
            None => env::current_dir().unwrap(),
            Some(path) => path.clone(),
        };

        let path = parent_dir.join(dir);
        fs::canonicalize(path.clone()).unwrap_or(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_path_expands_tilde_correctly() {
        let tilde = PathBuf::from("~");
        let result = expand_path(&tilde, None);
        assert_ne!(result, tilde);
        assert!(!result.display().to_string().contains('~'));
    }

    #[test]
    fn expand_path_handles_absolute_paths() {
        let absolute_path = PathBuf::from("/absolute/path");
        let result = expand_path(&absolute_path, None);
        assert_eq!(result, absolute_path);
    }

    #[test]
    fn expand_path_expands_relative_paths_with_parent() {
        let file = PathBuf::from("somefile");
        let mut tmp = PathBuf::from("/tmp");
        let result = expand_path(&file, Some(&tmp));
        tmp.push(file);
        assert_eq!(result, tmp);
    }

    #[test]
    fn expand_path_expands_relative_paths_without_parent() {
        let file = PathBuf::from("somefile");

        let result = expand_path(&file, None);
        assert!(result.display().to_string().starts_with('/'));
    }
}
