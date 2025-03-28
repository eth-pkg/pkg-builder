use std::{borrow::Cow, collections::HashMap};

use types::{config::Architecture, distribution::{Distribution, UbuntuCodename}};

use crate::configs::pkg_config::DotnetConfig;

use super::{command_builder::CommandBuilder, language_installer::LanguageInstaller};

pub struct DotnetInstaller(pub(crate) DotnetConfig);

impl LanguageInstaller for DotnetInstaller {
    fn recipe(&self) -> Cow<'static, str> {
        let recipe = include_str!("../recipes/dotnet_installer.sh");
        Cow::Borrowed(&recipe)
    }

    fn substitutions(&self) -> HashMap<&str, &str> {
        let subs = HashMap::new();
        subs
    }
    fn get_build_deps(&self, arch: &Architecture, codename: &Distribution) -> Vec<String> {
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
                    .add_with("cd /tmp && wget -q {}", &package.url.as_ref())
                    .add_with("cd /tmp && ls && dpkg -i {}.deb", &package.name)
                    .add_with("cd /tmp && ls && sha1sum {}.deb", &package.name)
                    .add_with_args(
                        "cd /tmp &&  echo {} {}.deb > hash_file.txt && cat hash_file.txt",
                        &[&package.hash, &package.name],
                    )
                    .add("cd /tmp && sha1sum -c hash_file.txt");
            }

            builder.add("dotnet --version").add("apt remove -y wget");
        } else {
            match codename {
                Distribution::Debian(_) | Distribution::Ubuntu(UbuntuCodename::Jammy) => {
                    builder
                    .add("apt install -y wget")
                    .add("cd /tmp && wget -q https://packages.microsoft.com/config/debian/12/packages-microsoft-prod.deb -O packages-microsoft-prod.deb")
                    .add("cd /tmp && dpkg -i packages-microsoft-prod.deb")
                    .add("apt update -y");

                    for package in dotnet_packages {
                        let pkg = transform_name(&package.name, arch);
                        builder
                            .add_with("cd /tmp && wget -q {}", &package.url.as_ref())
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
                }
                Distribution::Ubuntu(UbuntuCodename::Noble) => {
                    builder
                        .add("apt-get install software-properties-common -y")
                        .add("add-apt-repository ppa:dotnet/backports")
                        .add("apt-get update -y")
                        .add("apt install -y wget");

                    for package in dotnet_packages {
                        let pkg = transform_name(&package.name, arch);
                        builder
                            .add_with("cd /tmp && wget -q {}", &package.url.as_ref())
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
            }
        }

        builder.build()
    }
    fn get_test_deps(&self, codename: &Distribution) -> Vec<String> {
        let mut builder = CommandBuilder::new();

        match codename {
            Distribution::Debian(_) | Distribution::Ubuntu(UbuntuCodename::Jammy) => {
                builder
                .add("apt install -y wget")
                .add("cd /tmp && wget https://packages.microsoft.com/config/debian/12/packages-microsoft-prod.deb -O packages-microsoft-prod.deb")
                .add("cd /tmp && dpkg -i packages-microsoft-prod.deb")
                .add("apt-get update -y")
                .add("apt remove -y wget");

                builder.build()
            }
            Distribution::Ubuntu(UbuntuCodename::Noble) => {
                vec![]
            }
        }
    }
}

fn transform_name(input: &str, arch: &Architecture) -> String {
    if let Some(pos) = input.find(format!("_{}", arch).as_str()) {
        let trimmed = &input[..pos];
        trimmed.replace('_', "=")
    } else {
        input.replace('_', "=")
    }
}
