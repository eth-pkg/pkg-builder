use std::path::Path;

use config::build_env::Distribution;
use log::info;

use crate::command::run_command;
use crate::ToolError;

/// Lintian command wrapper.
pub struct Lintian<'a> {
    pub changes_file: &'a Path,
    pub distribution: &'a Distribution,
}

impl Lintian<'_> {
    pub fn run(&self) -> Result<(), ToolError> {
        let args = self.build_args();
        info!("Running: lintian {}", args.join(" "));
        run_command("lintian", &args, None)
    }

    fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        // Suppress standard tags
        args.push("--suppress-tags".to_string());
        args.push("bad-distribution-in-changes-file".to_string());
        args.push("--suppress-tags".to_string());
        args.push("debug-file-with-no-debug-symbols".to_string());

        // Distribution-specific suppressions
        for tag in self.distribution.lintian_suppressions() {
            args.push("--suppress-tags".to_string());
            args.push(tag.to_string());
        }

        args.push("-i".to_string());
        args.push("-I".to_string());

        args.push(format!("{:?}", self.changes_file));

        args.push("--tag-display-limit=0".to_string());
        args.push("--fail-on=warning".to_string());
        args.push("--fail-on=error".to_string());

        args
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::build_env::{DebianCodename, UbuntuCodename};
    use std::path::Path;

    #[test]
    fn test_lintian_build_args_debian() {
        let dist = Distribution::Debian(DebianCodename::Bookworm);
        let lintian = Lintian {
            changes_file: Path::new("/build/hello_1.0.0-1_amd64.changes"),
            distribution: &dist,
        };

        let args = lintian.build_args();
        assert!(args.contains(&"-i".to_string()));
        assert!(args.contains(&"-I".to_string()));
        assert!(args.contains(&"--fail-on=warning".to_string()));
        assert!(args.contains(&"--fail-on=error".to_string()));
        assert!(args.contains(&"--tag-display-limit=0".to_string()));
        // Standard suppressions
        assert!(args.contains(&"bad-distribution-in-changes-file".to_string()));
        assert!(args.contains(&"debug-file-with-no-debug-symbols".to_string()));
        // Debian should NOT have malformed-deb-archive
        assert!(!args.iter().any(|a| a == "malformed-deb-archive"));
    }

    #[test]
    fn test_lintian_build_args_ubuntu() {
        let dist = Distribution::Ubuntu(UbuntuCodename::Noble);
        let lintian = Lintian {
            changes_file: Path::new("/build/hello_1.0.0-1_amd64.changes"),
            distribution: &dist,
        };

        let args = lintian.build_args();
        // Ubuntu should have malformed-deb-archive suppression
        assert!(args.iter().any(|a| a == "malformed-deb-archive"));
    }
}
