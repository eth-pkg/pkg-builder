use std::{borrow::Cow, collections::HashMap};

use types::distribution::Distribution;

use crate::configs::pkg_config::GoConfig;

use super::language_installer::LanguageInstaller;

pub struct GoInstaller(pub(crate) GoConfig);

impl LanguageInstaller for GoInstaller {
    fn recipe(&self) -> Cow<'static, str> {
        let recipe = include_str!("../recipes/go_installer.sh");
        Cow::Borrowed(recipe)
    }

    fn substitutions(&self) -> HashMap<&str, &str> {
        let mut subs = HashMap::new();
        subs.insert("${go_binary_url}", self.0.go_binary_url.as_str());
        subs.insert("${go_binary_checksum}", self.0.go_binary_checksum.as_str());
        subs
    }
    fn get_test_deps(&self, _codename: &Distribution) -> Vec<String> {
        vec![]
    }
}
