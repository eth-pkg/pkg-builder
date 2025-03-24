use std::{borrow::Cow, collections::HashMap};

use crate::pkg_config::JavascriptConfig;

use super::language_installer::LanguageInstaller;

pub struct NodeInstaller(pub(crate) JavascriptConfig);
impl LanguageInstaller for NodeInstaller {
    fn recipe(&self) -> Cow<'static, str> {
        let recipe = include_str!("../recipes/node_installer.sh");
        if let Some(_) = &self.0.yarn_version {
            let yarn_installer = include_str!("../recipes/yarn_installer.sh");
            let installer = recipe.to_string() + yarn_installer;
            Cow::Owned(installer)
        } else {
            Cow::Borrowed(recipe)
        }
    }

    fn substitutions(&self) -> HashMap<&str, &str> {
        let mut subs = HashMap::new();
        subs.insert(
            "${node_binary_checksum}",
            self.0.node_binary_checksum.as_str(),
        );
        subs.insert("${node_binary_url}", &self.0.node_binary_url.as_str());
        subs.insert("${node_version}", &&self.0.node_version.as_str());
        if let Some(yarn_version) = &self.0.yarn_version {
            subs.insert("${yarn_version}", &yarn_version.as_str());
        }
        subs
    }
    fn get_test_deps(&self, _codename: &str) -> Vec<String> {
        vec![]
    }
}
