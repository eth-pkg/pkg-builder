use std::{borrow::Cow, collections::HashMap};

use types::pkg_config::JavaConfig;

use super::language_installer::LanguageInstaller;

pub struct JavaInstaller(pub(crate) JavaConfig);

impl LanguageInstaller for JavaInstaller {
    fn recipe(&self) -> Cow<'static, str> {
        let java_installer = include_str!("../recipes/java_installer.sh");
        if let Some(_) = &self.0.gradle {
            let java_gradle_installer = include_str!("../recipes/java_gradle_installer.sh");
            let installer = java_installer.to_string() + java_gradle_installer;
            Cow::Owned(installer)
        } else {
            Cow::Borrowed(java_installer)
        }
    }

    fn substitutions(&self) -> HashMap<&str, &str> {
        let mut subs = HashMap::new();
        subs.insert("${jdk_version}", self.0.jdk_version.as_str());
        subs.insert("${jdk_binary_url}", &self.0.jdk_binary_url.as_str());
        subs.insert(
            "${jdk_binary_checksum}",
            &self.0.jdk_binary_checksum.as_str(),
        );
        if let Some(gradle_config) = &self.0.gradle {
            let gradle_version = &gradle_config.gradle_version;
            let gradle_binary_url = &gradle_config.gradle_binary_url;
            let gradle_binary_checksum = &gradle_config.gradle_binary_checksum;
            subs.insert("${gradle_version}", gradle_version.as_str());
            subs.insert("${gradle_binary_url}", &gradle_binary_url.as_str());
            subs.insert(
                "${gradle_binary_checksum}",
                &gradle_binary_checksum.as_str(),
            );
        }
        subs
    }
    fn get_test_deps(&self, _codename: &str) -> Vec<String> {
        vec![]
    }
}
