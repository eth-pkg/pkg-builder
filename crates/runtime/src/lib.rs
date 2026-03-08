mod c;
mod dotnet;
mod go;
mod java;
mod nim;
mod node;
mod python;
mod rust;
pub mod script;

use config::build_env::{Architecture, Distribution};
use config::language::LanguageEnv;

/// Trait for language runtime installers.
pub trait Runtime: Send + Sync {
    /// Shell commands to install this runtime inside the chroot.
    fn install_commands(&self) -> Vec<String>;

    /// Additional sbuild chroot-setup-commands.
    fn build_deps(&self, _arch: &Architecture, _dist: &Distribution) -> Vec<String> {
        self.install_commands()
    }

    /// Additional test dependencies for autopkgtest.
    fn test_deps(&self, _dist: &Distribution) -> Vec<String> {
        vec![]
    }
}

/// Create a runtime installer for the given language environment.
pub fn runtime_for(lang: &LanguageEnv) -> Box<dyn Runtime> {
    match lang {
        LanguageEnv::Rust(config) => Box::new(rust::RustRuntime(config.clone())),
        LanguageEnv::Go(config) => Box::new(go::GoRuntime(config.clone())),
        LanguageEnv::JavaScript(config) | LanguageEnv::TypeScript(config) => {
            Box::new(node::NodeRuntime(config.clone()))
        }
        LanguageEnv::Java(config) => Box::new(java::JavaRuntime(config.clone())),
        LanguageEnv::Dotnet(config) => Box::new(dotnet::DotnetRuntime(config.clone())),
        LanguageEnv::Nim(config) => Box::new(nim::NimRuntime(config.clone())),
        LanguageEnv::C => Box::new(c::CRuntime),
        LanguageEnv::Python => Box::new(python::PythonRuntime),
    }
}
