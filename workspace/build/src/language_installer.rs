use std::{borrow::Cow, collections::HashMap};


use common::pkg_config::{DotnetConfig, GoConfig, JavaConfig, JavascriptConfig, LanguageEnv, NimConfig, RustConfig};

use super::command_builder::CommandBuilder;

pub trait LanguageInstaller {
    fn recipe(&self) -> Cow<'static, str>;
    fn substitutions(&self) -> HashMap<&str, &str>;

    fn get_build_deps(&self, _arch: &str, _codename: &str) -> Vec<String> {
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
    fn get_test_deps(&self, codename: &str) -> Vec<String>;
}

pub fn get_installer(lang_env: &LanguageEnv) -> Box<dyn LanguageInstaller> {
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

pub struct EmptyInstaller;
pub struct GoInstaller(GoConfig);
pub struct NodeInstaller(JavascriptConfig);
pub struct JavaInstaller(JavaConfig);
pub struct DotnetInstaller(DotnetConfig);
pub struct NimInstaller(NimConfig);
pub struct RustInstaller(RustConfig);

impl LanguageInstaller for EmptyInstaller {
    fn get_test_deps(&self, _codename: &str) -> Vec<String> {
        vec![]
    }

    fn recipe(&self) -> Cow<'static, str> {
        Cow::Borrowed("")
    }

    fn substitutions(&self) -> HashMap<&str, &str> {
        HashMap::new()
    }
}

impl LanguageInstaller for RustInstaller {
    fn recipe(&self) -> Cow<'static, str> {
        let recipe = include_str!("recipes/rust_installer.sh");
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

    fn get_test_deps(&self, _codename: &str) -> Vec<String> {
        vec![] // Rust compiles to binary, no test deps needed
    }
}

impl LanguageInstaller for GoInstaller {
    fn recipe(&self) -> Cow<'static, str> {
        let recipe = include_str!("recipes/go_installer.sh");
        Cow::Borrowed(recipe)
    }

    fn substitutions(&self) -> HashMap<&str, &str> {
        let mut subs = HashMap::new();
        subs.insert("${go_binary_url}", self.0.go_binary_url.as_str());
        subs.insert("${go_binary_checksum}", self.0.go_binary_checksum.as_str());
        subs
    }
    fn get_test_deps(&self, _codename: &str) -> Vec<String> {
        vec![]
    }
}

impl LanguageInstaller for NodeInstaller {
    fn recipe(&self) -> Cow<'static, str> {
        let recipe = include_str!("recipes/node_installer.sh");
        if let Some(_) = &self.0.yarn_version {
            let yarn_installer = include_str!("recipes/yarn_installer.sh");
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

impl LanguageInstaller for JavaInstaller {
    fn recipe(&self) -> Cow<'static, str> {
        let java_installer = include_str!("recipes/java_installer.sh");
        if let Some(_) = &self.0.gradle {
            let java_gradle_installer = include_str!("recipes/java_gradle_installer.sh");
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

impl LanguageInstaller for DotnetInstaller {
    fn recipe(&self) -> Cow<'static, str> {
        let recipe = include_str!("recipes/dotnet_installer.sh");
        Cow::Borrowed(&recipe)
    }

    fn substitutions(&self) -> HashMap<&str, &str> {
        let subs = HashMap::new();
        subs
    }
    fn get_build_deps(&self, arch: &str, codename: &str) -> Vec<String> {
        let dotnet_packages = &self.0.dotnet_packages;
        let deps = self.0.deps.clone().unwrap_or_default();
        let mut builder = CommandBuilder::new();

        if self.0.use_backup_version {
            builder
                .add("apt install -y wget")
                .add("apt install -y libicu-dev");

            for package in deps {
                builder.add_with("apt install -y {}", &package);
            }

            for package in dotnet_packages {
                builder
                    .add_with("cd /tmp && wget -q {}", &package.url)
                    .add_with("cd /tmp && ls && dpkg -i {}.deb", &package.name)
                    .add_with("cd /tmp && ls && sha1sum {}.deb", &package.name)
                    .add_with_args(
                        "cd /tmp &&  echo {} {}.deb > hash_file.txt && cat hash_file.txt",
                        &[&package.hash, &package.name],
                    )
                    .add("cd /tmp && sha1sum -c hash_file.txt");
            }

            builder.add("dotnet --version").add("apt remove -y wget");
        } else if codename == "bookworm" || codename == "jammy jellyfish" {
            builder
            .add("apt install -y wget")
            .add("cd /tmp && wget -q https://packages.microsoft.com/config/debian/12/packages-microsoft-prod.deb -O packages-microsoft-prod.deb")
            .add("cd /tmp && dpkg -i packages-microsoft-prod.deb")
            .add("apt update -y");

            for package in dotnet_packages {
                let pkg = transform_name(&package.name, arch);
                builder
                    .add_with("cd /tmp && wget -q {}", &package.url)
                    .add_with("cd /tmp && apt install -y --allow-downgrades {}", &pkg)
                    .add_with("cd /tmp && apt download -y {}", &pkg)
                    .add_with("cd /tmp && ls && sha1sum {}.deb", &package.name)
                    .add_with_args(
                        "cd /tmp &&  echo {} {}.deb >> hash_file.txt && cat hash_file.txt",
                        &[&package.hash, &package.name],
                    )
                    .add("cd /tmp && sha1sum -c hash_file.txt");
            }

            builder.add("dotnet --version").add("apt remove -y wget");
        } else if codename == "noble numbat" {
            builder
                .add("apt-get install software-properties-common -y")
                .add("add-apt-repository ppa:dotnet/backports")
                .add("apt-get update -y")
                .add("apt install -y wget");

            for package in dotnet_packages {
                let pkg = transform_name(&package.name, arch);
                builder
                    .add_with("cd /tmp && wget -q {}", &package.url)
                    .add_with("cd /tmp && apt install -y {}", &pkg)
                    .add_with("cd /tmp && apt download -y {}", &pkg)
                    .add_with("cd /tmp && ls && sha1sum {}.deb", &package.name)
                    .add_with_args(
                        "cd /tmp &&  echo {} {}.deb >> hash_file.txt && cat hash_file.txt",
                        &[&package.hash, &package.name],
                    )
                    .add("cd /tmp && sha1sum -c hash_file.txt");
            }

            builder.add("dotnet --version").add("apt remove -y wget");
        }

        builder.build()
    }
    fn get_test_deps(&self, codename: &str) -> Vec<String> {
        let mut builder = CommandBuilder::new();

        if codename == "bookworm" || codename == "jammy jellyfish" {
            builder
                .add("apt install -y wget")
                .add("cd /tmp && wget https://packages.microsoft.com/config/debian/12/packages-microsoft-prod.deb -O packages-microsoft-prod.deb")
                .add("cd /tmp && dpkg -i packages-microsoft-prod.deb")
                .add("apt-get update -y")
                .add("apt remove -y wget");

            builder.build()
        } else {
            vec![]
        }
    }
}

impl LanguageInstaller for NimInstaller {
    fn recipe(&self) -> Cow<'static, str> {
        let recipe = include_str!("recipes/nim_installer.sh");
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
    fn get_test_deps(&self, _codename: &str) -> Vec<String> {
        vec![]
    }
}

fn transform_name(input: &str, arch: &str) -> String {
    if let Some(pos) = input.find(format!("_{}", arch).as_str()) {
        let trimmed = &input[..pos];
        trimmed.replace('_', "=")
    } else {
        input.replace('_', "=")
    }
}
