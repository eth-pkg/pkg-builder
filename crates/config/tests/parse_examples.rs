//! Integration tests that parse every example TOML config file.
//! This ensures the new config crate is backwards-compatible with all existing configs.

use config::build_env::{Architecture, Distribution};
use config::language::LanguageEnv;
use config::source::SourceKind;
use config::verify::PkgVerifyConfig;
use config::PkgConfig;
use std::path::Path;

fn examples_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("examples")
        .leak()
}

fn load_example(distro: &str, lang: &str, name: &str) -> PkgConfig {
    let path = examples_dir().join(distro).join(lang).join(name);
    PkgConfig::load(&path).unwrap_or_else(|e| {
        panic!(
            "Failed to load config at {}: {}",
            path.display(),
            e
        )
    })
}

fn load_verify(distro: &str, lang: &str, name: &str) -> PkgVerifyConfig {
    let path = examples_dir()
        .join(distro)
        .join(lang)
        .join(name)
        .join("pkg-builder-verify.toml");
    PkgVerifyConfig::load(&path).unwrap_or_else(|e| {
        panic!(
            "Failed to load verify config at {}: {}",
            path.display(),
            e
        )
    })
}

// ─── Bookworm examples ───

#[test]
fn parse_bookworm_c() {
    let cfg = load_example("bookworm", "c", "hello-world");
    assert_eq!(cfg.package.package_name, "hello-world");
    assert_eq!(cfg.package.version_number, "1.0.0");
    assert_eq!(cfg.package.revision_number, "1");
    assert!(matches!(cfg.build_env.distribution, Distribution::Debian(_)));
    assert!(matches!(cfg.build_env.arch, Architecture::Amd64));
    match &cfg.source {
        SourceKind::Tarball { hash, language, .. } => {
            assert!(hash.is_some());
            assert!(matches!(language, LanguageEnv::C));
        }
        _ => panic!("Expected Tarball source"),
    }
    assert!(cfg.build_env.testing.run_lintian);
    assert!(cfg.build_env.testing.run_piuparts);
    assert!(cfg.build_env.testing.run_autopkgtest);
}

#[test]
fn parse_bookworm_rust() {
    let cfg = load_example("bookworm", "rust", "hello-world");
    assert_eq!(cfg.package.package_name, "hello-world-rust");
    match &cfg.source {
        SourceKind::Tarball { language, .. } => {
            match language {
                LanguageEnv::Rust(rc) => {
                    assert_eq!(rc.rust_version, "1.77.2");
                    assert!(rc.rust_binary_url.contains("rust-lang.org"));
                    assert!(rc.rust_binary_gpg_asc.contains("BEGIN PGP SIGNATURE"));
                }
                _ => panic!("Expected Rust language"),
            }
        }
        _ => panic!("Expected Tarball source"),
    }
}

#[test]
fn parse_bookworm_go() {
    let cfg = load_example("bookworm", "go", "hello-world");
    assert_eq!(cfg.package.package_name, "hello-world-go");
    match &cfg.source {
        SourceKind::Tarball { language, .. } => {
            match language {
                LanguageEnv::Go(gc) => {
                    assert_eq!(gc.go_version, "1.22.2");
                    assert!(gc.go_binary_url.contains("go.dev"));
                    assert!(!gc.go_binary_checksum.is_empty());
                }
                _ => panic!("Expected Go language"),
            }
        }
        _ => panic!("Expected Tarball source"),
    }
}

#[test]
fn parse_bookworm_java() {
    let cfg = load_example("bookworm", "java", "hello-world");
    assert_eq!(cfg.package.package_name, "hello-world-java");
    match &cfg.source {
        SourceKind::Tarball { language, .. } => {
            match language {
                LanguageEnv::Java(jc) => {
                    assert!(jc.is_oracle);
                    assert_eq!(jc.jdk_version, "17.0");
                    assert!(jc.gradle.is_none());
                }
                _ => panic!("Expected Java language"),
            }
        }
        _ => panic!("Expected Tarball source"),
    }
}

