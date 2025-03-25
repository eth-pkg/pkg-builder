use std::path::{Path, PathBuf};

use types::distribution::Distribution;

use crate::{
    installers::language_installer::LanguageInstaller,
    pkg_config::{LanguageEnv, PackageType},
    sbuild::Sbuild,
};

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

    pub fn get_deb_dir(&self) -> PathBuf {
        PathBuf::from(&self.build_files_dir)
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
}
