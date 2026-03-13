//! Integration tests that parse every example TOML config file.

use config::build_env::{Architecture, Distribution};
use config::source::SourceKind;
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

// ─── Bookworm examples ───

#[test]
fn parse_bookworm_c() {
    let cfg = load_example("bookworm", "c", "hello-world");
    assert_eq!(cfg.package.name, "hello-world-c");
    assert_eq!(cfg.package.version, "1.0.0");
    assert_eq!(cfg.package.revision, "1");
    assert!(matches!(cfg.build_env.distribution, Distribution::Debian(_)));
    assert!(matches!(cfg.build_env.arch, Architecture::Amd64));
    match &cfg.source {
        SourceKind::Tarball { hash, .. } => {
            assert!(hash.is_some());
        }
        _ => panic!("Expected Tarball source"),
    }
    assert!(cfg.runtime.is_none()); // C has no runtime
    assert!(cfg.build_env.testing.run_lintian);
    assert!(cfg.build_env.testing.run_piuparts);
    assert!(cfg.build_env.testing.run_autopkgtest);
}

#[test]
fn parse_bookworm_rust() {
    let cfg = load_example("bookworm", "rust", "hello-world");
    assert_eq!(cfg.package.name, "hello-world-rust");
    let rt = cfg.runtime.as_ref().expect("Expected runtime");
    assert_eq!(rt.recipe, "rust");
    assert!(rt.vars.contains_key("binary_url"));
    assert!(rt.vars.contains_key("binary_gpg_asc"));
}

#[test]
fn parse_bookworm_go() {
    let cfg = load_example("bookworm", "go", "hello-world");
    assert_eq!(cfg.package.name, "hello-world-go");
    let rt = cfg.runtime.as_ref().expect("Expected runtime");
    assert_eq!(rt.recipe, "go");
    let url = rt.vars.get("binary_url").unwrap().as_str().unwrap();
    assert!(url.contains("go.dev"));
}

#[test]
fn parse_bookworm_java() {
    let cfg = load_example("bookworm", "java", "hello-world");
    assert_eq!(cfg.package.name, "hello-world-java");
    let rt = cfg.runtime.as_ref().expect("Expected runtime");
    assert_eq!(rt.recipe, "java");
    assert!(rt.vars.contains_key("binary_url"));
}

#[test]
fn parse_bookworm_java_gradle() {
    let cfg = load_example("bookworm", "java-gradle", "hello-world");
    assert_eq!(cfg.package.name, "hello-world-java-gradle");
    let rt = cfg.runtime.as_ref().expect("Expected runtime");
    assert_eq!(rt.recipe, "java-gradle");
    assert!(rt.vars.contains_key("gradle_binary_url"));
}

#[test]
fn parse_bookworm_javascript() {
    let cfg = load_example("bookworm", "javascript", "hello-world");
    assert_eq!(cfg.package.name, "hello-world-javascript");
    let rt = cfg.runtime.as_ref().expect("Expected runtime");
    assert_eq!(rt.recipe, "node");
    assert!(rt.vars.contains_key("binary_url"));
    assert!(rt.vars.contains_key("yarn_version"));
}

#[test]
fn parse_bookworm_typescript() {
    let cfg = load_example("bookworm", "typescript", "hello-world");
    assert_eq!(cfg.package.name, "hello-world-typescript");
    let rt = cfg.runtime.as_ref().expect("Expected runtime");
    assert_eq!(rt.recipe, "node");
}

#[test]
fn parse_bookworm_dotnet() {
    let cfg = load_example("bookworm", "dotnet", "hello-world");
    assert_eq!(cfg.package.name, "hello-world-dotnet");
    let rt = cfg.runtime.as_ref().expect("Expected runtime");
    assert_eq!(rt.recipe, "dotnet-backup");
    let pkgs = rt.vars.get("packages").unwrap().as_array().unwrap();
    assert!(pkgs.len() >= 10);
}

#[test]
fn parse_bookworm_nim() {
    let cfg = load_example("bookworm", "nim", "hello-world");
    assert_eq!(cfg.package.name, "hello-world-nim");
    let rt = cfg.runtime.as_ref().expect("Expected runtime");
    assert_eq!(rt.recipe, "nim");
    let url = rt.vars.get("binary_url").unwrap().as_str().unwrap();
    assert!(url.contains("nim-lang.org"));
}

#[test]
fn parse_bookworm_virtual() {
    let cfg = load_example("bookworm", "virtual", "hello-world");
    assert_eq!(cfg.package.name, "test-virtual-package");
    assert!(matches!(cfg.source, SourceKind::Virtual));
    assert!(cfg.runtime.is_none());
    assert!(!cfg.build_env.testing.run_autopkgtest);
}

