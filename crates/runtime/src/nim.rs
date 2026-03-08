use config::language::NimConfig;

use crate::Runtime;
use crate::script::ScriptBuilder;

pub struct NimRuntime(pub NimConfig);

impl Runtime for NimRuntime {
    fn install_commands(&self) -> Vec<String> {
        let mut s = ScriptBuilder::new();
        s.cmd("apt install -y wget");
        s.cmd(format!(
            "rm -rf /tmp/nim-{v} && rm -rf /usr/lib/nim/nim-{v} && rm -rf /opt/lib/nim/nim-{v} && mkdir /tmp/nim-{v}",
            v = self.0.nim_version
        ));
        s.cmd("mkdir -p /opt/lib/nim && mkdir -p /usr/lib/nim");
        s.cmd(format!(
            "cd /tmp && wget -q {}",
            self.0.nim_binary_url
        ));
        s.cmd(format!(
            "cd /tmp && echo {} >> hash_file.txt && cat hash_file.txt",
            self.0.nim_version_checksum
        ));
        s.cmd("cd /tmp && sha256sum -c hash_file.txt");
        s.cmd(format!(
            "cd /tmp && tar xJf nim-{v}-linux_x64.tar.xz -C nim-{v} --strip-components=1",
            v = self.0.nim_version
        ));
        s.cmd(format!(
            "cd /tmp && mv nim-{v} /opt/lib/nim",
            v = self.0.nim_version
        ));
        s.cmd(format!(
            "ln -s /opt/lib/nim/nim-{v}/bin/nim /usr/bin/nim",
            v = self.0.nim_version
        ));
        s.cmd("nim --version");
        s.cmd("apt remove -y wget");
        s.build()
    }
}
