use config::build_env::DebianCodename;
use config::PkgConfig;

use crate::runtime::{load_runtime_perl, substitute_template};
use crate::ExecutorError;

/// Chroot setup commands and runtime Perl code for sbuild.conf.
pub struct ChrootSetup {
    pub runtime_perl: Option<String>,
    pub pre_runtime_commands: Vec<String>,
    pub post_runtime_commands: Vec<String>,
}

impl ChrootSetup {
    pub fn from_config(config: &PkgConfig) -> Result<Self, ExecutorError> {
        let mut pre = Vec::new();
        let mut post = Vec::new();

        // Snapshot workaround (must come first)
        if config.build_env.uses_snapshot() {
            pre.push(
                r#"echo 'Acquire::Check-Valid-Until "false";' > /etc/apt/apt.conf.d/99snapshot"#
                    .to_string(),
            );
        }

        // Noble/distribution repos (before runtime)
        for cmd in config.build_env.distribution.extra_chroot_commands() {
            pre.push(cmd);
        }

        // Runtime Perl template (substitute {{var}} placeholders)
        let runtime_perl = if let Some(ref rt) = config.runtime {
            let template = load_runtime_perl(&rt.profile, &config.config_root)?;
            Some(substitute_template(&template, config))
        } else {
            None
        };

        // Snapshot security (must come after runtime)
        if let Some(security_url) = config.build_env.security_repo_url() {
            let codename = match &config.build_env.distribution {
                config::build_env::Distribution::Debian(DebianCodename::Bookworm) => "bookworm",
                config::build_env::Distribution::Debian(DebianCodename::Trixie) => "trixie",
                _ => config.build_env.distribution.as_short(),
            };
            post.push(format!(
                "echo 'deb {} {}-security main' > /etc/apt/sources.list.d/security-snapshot.list",
                security_url, codename
            ));
            post.push("apt-get update".to_string());
        }

        Ok(ChrootSetup {
            runtime_perl,
            pre_runtime_commands: pre,
            post_runtime_commands: post,
        })
    }
}