#[test]
fn parse_bookworm_git_nimbus() {
    let cfg = load_example("bookworm", "git-package", "nimbus");
    assert_eq!(cfg.package.name, "hello-world-git-nim");
    match &cfg.source {
        SourceKind::Git {
            url,
            tag,
            submodules,
        } => {
            assert!(url.contains("nimbus-eth2"));
            assert_eq!(tag, "v24.3.0");
            assert!(!submodules.is_empty());
            assert!(submodules.len() > 40);
        }
        _ => panic!("Expected Git source"),
    }
    let rt = cfg.runtime.as_ref().expect("Expected runtime");
    assert_eq!(rt.recipe, "nim");
    assert!(!cfg.build_env.testing.run_lintian);
}

// ─── Trixie examples ───

#[test]
fn parse_trixie_c() {
    let cfg = load_example("trixie", "c", "hello-world");
    assert!(matches!(cfg.build_env.distribution, Distribution::Debian(_)));
    assert_eq!(cfg.build_env.distribution.as_short(), "trixie");
    assert!(cfg.runtime.is_none());
}

#[test]
fn parse_trixie_rust() {
    let cfg = load_example("trixie", "rust", "hello-world");
    assert_eq!(cfg.build_env.distribution.as_short(), "trixie");
    assert_eq!(cfg.runtime.as_ref().unwrap().recipe, "rust");
}

#[test]
fn parse_trixie_go() {
    let cfg = load_example("trixie", "go", "hello-world");
    assert_eq!(cfg.build_env.distribution.as_short(), "trixie");
    assert_eq!(cfg.runtime.as_ref().unwrap().recipe, "go");
}

#[test]
fn parse_trixie_java() {
    let cfg = load_example("trixie", "java", "hello-world");
    assert_eq!(cfg.runtime.as_ref().unwrap().recipe, "java");
}

#[test]
fn parse_trixie_java_gradle() {
    let cfg = load_example("trixie", "java-gradle", "hello-world");
    assert_eq!(cfg.runtime.as_ref().unwrap().recipe, "java-gradle");
}

#[test]
fn parse_trixie_javascript() {
    let cfg = load_example("trixie", "javascript", "hello-world");
    assert_eq!(cfg.runtime.as_ref().unwrap().recipe, "node");
}

#[test]
fn parse_trixie_typescript() {
    let cfg = load_example("trixie", "typescript", "hello-world");
    assert_eq!(cfg.runtime.as_ref().unwrap().recipe, "node");
}

#[test]
fn parse_trixie_dotnet() {
    let cfg = load_example("trixie", "dotnet", "hello-world");
    assert!(cfg.runtime.as_ref().unwrap().recipe.starts_with("dotnet"));
}

#[test]
fn parse_trixie_nim() {
    let cfg = load_example("trixie", "nim", "hello-world");
    assert_eq!(cfg.runtime.as_ref().unwrap().recipe, "nim");
}

#[test]
fn parse_trixie_virtual() {
    let cfg = load_example("trixie", "virtual", "hello-world");
    assert!(matches!(cfg.source, SourceKind::Virtual));
}

#[test]
fn parse_trixie_git_nimbus() {
    let cfg = load_example("trixie", "git-package", "nimbus");
    assert!(matches!(cfg.source, SourceKind::Git { .. }));
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
    assert_eq!(cfg.runtime.as_ref().unwrap().recipe, "rust");
}

#[test]
fn parse_noble_dotnet9() {
    let cfg = load_example("noble", "dotnet-9", "hello-world");
    assert_eq!(cfg.package.name, "hello-world-dotnet");
    let rt = cfg.runtime.as_ref().expect("Expected runtime");
    assert_eq!(rt.recipe, "dotnet-backup");
    // Noble dotnet-9 has deps field
    assert!(rt.vars.contains_key("deps"));
}

#[test]
fn parse_noble_virtual() {
    let cfg = load_example("noble", "virtual", "hello-world");
    assert!(matches!(cfg.source, SourceKind::Virtual));
}

#[test]
fn parse_noble_nim() {
    let cfg = load_example("noble", "nim", "hello-world");
    assert_eq!(cfg.runtime.as_ref().unwrap().recipe, "nim");
}

#[test]
fn parse_noble_java() {
    let cfg = load_example("noble", "java", "hello-world");
    assert_eq!(cfg.runtime.as_ref().unwrap().recipe, "java");
}

#[test]
fn parse_noble_javascript() {
    let cfg = load_example("noble", "javascript", "hello-world");
    assert_eq!(cfg.runtime.as_ref().unwrap().recipe, "node");
}

#[test]
fn parse_noble_git_nimbus() {
    let cfg = load_example("noble", "git-package", "nimbus");
    assert!(matches!(cfg.source, SourceKind::Git { .. }));
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
    assert!(count >= 34, "Expected at least 34 example configs, found {}", count);
}
