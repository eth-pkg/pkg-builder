use std::path::PathBuf;
use std::sync::Arc;

use config::PkgConfig;

use crate::PipelineError;

/// Workspace holds all derived paths and the shared config.
/// Replaces BuildContext + SbuildArgs.
pub struct Workspace {
    pub config: Arc<PkgConfig>,
    pub build_artifacts_dir: PathBuf,
    pub build_files_dir: PathBuf,
    pub tarball_path: PathBuf,
    pub deb_path: PathBuf,
    pub changes_path: PathBuf,
    pub cache_file: PathBuf,
    pub src_dir: PathBuf,
}

impl Workspace {
    /// Derives all paths from config.
    pub fn new(config: Arc<PkgConfig>) -> Result<Self, PipelineError> {
        let pkg = &config.package;
        let env = &config.build_env;

        let workdir = &env.workdir;

        let build_artifacts_dir = workdir.join(format!(
            "{}-{}-{}",
            pkg.name, pkg.version, pkg.revision
        ));

        let build_files_dir = build_artifacts_dir.join(format!(
            "{}-{}",
            pkg.name, pkg.version
        ));

        let tarball_path = build_artifacts_dir.join(format!(
            "{}_{}.orig.tar.gz",
            pkg.name, pkg.version
        ));

        let deb_path = build_artifacts_dir.join(format!(
            "{}_{}-{}_{}.deb",
            pkg.name, pkg.version, pkg.revision, env.arch
        ));

        let changes_path = build_artifacts_dir.join(format!(
            "{}_{}-{}_{}.changes",
            pkg.name, pkg.version, pkg.revision, env.arch
        ));

        let cache_dir_expanded =
            shellexpand::tilde(&env.chroot_dir.display().to_string()).to_string();
        let cache_file_name = match &env.snapshot_date {
            Some(date) => format!("{}-{}-{}.tar.gz", env.distribution.as_short(), env.arch, date),
            None => format!("{}-{}.tar.gz", env.distribution.as_short(), env.arch),
        };
        let cache_file = PathBuf::from(cache_dir_expanded).join(cache_file_name);

        let src_dir = config.config_root.join("src");

        Ok(Workspace {
            config,
            build_artifacts_dir,
            build_files_dir,
            tarball_path,
            deb_path,
            changes_path,
            cache_file,
            src_dir,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::build_env::*;
    use config::package::PackageFields;
    use config::source::SourceKind;

    fn test_config(workdir: &str) -> PkgConfig {
        PkgConfig {
            package: PackageFields {
                spec: "hello.sss".into(),
                name: "hello-world".to_string(),
                version: "1.0.0".to_string(),
                revision: "1".to_string(),
                homepage: "https://example.com".to_string(),
            },
            source: SourceKind::Tarball {
                url: "test.tar.gz".to_string(),
                hash: Some("abc123".to_string()),
            },
            build_env: BuildEnv {
                distribution: Distribution::bookworm(),
                arch: Architecture::Amd64,
                pkg_builder_version: "0.3.1".to_string(),

                chroot_dir: PathBuf::from("/tmp/cache/sbuild"),
                workdir: PathBuf::from(workdir),
                testing: TestingConfig {
                    run_lintian: true,
                    run_piuparts: false,
                    run_autopkgtest: false,
                },
                tool_versions: ToolVersions {
                    debcrafter: "8189263".to_string(),
                    sbuild: "0.85.6".to_string(),
                    lintian: "2.116.3".to_string(),
                    piuparts: "1.1.7".to_string(),
                    autopkgtest: "5.28".to_string(),
                },
                snapshot_date: None,
                snapshot_security_date: None,
            },
            runtime: None,
            verify: None,
            config_root: PathBuf::from("/test/config"),
        }
    }

    #[test]
    fn test_workspace_build_artifacts_dir() {
        let cfg = test_config("/tmp/workdir/packages");
        let ws = Workspace::new(Arc::new(cfg)).unwrap();
        assert_eq!(
            ws.build_artifacts_dir,
            PathBuf::from("/tmp/workdir/packages/hello-world-1.0.0-1")
        );
    }

    #[test]
    fn test_workspace_build_files_dir() {
        let cfg = test_config("/tmp/workdir/packages");
        let ws = Workspace::new(Arc::new(cfg)).unwrap();
        assert_eq!(
            ws.build_files_dir,
            PathBuf::from("/tmp/workdir/packages/hello-world-1.0.0-1/hello-world-1.0.0")
        );
    }

    #[test]
    fn test_workspace_tarball_path() {
        let cfg = test_config("/tmp/workdir/packages");
        let ws = Workspace::new(Arc::new(cfg)).unwrap();
        assert_eq!(
            ws.tarball_path,
            PathBuf::from("/tmp/workdir/packages/hello-world-1.0.0-1/hello-world_1.0.0.orig.tar.gz")
        );
    }

    #[test]
    fn test_workspace_deb_path() {
        let cfg = test_config("/tmp/workdir/packages");
        let ws = Workspace::new(Arc::new(cfg)).unwrap();
        assert_eq!(
            ws.deb_path,
            PathBuf::from("/tmp/workdir/packages/hello-world-1.0.0-1/hello-world_1.0.0-1_amd64.deb")
        );
    }

    #[test]
    fn test_workspace_changes_path() {
        let cfg = test_config("/tmp/workdir/packages");
        let ws = Workspace::new(Arc::new(cfg)).unwrap();
        assert_eq!(
            ws.changes_path,
            PathBuf::from("/tmp/workdir/packages/hello-world-1.0.0-1/hello-world_1.0.0-1_amd64.changes")
        );
    }

    #[test]
    fn test_workspace_cache_file() {
        let cfg = test_config("/tmp/workdir/packages");
        let ws = Workspace::new(Arc::new(cfg)).unwrap();
        assert_eq!(
            ws.cache_file,
            PathBuf::from("/tmp/cache/sbuild/bookworm-amd64.tar.gz")
        );
    }

    #[test]
    fn test_workspace_src_dir() {
        let cfg = test_config("/tmp/workdir/packages");
        let ws = Workspace::new(Arc::new(cfg)).unwrap();
        assert_eq!(ws.src_dir, PathBuf::from("/test/config/src"));
    }

    #[test]
    fn test_workspace_cache_file_with_snapshot() {
        let mut cfg = test_config("/tmp/workdir/packages");
        cfg.build_env.snapshot_date = Some("20250101T000000Z".to_string());
        let ws = Workspace::new(Arc::new(cfg)).unwrap();
        assert_eq!(
            ws.cache_file,
            PathBuf::from("/tmp/cache/sbuild/bookworm-amd64-20250101T000000Z.tar.gz")
        );
    }
}
