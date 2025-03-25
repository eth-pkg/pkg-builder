use std::{borrow::Cow, collections::HashMap};

use types::distribution::Distribution;

use crate::configs::pkg_config::NimConfig;

use super::language_installer::LanguageInstaller;

pub struct NimInstaller(pub(crate) NimConfig);

impl LanguageInstaller for NimInstaller {
    fn recipe(&self) -> Cow<'static, str> {
        let recipe = include_str!("../recipes/nim_installer.sh");
        Cow::Borrowed(recipe)
    }

    fn substitutions(&self) -> HashMap<&str, &str> {
        let mut subs = HashMap::new();
        subs.insert("${nim_binary_url}", self.0.nim_binary_url.as_str());
        subs.insert("${nim_version}", &self.0.nim_version.as_str());
        subs.insert(
            "${nim_version_checksum}",
            &self.0.nim_version_checksum.as_str(),
        );
        subs
    }
    fn get_test_deps(&self, _codename: &Distribution) -> Vec<String> {
        vec![]
    }
}
