use config::language::GoConfig;

use crate::Runtime;
use crate::script::ScriptBuilder;

pub struct GoRuntime(pub GoConfig);

impl Runtime for GoRuntime {
    fn install_commands(&self) -> Vec<String> {
        let mut s = ScriptBuilder::new();
        s.cmd("apt install -y wget");
        s.cmd(format!(
            "cd /tmp && wget -q -O go.tar.gz {}",
            self.0.go_binary_url
        ));
        s.cmd(format!(
            "cd /tmp && echo \"{} go.tar.gz\" >> hash_file.txt && cat hash_file.txt",
            self.0.go_binary_checksum
        ));
        s.cmd("cd /tmp && sha256sum -c hash_file.txt");
        s.cmd("cd /tmp && rm -rf /usr/local/go && mkdir /usr/local/go && tar -C /usr/local -xzf go.tar.gz");
        s.cmd("ln -s /usr/local/go/bin/go /usr/bin/go");
        s.cmd("go version");
        s.cmd("chmod -R a+rwx /usr/local/go/pkg");
        s.cmd("apt remove -y wget");
        s.build()
    }
}
