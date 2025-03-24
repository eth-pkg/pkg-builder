use types::distribution::Distribution;

use super::{
    command_builder::CommandBuilder, dotnet_installer::DotnetInstaller,
    empty_installer::EmptyInstaller, go_installer::GoInstaller, java_installer::JavaInstaller,
    nim_installer::NimInstaller, node_installer::NodeInstaller, rust_installer::RustInstaller,
};
use crate::pkg_config::LanguageEnv;
use std::{borrow::Cow, collections::HashMap};

pub trait LanguageInstaller {
    fn recipe(&self) -> Cow<'static, str>;
    fn substitutions(&self) -> HashMap<&str, &str>;

    fn get_build_deps(&self, _arch: &str, _codename: &Distribution) -> Vec<String> {
        let mut builder = CommandBuilder::new();
        let recipe = self.recipe();
        let substitutions = self.substitutions();

        for line in recipe.lines() {
            let command = line.trim();
            let mut processed_command = String::from(command);

            for (placeholder, value) in &substitutions {
                processed_command = processed_command.replace(placeholder, value);
            }

            builder.add(&processed_command);
        }

        builder.build()
    }
    fn get_test_deps(&self, codename: &Distribution) -> Vec<String>;
}

impl From<&LanguageEnv> for Box<dyn LanguageInstaller> {
    fn from(lang_env: &LanguageEnv) -> Self {
        match lang_env {
            LanguageEnv::Rust(config) => Box::new(RustInstaller(config.clone())),
            LanguageEnv::Go(config) => Box::new(GoInstaller(config.clone())),
            LanguageEnv::JavaScript(config) | LanguageEnv::TypeScript(config) => {
                Box::new(NodeInstaller(config.clone()))
            }
            LanguageEnv::Java(config) => Box::new(JavaInstaller(config.clone())),
            LanguageEnv::Dotnet(config) => Box::new(DotnetInstaller(config.clone())),
            LanguageEnv::Nim(config) => Box::new(NimInstaller(config.clone())),
            _ => Box::new(EmptyInstaller),
        }
    }
}
