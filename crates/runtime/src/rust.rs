use config::language::RustConfig;

use crate::Runtime;
use crate::script::ScriptBuilder;

pub struct RustRuntime(pub RustConfig);

impl Runtime for RustRuntime {
    fn install_commands(&self) -> Vec<String> {
        let mut s = ScriptBuilder::new();
        s.cmd("apt install -y wget gpg gpg-agent");
        s.cmd(format!(
            "cd /tmp && wget -q -O package.tar.xz {}",
            self.0.rust_binary_url
        ));
        s.cmd(format!(
            "cd /tmp && echo \"{}\" >> package.tar.xz.asc",
            self.0.rust_binary_gpg_asc
        ));
        s.cmd("wget -qO- https://keybase.io/rust/pgp_keys.asc | gpg --import");
        s.cmd("cd /tmp && gpg --verify package.tar.xz.asc package.tar.xz");
        s.cmd("cd /tmp && tar xvJf package.tar.xz -C . --strip-components=1");
        s.cmd("cd /tmp && /bin/bash install.sh");
        s.cmd("apt remove -y wget gpg gpg-agent");
        s.build()
    }
}
