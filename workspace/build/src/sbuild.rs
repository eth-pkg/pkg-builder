use std::path::{Path, PathBuf};

use common::pkg_config::{LanguageEnv, PkgConfig};
use eyre::{eyre, Result};

use super::language_installer::{get_installer, LanguageInstaller};

pub struct Sbuild {
    pub(crate) config: PkgConfig,
    pub(crate) build_files_dir: String,
    pub(crate) cache_dir: String,
}

impl Sbuild {
    pub fn new(config: PkgConfig, build_files_dir: String) -> Self {
        let cache_dir = config
            .build_env
            .sbuild_cache_dir
            .clone()
            .unwrap_or_else(|| "~/.cache/sbuild".to_string());

        Self {
            cache_dir,
            config,
            build_files_dir,
        }
    }

    pub fn get_cache_file(&self) -> String {
        let dir = shellexpand::tilde(&self.cache_dir).to_string();
        let codename = normalize_codename(&self.config.build_env.codename)
            .unwrap_or_else(|_| "unknown");
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

    fn get_installer(&self, lang_env: &LanguageEnv) -> Box<dyn LanguageInstaller> {
        get_installer(lang_env)
    }

    pub fn get_build_deps_not_in_debian(&self) -> Vec<String> {
        let lang_env = self.config.package_type.get_language_env();
        match lang_env {
            Some(env) => {
                let installer = self.get_installer(env);
                installer.get_build_deps(&self.config.build_env.arch, &self.config.build_env.codename)
            }
            None => vec![],
        }
    }

    pub fn get_test_deps_not_in_debian(&self) -> Vec<String> {
        let lang_env = self.config.package_type.get_language_env();
        match lang_env {
            Some(env) => {
                let installer = self.get_installer(env);
                installer.get_test_deps(&self.config.build_env.codename)
            }
            None => vec![],
        }
    }
}

pub fn normalize_codename(codename: &str) -> Result<&str> {
    match codename {
        "bookworm" => Ok("bookworm"),
        "noble numbat" => Ok("noble"),
        "jammy jellyfish" => Ok("jammy"),
        _ => Err(eyre!("Not supported distribution")),
    }
}
