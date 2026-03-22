use std::path::PathBuf;

use config::PkgConfig;

/// Concrete paths computed from config for the build pipeline.
pub struct BuildPaths {
    pub config_root: PathBuf,
    pub out_dir: PathBuf,
    pub src_dir: PathBuf,
    pub src_tarball: PathBuf,
    pub chroot_tarball: PathBuf,
    pub spec_file: PathBuf,
}

impl BuildPaths {
    pub fn from_config(config: &PkgConfig) -> Self {
        let pkg = &config.package;
        let env = &config.build_env;

        let out_dir = env
            .workdir
            .join(format!("{}-{}-{}", pkg.name, pkg.version, pkg.revision));
        let src_dir = out_dir.join(format!("{}-{}", pkg.name, pkg.version));
        let src_tarball = out_dir.join(format!("{}_{}.orig.tar.gz", pkg.name, pkg.version));

        let chroot_tarball = if let Some(ref date) = env.snapshot_date {
            env.chroot_dir
                .join(format!("{}-{}-{}.tar.gz", env.distribution.as_short(), env.arch, date))
        } else {
            env.chroot_dir
                .join(format!("{}-{}.tar.gz", env.distribution.as_short(), env.arch))
        };

        // spec is already resolved by config loader (relative paths joined with config_root)
        let spec_file = pkg.spec.clone();

        BuildPaths {
            config_root: config.config_root.clone(),
            out_dir,
            src_dir,
            src_tarball,
            chroot_tarball,
            spec_file,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::build_env::*;
    use config::package::PackageFields;
    use config::source::SourceKind;

    #[test]
    fn test_paths_basic() {
        let config = PkgConfig {
            package: PackageFields {
                name: "hello".into(),
                version: "1.0.0".into(),
                revision: "1".into(),
                homepage: "https://example.com".into(),
                spec: PathBuf::from("/home/user/pkg/hello.sss"),
            },
            source: SourceKind::Virtual,
            build_env: BuildEnv {
                distribution: Distribution::bookworm(),
                arch: Architecture::Amd64,
                pkg_builder_version: "0.3.1".into(),
                chroot_dir: PathBuf::from("/var/cache/sbuild"),
                workdir: PathBuf::from("/tmp/work"),
                tool_versions: ToolVersions {
                    debcrafter: "abc123".into(),
                    sbuild: "0.85.6".into(),
                },
                snapshot_date: None,
                snapshot_security_date: None,
            },
            runtime: None,
            verify: None,
            config_root: PathBuf::from("/home/user/pkg"),
        };

        let paths = BuildPaths::from_config(&config);
        assert_eq!(paths.out_dir, PathBuf::from("/tmp/work/hello-1.0.0-1"));
        assert_eq!(paths.src_dir, PathBuf::from("/tmp/work/hello-1.0.0-1/hello-1.0.0"));
        assert_eq!(paths.src_tarball, PathBuf::from("/tmp/work/hello-1.0.0-1/hello_1.0.0.orig.tar.gz"));
        assert_eq!(paths.chroot_tarball, PathBuf::from("/var/cache/sbuild/bookworm-amd64.tar.gz"));
        assert_eq!(paths.spec_file, PathBuf::from("/home/user/pkg/hello.sss"));
    }

    #[test]
    fn test_paths_with_snapshot() {
        let config = PkgConfig {
            package: PackageFields {
                name: "hello".into(),
                version: "1.0.0".into(),
                revision: "1".into(),
                homepage: "https://example.com".into(),
                spec: PathBuf::from("/home/user/pkg/hello.sss"),
            },
            source: SourceKind::Virtual,
            build_env: BuildEnv {
                distribution: Distribution::bookworm(),
                arch: Architecture::Amd64,
                pkg_builder_version: "0.3.1".into(),
                chroot_dir: PathBuf::from("/var/cache/sbuild"),
                workdir: PathBuf::from("/tmp/work"),
                tool_versions: ToolVersions {
                    debcrafter: "abc123".into(),
                    sbuild: "0.85.6".into(),
                },
                snapshot_date: Some("20240101T000000Z".into()),
                snapshot_security_date: None,
            },
            runtime: None,
            verify: None,
            config_root: PathBuf::from("/home/user/pkg"),
        };

        let paths = BuildPaths::from_config(&config);
        assert_eq!(
            paths.chroot_tarball,
            PathBuf::from("/var/cache/sbuild/bookworm-amd64-20240101T000000Z.tar.gz")
        );
    }
}
