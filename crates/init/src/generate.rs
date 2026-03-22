use std::collections::BTreeMap;

use serde::Serialize;

use crate::{InitConfig, Runtime, SourceConfig};

// --- Serializable types matching the pkg-builder.toml schema ---

#[derive(Serialize)]
struct PkgBuilderToml {
    package: Package,
    source: Source,
    build: Build,
    #[serde(skip_serializing_if = "Option::is_none")]
    runtime: Option<RuntimeSection>,
    tools: Tools,
}

#[derive(Serialize)]
struct Package {
    name: String,
    version: String,
    revision: String,
    homepage: String,
    spec: String,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum Source {
    Tarball {
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        hash: Option<String>,
    },
    Git {
        url: String,
        tag: String,
        submodules: Vec<String>,
    },
    Virtual,
}

#[derive(Serialize)]
struct Build {
    distribution: String,
    arch: String,
    workdir: String,
}

#[derive(Serialize)]
struct RuntimeSection {
    profile: String,
    #[serde(flatten)]
    vars: BTreeMap<String, toml::Value>,
}

#[derive(Serialize)]
struct Tools {
    pkg_builder: String,
    debcrafter: String,
    sbuild: String,
}

/// Generate the contents of `pkg-builder.toml`.
pub fn generate_toml(config: &InitConfig) -> String {
    let source = match &config.source {
        SourceConfig::Virtual => Source::Virtual,
        SourceConfig::Git { url, tag } => Source::Git {
            url: url.clone(),
            tag: tag.clone(),
            submodules: vec![],
        },
        SourceConfig::Tarball { url, hash } => {
            let filename = url.rsplit('/').next().unwrap_or(url);
            Source::Tarball {
                url: filename.to_string(),
                hash: hash.clone(),
            }
        }
    };

    let runtime = if config.runtime != Runtime::None {
        let mut vars = BTreeMap::new();
        for (key, value) in &config.runtime_vars {
            vars.insert(key.clone(), toml::Value::String(value.clone()));
        }
        Some(RuntimeSection {
            profile: config.runtime.to_string(),
            vars,
        })
    } else {
        None
    };

    let toml_config = PkgBuilderToml {
        package: Package {
            name: config.name.clone(),
            version: config.version.clone(),
            revision: config.revision.clone(),
            homepage: config.homepage.clone(),
            spec: format!("{}.sss", config.name),
        },
        source,
        build: Build {
            distribution: config.distribution.to_string(),
            arch: config.arch.clone(),
            workdir: format!("~/.pkg-builder/packages/{}", config.distribution),
        },
        runtime,
        tools: Tools {
            pkg_builder: "0.3.1".into(),
            debcrafter: "8189263".into(),
            sbuild: "0.85.6".into(),
        },
    };

    toml::to_string_pretty(&toml_config).expect("failed to serialize config")
}

// --- Serializable types for debcrafter specs ---

#[derive(Serialize)]
struct SourceServiceSpec {
    name: String,
    maintainer: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    homepage: Option<String>,
    standards_version: String,
    section: String,
    variants: Vec<String>,
    build_depends: Vec<String>,
    packages: Vec<String>,
    skip_debug_symbols: bool,
}

#[derive(Serialize)]
struct PackageSpec {
    name: String,
    architecture: String,
    summary: String,
    conflicts: Vec<String>,
    recommends: Vec<String>,
    provides: Vec<String>,
    suggests: Vec<String>,
    depends: Vec<String>,
    add_files: Vec<String>,
    add_links: Vec<String>,
    add_manpages: Vec<String>,
    long_doc: String,
}

/// Generate the contents of `{name}.sss` (debcrafter source service spec).
pub fn generate_sss(config: &InitConfig) -> String {
    let homepage = if config.homepage.is_empty() {
        None
    } else {
        Some(config.homepage.clone())
    };
    let spec = SourceServiceSpec {
        name: config.name.clone(),
        maintainer: format!("{} <{}>", config.maintainer_name, config.maintainer_email),
        homepage,
        standards_version: "4.5.1".into(),
        section: config.section.clone(),
        variants: vec![],
        build_depends: vec![],
        packages: vec![config.name.clone()],
        skip_debug_symbols: true,
    };
    toml::to_string_pretty(&spec).expect("failed to serialize sss")
}

/// Generate the contents of `{name}.sps` (debcrafter package spec).
pub fn generate_sps(config: &InitConfig) -> String {
    let summary = if config.summary.is_empty() {
        &config.name
    } else {
        &config.summary
    };
    let spec = PackageSpec {
        name: config.name.clone(),
        architecture: "any".into(),
        summary: summary.clone().to_string(),
        conflicts: vec![],
        recommends: vec![],
        provides: vec![],
        suggests: vec![],
        depends: vec![],
        add_files: vec![],
        add_links: vec![],
        add_manpages: vec![],
        long_doc: format!(
            "{}\n Long Description:\n  TODO: Add a detailed description of the package.",
            summary
        ),
    };
    toml::to_string_pretty(&spec).expect("failed to serialize sps")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Distribution;

    fn test_config() -> InitConfig {
        InitConfig {
            name: "hello-world".into(),
            version: "1.0.0".into(),
            revision: "1".into(),
            homepage: "https://example.com".into(),
            maintainer_name: "Test".into(),
            maintainer_email: "test@example.com".into(),
            section: "net".into(),
            summary: "A test package".into(),
            source: SourceConfig::Tarball {
                url: "https://example.com/hello-1.0.0.tar.gz".into(),
                hash: Some("abc123".into()),
            },
            distribution: Distribution::Bookworm,
            arch: "amd64".into(),
            runtime: Runtime::None,
            runtime_vars: vec![],
        }
    }

    #[test]
    fn test_generate_toml_is_valid() {
        let toml_str = generate_toml(&test_config());
        let parsed: toml::Value = toml::from_str(&toml_str).expect("generated TOML should parse");
        assert_eq!(parsed["package"]["name"].as_str(), Some("hello-world"));
        assert_eq!(parsed["source"]["type"].as_str(), Some("tarball"));
        assert_eq!(parsed["tools"]["sbuild"].as_str(), Some("0.85.6"));
    }

    #[test]
    fn test_generate_toml_virtual() {
        let mut config = test_config();
        config.source = SourceConfig::Virtual;
        let toml_str = generate_toml(&config);
        let parsed: toml::Value = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed["source"]["type"].as_str(), Some("virtual"));
    }

