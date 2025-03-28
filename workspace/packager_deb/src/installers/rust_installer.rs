use std::{borrow::Cow, collections::HashMap};

use types::distribution::Distribution;

use crate::configs::pkg_config::RustConfig;

use super::language_installer::LanguageInstaller;

pub struct RustInstaller(pub(crate) RustConfig);

impl LanguageInstaller for RustInstaller {
    fn recipe(&self) -> Cow<'static, str> {
        let recipe = include_str!("../recipes/rust_installer.sh");
        Cow::Borrowed(recipe)
    }

    fn substitutions(&self) -> HashMap<&str, &str> {
        let mut subs = HashMap::new();
        subs.insert("${rust_binary_url}", self.0.rust_binary_url.as_str());
        subs.insert(
            "${rust_binary_gpg_asc}",
            self.0.rust_binary_gpg_asc.as_str(),
        );
        subs
    }

    fn get_test_deps(&self, _codename: &Distribution) -> Vec<String> {
        vec![] // Rust compiles to binary, no test deps needed
    }
}