#[test]
fn parse_bookworm_java_gradle() {
    let cfg = load_example("bookworm", "java-gradle", "hello-world");
    assert_eq!(cfg.package.package_name, "hello-world-java-gradle");
    match &cfg.source {
        SourceKind::Tarball { language, .. } => {
            match language {
                LanguageEnv::Java(jc) => {
                    assert!(jc.is_oracle);
                    let gradle = jc.gradle.as_ref().expect("Expected gradle config");
                    assert_eq!(gradle.gradle_version, "8.7");
                    assert!(gradle.gradle_binary_url.contains("gradle"));
                }
                _ => panic!("Expected Java language"),
            }
        }
        _ => panic!("Expected Tarball source"),
    }
}

#[test]
fn parse_bookworm_javascript() {
    let cfg = load_example("bookworm", "javascript", "hello-world");
    assert_eq!(cfg.package.package_name, "hello-world-javascript");
    match &cfg.source {
        SourceKind::Tarball { language, .. } => {
            match language {
                LanguageEnv::JavaScript(nc) => {
                    assert_eq!(nc.node_version, "20.12.2");
                    assert_eq!(nc.yarn_version, Some("1.22.19".to_string()));
                }
                _ => panic!("Expected JavaScript language"),
            }
        }
        _ => panic!("Expected Tarball source"),
    }
}

#[test]
fn parse_bookworm_typescript() {
    let cfg = load_example("bookworm", "typescript", "hello-world");
    assert_eq!(cfg.package.package_name, "hello-world-typescript");
    // TypeScript examples use language_env = "javascript" in TOML
    match &cfg.source {
        SourceKind::Tarball { language, .. } => {
            assert!(matches!(language, LanguageEnv::JavaScript(_)));
        }
        _ => panic!("Expected Tarball source"),
    }
}

#[test]
fn parse_bookworm_dotnet() {
    let cfg = load_example("bookworm", "dotnet", "hello-world");
    assert_eq!(cfg.package.package_name, "hello-world-dotnet");
    match &cfg.source {
        SourceKind::Tarball { language, .. } => {
            match language {
                LanguageEnv::Dotnet(dc) => {
                    assert!(dc.use_backup_version);
                    assert!(!dc.dotnet_packages.is_empty());
                    assert!(dc.dotnet_packages.len() >= 10);
                }
                _ => panic!("Expected Dotnet language"),
            }
        }
        _ => panic!("Expected Tarball source"),
    }
}

#[test]
fn parse_bookworm_nim() {
    let cfg = load_example("bookworm", "nim", "hello-world");
    assert_eq!(cfg.package.package_name, "hello-world-nim");
    match &cfg.source {
        SourceKind::Tarball { language, .. } => {
            match language {
                LanguageEnv::Nim(nc) => {
                    assert_eq!(nc.nim_version, "2.0.2");
                    assert!(nc.nim_binary_url.contains("nim-lang.org"));
                }
                _ => panic!("Expected Nim language"),
            }
        }
        _ => panic!("Expected Tarball source"),
    }
}

#[test]
fn parse_bookworm_python() {
    let cfg = load_example("bookworm", "python", "hello-world");
    assert_eq!(cfg.package.package_name, "hello-world-python");
    match &cfg.source {
        SourceKind::Tarball { language, .. } => {
            assert!(matches!(language, LanguageEnv::Python));
        }
        _ => panic!("Expected Tarball source"),
    }
}

#[test]
fn parse_bookworm_virtual() {
    let cfg = load_example("bookworm", "virtual", "hello-world");
    assert_eq!(cfg.package.package_name, "test-virtual-package");
    assert!(matches!(cfg.source, SourceKind::Virtual));
    assert!(!cfg.build_env.testing.run_autopkgtest);
}

