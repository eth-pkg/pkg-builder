use config::language::JavascriptConfig;

use crate::Runtime;
use crate::script::ScriptBuilder;

pub struct NodeRuntime(pub JavascriptConfig);

impl Runtime for NodeRuntime {
    fn install_commands(&self) -> Vec<String> {
        let mut s = ScriptBuilder::new();
        s.cmd("apt install -y wget");
        s.cmd(format!(
            "cd /tmp && wget -q -O node.tar.gz {}",
            self.0.node_binary_url
        ));
        s.cmd(format!(
            "cd /tmp && echo \"{} node.tar.gz\" >> hash_file.txt && cat hash_file.txt",
            self.0.node_binary_checksum
        ));
        s.cmd("cd /tmp && sha256sum -c hash_file.txt");
        s.cmd("cd /tmp && rm -rf /usr/share/node && mkdir /usr/share/node && tar -C /usr/share/node -xzf node.tar.gz --strip-components=1");
        s.cmd("ls -l /usr/share/node/bin");
        s.cmd("ln -s /usr/share/node/bin/node /usr/bin/node");
        s.cmd("ln -s /usr/share/node/bin/npm /usr/bin/npm");
        s.cmd("ln -s /usr/share/node/bin/npx /usr/bin/npx");
        s.cmd("ln -s /usr/share/node/bin/corepack /usr/bin/corepack");
        s.cmd("apt remove -y wget");
        s.cmd("node --version");
        s.cmd("npm --version");

        // Yarn support
        if let Some(ref yarn_version) = self.0.yarn_version {
            s.cmd(format!("npm install --global yarn@{}", yarn_version));
            s.cmd("ln -s /usr/share/node/bin/yarn /usr/bin/yarn");
            s.cmd("yarn --version");
        }

        s.build()
    }
}