    #[test]
    fn test_generate_toml_git() {
        let mut config = test_config();
        config.source = SourceConfig::Git {
            url: "https://github.com/example/repo.git".into(),
            tag: "v1.0.0".into(),
        };
        let toml_str = generate_toml(&config);
        let parsed: toml::Value = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed["source"]["type"].as_str(), Some("git"));
        assert_eq!(parsed["source"]["tag"].as_str(), Some("v1.0.0"));
    }

    #[test]
    fn test_generate_toml_with_runtime() {
        let mut config = test_config();
        config.runtime = Runtime::Go;
        config.runtime_vars = vec![
            ("binary_url".into(), "https://go.dev/dl/go1.22.tar.gz".into()),
            ("binary_checksum".into(), "abc123".into()),
        ];
        let toml_str = generate_toml(&config);
        let parsed: toml::Value = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed["runtime"]["profile"].as_str(), Some("go"));
        assert_eq!(
            parsed["runtime"]["binary_url"].as_str(),
            Some("https://go.dev/dl/go1.22.tar.gz")
        );
    }

    #[test]
    fn test_generate_toml_no_runtime_section_when_none() {
        let config = test_config();
        let toml_str = generate_toml(&config);
        let parsed: toml::Value = toml::from_str(&toml_str).unwrap();
        assert!(parsed.get("runtime").is_none());
    }
}