#[test]
fn parse_bookworm_git_nimbus() {
    let cfg = load_example("bookworm", "git-package", "nimbus");
    assert_eq!(cfg.package.package_name, "hello-world");
    match &cfg.source {
        SourceKind::Git {
            url,
            tag,
            submodules,
            language,
        } => {
            assert!(url.contains("nimbus-eth2"));
            assert_eq!(tag, "v24.3.0");
            assert!(!submodules.is_empty());
            // Nimbus has many submodules
            assert!(submodules.len() > 40);
            match language {
                LanguageEnv::Nim(nc) => {
                    assert_eq!(nc.nim_version, "2.0.2");
                }
                _ => panic!("Expected Nim language"),
            }
        }
        _ => panic!("Expected Git source"),
    }
    // This example has testing disabled
    assert!(!cfg.build_env.testing.run_lintian);
    assert!(!cfg.build_env.testing.run_piuparts);
    assert!(!cfg.build_env.testing.run_autopkgtest);
}

// ─── Noble examples ───

#[test]
fn parse_noble_go() {
    let cfg = load_example("noble", "go", "hello-world");
    assert!(matches!(
        cfg.build_env.distribution,
        Distribution::Ubuntu(_)
    ));
    assert_eq!(cfg.build_env.distribution.as_short(), "noble");
}

#[test]
fn parse_noble_rust() {
    let cfg = load_example("noble", "rust", "hello-world");
    assert!(matches!(
        cfg.build_env.distribution,
        Distribution::Ubuntu(_)
    ));
}

#[test]
fn parse_noble_dotnet9() {
    let cfg = load_example("noble", "dotnet-9", "hello-world");
    assert_eq!(cfg.package.package_name, "hello-world-dotnet");
    assert!(matches!(
        cfg.build_env.distribution,
        Distribution::Ubuntu(_)
    ));
    match &cfg.source {
        SourceKind::Tarball { language, .. } => {
            match language {
                LanguageEnv::Dotnet(dc) => {
                    assert!(dc.use_backup_version);
                    // Noble dotnet-9 has deps field
                    assert!(dc.deps.is_some());
                    let deps = dc.deps.as_ref().unwrap();
                    assert!(deps.contains(&"libbrotli1".to_string()));
                }
                _ => panic!("Expected Dotnet language"),
            }
        }
        _ => panic!("Expected Tarball source"),
    }
}

#[test]
fn parse_noble_virtual() {
    let cfg = load_example("noble", "virtual", "hello-world");
    assert!(matches!(cfg.source, SourceKind::Virtual));
}

#[test]
fn parse_noble_nim() {
    let cfg = load_example("noble", "nim", "hello-world");
    match &cfg.source {
        SourceKind::Tarball { language, .. } => {
            assert!(matches!(language, LanguageEnv::Nim(_)));
        }
        _ => panic!("Expected Tarball source"),
    }
}

#[test]
fn parse_noble_java() {
    let cfg = load_example("noble", "java", "hello-world");
    match &cfg.source {
        SourceKind::Tarball { language, .. } => {
            assert!(matches!(language, LanguageEnv::Java(_)));
        }
        _ => panic!("Expected Tarball source"),
    }
}

#[test]
fn parse_noble_javascript() {
    let cfg = load_example("noble", "javascript", "hello-world");
    match &cfg.source {
        SourceKind::Tarball { language, .. } => {
            assert!(matches!(language, LanguageEnv::JavaScript(_)));
        }
        _ => panic!("Expected Tarball source"),
    }
}

#[test]
fn parse_noble_git_nimbus() {
    let cfg = load_example("noble", "git-package", "nimbus");
    assert!(matches!(cfg.source, SourceKind::Git { .. }));
}

// ─── Jammy examples ───

#[test]
fn parse_jammy_c() {
    let cfg = load_example("jammy", "c", "hello-world");
    assert_eq!(cfg.build_env.distribution.as_short(), "jammy");
}

#[test]
fn parse_jammy_rust() {
    let cfg = load_example("jammy", "rust", "hello-world");
    assert_eq!(cfg.build_env.distribution.as_short(), "jammy");
}

#[test]
fn parse_jammy_go() {
    let cfg = load_example("jammy", "go", "hello-world");
    assert_eq!(cfg.build_env.distribution.as_short(), "jammy");
}

#[test]
fn parse_jammy_dotnet() {
    let cfg = load_example("jammy", "dotnet", "hello-world");
    assert_eq!(cfg.build_env.distribution.as_short(), "jammy");
}

