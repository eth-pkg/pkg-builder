use std::{borrow::Cow, collections::HashMap};

use types::distribution::Distribution;

use super::language_installer::LanguageInstaller;

pub struct EmptyInstaller;

impl LanguageInstaller for EmptyInstaller {
    fn get_test_deps(&self, _codename: &Distribution) -> Vec<String> {
        vec![]
    }

    fn recipe(&self) -> Cow<'static, str> {
        Cow::Borrowed("")
    }

    fn substitutions(&self) -> HashMap<&str, &str> {
        HashMap::new()
    }
}
