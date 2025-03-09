use crate::v1::pkg_config::{
    DotnetConfig, GoConfig, JavaConfig, JavascriptConfig, LanguageEnv, NimConfig, RustConfig,
};

use super::command_builder::CommandBuilder;

pub trait LanguageInstaller {
    fn get_build_deps(&self, arch: &str, codename: &str) -> Vec<String>;
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
impl LanguageInstaller for EmptyInstaller {
    fn get_build_deps(&self, _arch: &str, _codename: &str) -> Vec<String> {
        vec![]
    }
    fn get_test_deps(&self, _codename: &str) -> Vec<String> {
        vec![]
    }
}
pub struct GoInstaller(GoConfig);
pub struct NodeInstaller(JavascriptConfig);
pub struct JavaInstaller(JavaConfig);
pub struct DotnetInstaller(DotnetConfig);
pub struct NimInstaller(NimConfig);
pub struct RustInstaller(RustConfig);

impl LanguageInstaller for RustInstaller {
    fn get_build_deps(&self, _arch: &str, _codename: &str) -> Vec<String> {
        let mut builder = CommandBuilder::new();
        builder
            .add("apt install -y wget gpg gpg-agent")
            .add_with(
                "cd /tmp && wget -q -O package.tar.xz {}",
                &self.0.rust_binary_url,
            )
            .add_with(
                "cd /tmp && echo \"{}\" >> package.tar.xz.asc",
                &self.0.rust_binary_gpg_asc,
            )
            .add("wget -qO- https://keybase.io/some-key/pgp_keys.asc | gpg --import")
            .add("cd /tmp && gpg --verify package.tar.xz.asc package.tar.xz")
            .add("cd /tmp && tar xvJf package.tar.xz -C . --strip-components=1")
            .add("cd /tmp && /bin/bash install.sh")
            .add("apt remove -y wget gpg gpg-agent");

        builder.build()
    }