#[test]
fn parse_jammy_virtual() {
    let cfg = load_example("jammy", "virtual", "hello-world");
    assert!(matches!(cfg.source, SourceKind::Virtual));
}

#[test]
fn parse_jammy_git_nimbus() {
    let cfg = load_example("jammy", "git-package", "nimbus");
    assert!(matches!(cfg.source, SourceKind::Git { .. }));
}

// ─── Verify configs ───

#[test]
fn parse_verify_bookworm_c() {
    let cfg = load_verify("bookworm", "c", "hello-world");
    assert!(!cfg.verify.package_hash.is_empty());
    assert_eq!(cfg.verify.package_hash.len(), 2);
    // Check hash format (SHA-1, 40 hex chars)
    for ph in &cfg.verify.package_hash {
        assert_eq!(ph.hash.len(), 40, "SHA-1 hash should be 40 chars: {}", ph.name);
    }
}

#[test]
fn parse_verify_bookworm_rust() {
    let cfg = load_verify("bookworm", "rust", "hello-world");
    assert_eq!(cfg.verify.package_hash.len(), 4);
    // Verify all expected file types are present
    let names: Vec<&str> = cfg.verify.package_hash.iter().map(|h| h.name.as_str()).collect();
    assert!(names.iter().any(|n| n.ends_with(".dsc")));
    assert!(names.iter().any(|n| n.ends_with(".orig.tar.gz")));
    assert!(names.iter().any(|n| n.ends_with(".debian.tar.xz")));
    assert!(names.iter().any(|n| n.ends_with(".deb")));
}

// ─── Parse ALL examples (comprehensive sweep) ───

#[test]
fn parse_all_pkg_builder_configs() {
    let examples = examples_dir();
    let mut count = 0;
    let mut failures = Vec::new();

    for distro_entry in std::fs::read_dir(examples).unwrap() {
        let distro_entry = distro_entry.unwrap();
        if !distro_entry.path().is_dir() {
            continue;
        }
        for lang_entry in std::fs::read_dir(distro_entry.path()).unwrap() {
            let lang_entry = lang_entry.unwrap();
            if !lang_entry.path().is_dir() {
                continue;
            }
            for pkg_entry in std::fs::read_dir(lang_entry.path()).unwrap() {
                let pkg_entry = pkg_entry.unwrap();
                let pkg_dir = pkg_entry.path();
                if !pkg_dir.is_dir() {
                    continue;
                }
                let config_file = pkg_dir.join("pkg-builder.toml");
                if config_file.exists() {
                    match PkgConfig::load(&pkg_dir) {
                        Ok(_) => count += 1,
                        Err(e) => {
                            failures.push(format!("{}: {}", pkg_dir.display(), e));
                        }
                    }
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "Failed to parse {} config(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
    // Ensure we actually found configs
    assert!(count >= 30, "Expected at least 30 example configs, found {}", count);
}

#[test]
fn parse_all_verify_configs() {
    let examples = examples_dir();
    let mut count = 0;
    let mut failures = Vec::new();

    for distro_entry in std::fs::read_dir(examples).unwrap() {
        let distro_entry = distro_entry.unwrap();
        if !distro_entry.path().is_dir() {
            continue;
        }
        for lang_entry in std::fs::read_dir(distro_entry.path()).unwrap() {
            let lang_entry = lang_entry.unwrap();
            if !lang_entry.path().is_dir() {
                continue;
            }
            for pkg_entry in std::fs::read_dir(lang_entry.path()).unwrap() {
                let pkg_entry = pkg_entry.unwrap();
                let pkg_dir = pkg_entry.path();
                if !pkg_dir.is_dir() {
                    continue;
                }
                let verify_file = pkg_dir.join("pkg-builder-verify.toml");
                if verify_file.exists() {
                    match PkgVerifyConfig::load(&verify_file) {
                        Ok(_) => count += 1,
                        Err(e) => {
                            failures.push(format!("{}: {}", verify_file.display(), e));
                        }
                    }
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "Failed to parse {} verify config(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
    assert!(count >= 30, "Expected at least 30 verify configs, found {}", count);
}
