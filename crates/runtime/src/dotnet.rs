use config::build_env::{Architecture, Distribution, UbuntuCodename};
use config::language::DotnetConfig;

use crate::Runtime;
use crate::script::ScriptBuilder;

pub struct DotnetRuntime(pub DotnetConfig);

impl Runtime for DotnetRuntime {
    fn install_commands(&self) -> Vec<String> {
        // Dotnet uses build_deps directly, install_commands is not used via recipe
        vec![]
    }

    fn build_deps(&self, arch: &Architecture, dist: &Distribution) -> Vec<String> {
        let mut s = ScriptBuilder::new();

        if self.0.use_backup_version {
            s.cmd("apt install -y wget");
            s.cmd("apt install -y libicu-dev");

            for package in self.0.deps.as_deref().unwrap_or_default() {
                s.cmd(format!("apt install -y {}", package));
            }

            for package in &self.0.dotnet_packages {
                s.cmd(format!("cd /tmp && wget -q {}", package.url));
                s.cmd(format!("cd /tmp && ls && dpkg -i {}.deb", package.name));
                s.cmd(format!("cd /tmp && ls && sha1sum {}.deb", package.name));
                s.cmd(format!(
                    "cd /tmp &&  echo {} {}.deb > hash_file.txt && cat hash_file.txt",
                    package.hash, package.name
                ));
                s.cmd("cd /tmp && sha1sum -c hash_file.txt");
            }

            s.cmd("dotnet --version");
            s.cmd("apt remove -y wget");
        } else {
            match dist {
                Distribution::Debian(_) | Distribution::Ubuntu(UbuntuCodename::Jammy) => {
                    s.cmd("apt install -y wget");
                    s.cmd("cd /tmp && wget -q https://packages.microsoft.com/config/debian/12/packages-microsoft-prod.deb -O packages-microsoft-prod.deb");
                    s.cmd("cd /tmp && dpkg -i packages-microsoft-prod.deb");
                    s.cmd("apt update -y");

                    for package in &self.0.dotnet_packages {
                        let pkg = transform_name(&package.name, arch);
                        s.cmd(format!("cd /tmp && wget -q {}", package.url));
                        s.cmd(format!(
                            "cd /tmp && apt install -y --allow-downgrades {}",
                            pkg
                        ));
                        s.cmd(format!("cd /tmp && apt download -y {}", pkg));
                        s.cmd(format!("cd /tmp && ls && sha1sum {}.deb", package.name));
                        s.cmd(format!(
                            "cd /tmp &&  echo {} {}.deb >> hash_file.txt && cat hash_file.txt",
                            package.hash, package.name
                        ));
                        s.cmd("cd /tmp && sha1sum -c hash_file.txt");
                    }

                    s.cmd("dotnet --version");
                    s.cmd("apt remove -y wget");
                }
                Distribution::Ubuntu(UbuntuCodename::Noble) => {
                    s.cmd("apt-get install software-properties-common -y");
                    s.cmd("add-apt-repository ppa:dotnet/backports");
                    s.cmd("apt-get update -y");
                    s.cmd("apt install -y wget");

                    for package in &self.0.dotnet_packages {
                        let pkg = transform_name(&package.name, arch);
                        s.cmd(format!("cd /tmp && wget -q {}", package.url));
                        s.cmd(format!("cd /tmp && apt install -y {}", pkg));
                        s.cmd(format!("cd /tmp && apt download -y {}", pkg));
                        s.cmd(format!("cd /tmp && ls && sha1sum {}.deb", package.name));
                        s.cmd(format!(
                            "cd /tmp &&  echo {} {}.deb >> hash_file.txt && cat hash_file.txt",
                            package.hash, package.name
                        ));
                        s.cmd("cd /tmp && sha1sum -c hash_file.txt");
                    }

                    s.cmd("dotnet --version");
                    s.cmd("apt remove -y wget");
                }
            }
        }

        s.build()
    }

    fn test_deps(&self, dist: &Distribution) -> Vec<String> {
        let mut s = ScriptBuilder::new();

        match dist {
            Distribution::Debian(_) | Distribution::Ubuntu(UbuntuCodename::Jammy) => {
                s.cmd("apt install -y wget");
                s.cmd("cd /tmp && wget https://packages.microsoft.com/config/debian/12/packages-microsoft-prod.deb -O packages-microsoft-prod.deb");
                s.cmd("cd /tmp && dpkg -i packages-microsoft-prod.deb");
                s.cmd("apt-get update -y");
                s.cmd("apt remove -y wget");
            }
            Distribution::Ubuntu(UbuntuCodename::Noble) => {
                return vec![];
            }
        }

        s.build()
    }
}

fn transform_name(input: &str, arch: &Architecture) -> String {
    let arch_str = format!("_{}", arch);
    if let Some(pos) = input.find(&arch_str) {
        let trimmed = &input[..pos];
        trimmed.replace('_', "=")
    } else {
        input.replace('_', "=")
    }
}