    fn get_test_deps(&self, _codename: &str) -> Vec<String> {
        vec![] // Rust compiles to binary, no test deps needed
    }
}

impl LanguageInstaller for GoInstaller {
    fn get_build_deps(&self, _arch: &str, _codename: &str) -> Vec<String> {
        let go_binary_url = &self.0.go_binary_url;
        let go_binary_checksum = &self.0.go_binary_checksum;

        let mut builder = CommandBuilder::new();
        builder
            .add("apt install -y wget")
            .add_with("cd /tmp && wget -q -O go.tar.gz {}", go_binary_url)
            .add_with("cd /tmp && echo \"{} go.tar.gz\" >> hash_file.txt && cat hash_file.txt", go_binary_checksum)
            .add("cd /tmp && sha256sum -c hash_file.txt")
            .add("cd /tmp && rm -rf /usr/local/go && mkdir /usr/local/go && tar -C /usr/local -xzf go.tar.gz")
            .add("ln -s /usr/local/go/bin/go /usr/bin/go")
            .add("go version")
            .add("chmod -R a+rwx /usr/local/go/pkg")
            .add("apt remove -y wget");

        builder.build()
    }
    fn get_test_deps(&self, _codename: &str) -> Vec<String> {
        vec![]
    }
}

impl LanguageInstaller for NodeInstaller {
    fn get_build_deps(&self, _arch: &str, _codename: &str) -> Vec<String> {
        // let node_version = &self.0.go_version; // Comment preserved as in original
        let node_binary_url = &self.0.node_binary_url;
        let node_binary_checksum = &self.0.node_binary_checksum;

        let mut builder = CommandBuilder::new();
        builder
        .add("apt install -y wget")
        .add_with("cd /tmp && wget -q -O node.tar.gz {}", node_binary_url)
        .add_with("cd /tmp && echo \"{} node.tar.gz\" >> hash_file.txt && cat hash_file.txt", node_binary_checksum)
        .add("cd /tmp && sha256sum -c hash_file.txt")
        .add("cd /tmp && rm -rf /usr/share/node && mkdir /usr/share/node && tar -C /usr/share/node -xzf node.tar.gz --strip-components=1")
        .add("ls -l /usr/share/node/bin")
        .add("ln -s /usr/share/node/bin/node /usr/bin/node")
        .add("ln -s /usr/share/node/bin/npm /usr/bin/npm")
        .add("ln -s /usr/share/node/bin/npx /usr/bin/npx")
        .add("ln -s /usr/share/node/bin/corepack /usr/bin/corepack")
        .add("apt remove -y wget")
        .add("node --version")
        .add("npm --version");

        if let Some(yarn_version) = &self.0.yarn_version {
            builder
                .add_with("npm install --global yarn@{}", yarn_version)
                .add("ln -s /usr/share/node/bin/yarn /usr/bin/yarn")
                .add("yarn --version");
        }

        builder.build()
    }
    fn get_test_deps(&self, _codename: &str) -> Vec<String> {
        vec![]
    }
}

impl LanguageInstaller for JavaInstaller {
    fn get_build_deps(&self, _arch: &str, _codename: &str) -> Vec<String> {
        let is_oracle = self.0.is_oracle;
        if !is_oracle {
            return Vec::new(); // Return empty vector if not Oracle
        }

        let jdk_version = &self.0.jdk_version;
        let jdk_binary_url = &self.0.jdk_binary_url;
        let jdk_binary_checksum = &self.0.jdk_binary_checksum;

        let mut builder = CommandBuilder::new();
        builder
            .add("apt install -y wget")
            .add_with(
                "mkdir -p /opt/lib/jvm/jdk-{}-oracle && mkdir -p /usr/lib/jvm",
                jdk_version,
            )
            .add_with(
                "cd /tmp && wget -q --output-document jdk.tar.gz {}",
                jdk_binary_url,
            )
            .add_with(
                "cd /tmp && echo \"{} jdk.tar.gz\" >> hash_file.txt && cat hash_file.txt",
                jdk_binary_checksum,
            )
            .add("cd /tmp && sha256sum -c hash_file.txt")
            .add_with(
                "cd /tmp && tar -zxf jdk.tar.gz -C /opt/lib/jvm/jdk-{}-oracle --strip-components=1",
                jdk_version,
            )
            .add_with(
                "ln -s /opt/lib/jvm/jdk-{}-oracle/bin/java /usr/bin/java",
                jdk_version,
            )
            .add_with(
                "ln -s /opt/lib/jvm/jdk-{}-oracle/bin/javac /usr/bin/javac",
                jdk_version,
            )
            .add("java -version")
            .add("apt remove -y wget");

        if let Some(gradle_config) = &self.0.gradle {
            let gradle_version = &gradle_config.gradle_version;
            let gradle_binary_url = &gradle_config.gradle_binary_url;
            let gradle_binary_checksum = &gradle_config.gradle_binary_checksum;

            builder
                .add("apt install -y wget unzip")
                .add_with("mkdir -p /opt/lib/gradle-{}", gradle_version)
                .add_with(
                    "cd /tmp && wget -q --output-document gradle.tar.gz {}",
                    gradle_binary_url,
                )
                .add_with(
                    "cd /tmp && echo \"{} gradle.tar.gz\" > hash_file.txt && cat hash_file.txt",
                    gradle_binary_checksum,
                )
                .add("cd /tmp && sha256sum -c hash_file.txt")
                .add_with(
                    "cd /tmp && unzip gradle.tar.gz && mv gradle-{} /opt/lib",
                    gradle_version,
                )
                .add_with(
                    "ln -s /opt/lib/gradle-{}/bin/gradle /usr/bin/gradle",
                    gradle_version,
                )
                .add("gradle -version")
                .add("apt remove -y wget");
        }

        builder.build()
    }
    fn get_test_deps(&self, _codename: &str) -> Vec<String> {
        vec![]
    }
}

impl LanguageInstaller for DotnetInstaller {
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
        } else if codename == "noble numbat" {
            Vec::new()
        } else {
            Vec::new()
        }
    }
}

impl LanguageInstaller for NimInstaller {
    fn get_build_deps(&self, _arch: &str, _codename: &str) -> Vec<String> {
        let nim_version = &self.0.nim_version;
        let nim_binary_url = &self.0.nim_binary_url;
        let nim_version_checksum = &self.0.nim_version_checksum;

        let mut builder = CommandBuilder::new();
        builder
            .add("apt install -y wget")
            .add_with("rm -rf /tmp/nim-{} && rm -rf /usr/lib/nim/nim-{} && rm -rf /opt/lib/nim/nim-{} && mkdir /tmp/nim-{}", nim_version)
            .add("mkdir -p /opt/lib/nim && mkdir -p /usr/lib/nim")
            .add_with("cd /tmp && wget -q {}", nim_binary_url)
            .add_with("cd /tmp && echo {} >> hash_file.txt && cat hash_file.txt", nim_version_checksum)
            .add("cd /tmp && sha256sum -c hash_file.txt")
            .add_with("cd /tmp && tar xJf nim-{}-linux_x64.tar.xz -C nim-{} --strip-components=1", nim_version)
            .add_with("cd /tmp && mv nim-{} /opt/lib/nim", nim_version)
            .add_with("ln -s /opt/lib/nim/nim-{}/bin/nim /usr/bin/nim", nim_version)
            // format!("installed_version=`nim --version | head -n 1 | awk '{{print $4}}'` && echo \"installed version: $installed_version\" && [ \"$installed_version\" != \"{}\" ] && exit 1", nim_version),
            .add("nim --version")
            .add("apt remove -y wget");

        builder.build()
    }
    fn get_test_deps(&self, _codename: &str) -> Vec<String> {
        todo!()
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
