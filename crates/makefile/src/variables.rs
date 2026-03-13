use std::collections::HashMap;

use config::PkgConfig;

/// Resolves {{variable}} placeholders in recipe templates.
pub struct VariableResolver {
    pub(crate) vars: HashMap<String, String>,
}

impl VariableResolver {
    pub fn from_config(config: &PkgConfig) -> Self {
        let mut vars = HashMap::new();

        // Package fields
        vars.insert("package_name".into(), config.package.name.clone());
        vars.insert("version".into(), config.package.version.clone());
        vars.insert("revision".into(), config.package.revision.clone());
        vars.insert("homepage".into(), config.package.homepage.clone());
        // Use Make variable reference so recipe commands get portable paths
        vars.insert("spec".into(), "$(PKG_SPEC)".into());

        // Build env
        vars.insert(
            "distribution".into(),
            config.build_env.distribution.as_short().to_string(),
        );
        vars.insert("arch".into(), config.build_env.arch.to_string());
        vars.insert(
            "build_dir".into(),
            format!(
                "$(WORK_DIR)/{}-{}-{}",
                config.package.name,
                config.package.version,
                config.package.revision
            ),
        );
        vars.insert(
            "chroot_dir".into(),
            config
                .build_env
                .chroot_dir
                .display()
                .to_string(),
        );

        // Repo URL
        vars.insert("repo_url".into(), config.build_env.repo_url());

        // Snapshot
        if let Some(ref date) = config.build_env.snapshot_date {
            vars.insert("snapshot_date".into(), date.clone());
        }
        if let Some(ref date) = config.build_env.snapshot_security_date {
            vars.insert("snapshot_security_date".into(), date.clone());
        }

        // Source
        match &config.source {
            config::source::SourceKind::Tarball { url, hash, .. } => {
                vars.insert("source_url".into(), url.clone());
                if let Some(h) = hash {
                    let algo = if h.len() == 128 { "sha512" } else { "sha256" };
                    vars.insert("source_hash".into(), h.clone());
                    vars.insert("source_hash_algo".into(), algo.into());
                }
            }
            config::source::SourceKind::Git { url, tag, .. } => {
                vars.insert("git_url".into(), url.clone());
                vars.insert("git_tag".into(), tag.clone());
            }
            config::source::SourceKind::Virtual => {
                vars.insert("source_type".into(), "virtual".into());
            }
        }

        // Runtime vars from [runtime] section
        if let Some(ref runtime) = config.runtime {
            insert_runtime_vars(&mut vars, runtime, config);
        }

        // Pipeline and runtime recipe names
        let (pipeline, runtime_recipe) = derive_recipe_names(config);
        vars.insert("pipeline".into(), pipeline);
        vars.insert("runtime_recipe".into(), runtime_recipe);

        // Tool versions
        vars.insert(
            "debcrafter_rev".into(),
            "$(DEBCRAFTER_REV)".into(),
        );
        vars.insert(
            "sbuild_version".into(),
            config.build_env.tool_versions.sbuild.clone(),
        );
        vars.insert(
            "lintian_version".into(),
            config.build_env.tool_versions.lintian.clone(),
        );
        vars.insert(
            "piuparts_version".into(),
            config.build_env.tool_versions.piuparts.clone(),
        );
        vars.insert(
            "autopkgtest_version".into(),
            config.build_env.tool_versions.autopkgtest.clone(),
        );
        vars.insert(
            "pkg_builder_version".into(),
            config.build_env.pkg_builder_version.clone(),
        );

        // Testing flags
        vars.insert(
            "run_lintian".into(),
            config.build_env.testing.run_lintian.to_string(),
        );
        vars.insert(
            "run_piuparts".into(),
            config.build_env.testing.run_piuparts.to_string(),
        );
        vars.insert(
            "run_autopkgtest".into(),
            config.build_env.testing.run_autopkgtest.to_string(),
        );

        Self { vars }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.vars.get(key)
    }

    /// Substitute all {{var}} placeholders in a string.
    /// Limits to 10 iterations to prevent infinite loops from circular references.
    pub fn substitute(&self, input: &str) -> String {
        let mut result = input.to_string();
        let max_iterations = 10;
        for _ in 0..max_iterations {
            let prev = result.clone();
            for (key, value) in &self.vars {
                result = result.replace(&format!("{{{{{}}}}}", key), value);
            }
            if result == prev {
                break;
            }
        }
        result
    }

    /// Get all variables matching a list prefix for REPEAT expansion.
    /// E.g., for "packages" returns vec of HashMaps for packages[0], packages[1], etc.
    pub fn get_list(&self, list_name: &str) -> Vec<HashMap<String, String>> {
        let mut items: HashMap<usize, HashMap<String, String>> = HashMap::new();

        let prefix = format!("{}.", list_name);
        for (key, value) in &self.vars {
            if let Some(rest) = key.strip_prefix(&prefix) {
                // rest is like "0.name", "0.url", etc.
                if let Some(dot_pos) = rest.find('.') {
                    if let Ok(idx) = rest[..dot_pos].parse::<usize>() {
                        let field = &rest[dot_pos + 1..];
                        items
                            .entry(idx)
                            .or_default()
                            .insert(field.to_string(), value.clone());
                    }
                }
            }
        }

        let mut indices: Vec<usize> = items.keys().copied().collect();
        indices.sort();
        indices.into_iter().filter_map(|i| items.remove(&i)).collect()
    }
}

/// Insert runtime variables from the RuntimeConfig's flat vars HashMap.
fn insert_runtime_vars(
    vars: &mut HashMap<String, String>,
    runtime: &config::runtime::RuntimeConfig,
    config: &PkgConfig,
) {
    for (key, value) in &runtime.vars {
        match value {
            toml::Value::String(s) => {
                vars.insert(key.clone(), s.clone());
            }
            toml::Value::Boolean(b) => {
                vars.insert(key.clone(), b.to_string());
            }
            toml::Value::Integer(n) => {
                vars.insert(key.clone(), n.to_string());
            }
            toml::Value::Float(f) => {
                vars.insert(key.clone(), f.to_string());
            }
            toml::Value::Array(arr) => {
                // Array of tables (e.g., packages) or array of strings (e.g., deps)
                for (i, item) in arr.iter().enumerate() {
                    match item {
                        toml::Value::Table(table) => {
                            for (field, val) in table {
                                if let toml::Value::String(s) = val {
                                    vars.insert(
                                        format!("{}.{}.{}", key, i, field),
                                        s.clone(),
                                    );
                                }
                            }
                        }
                        toml::Value::String(s) => {
                            vars.insert(format!("{}.{}.name", key, i), s.clone());
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    // Derive apt_name for dotnet packages
    if runtime.recipe.starts_with("dotnet") {
        if let Some(toml::Value::Array(pkgs)) = runtime.vars.get("packages") {
            for (i, item) in pkgs.iter().enumerate() {
                if let toml::Value::Table(table) = item {
                    if let Some(toml::Value::String(name)) = table.get("name") {
                        let apt_name =
                            transform_dotnet_name(name, &config.build_env.arch);
                        vars.insert(format!("packages.{}.apt_name", i), apt_name);
                    }
                }
            }
        }
    }
}

fn transform_dotnet_name(input: &str, arch: &config::build_env::Architecture) -> String {
    let arch_str = format!("_{}", arch);
    if let Some(pos) = input.find(&arch_str) {
        let trimmed = &input[..pos];
        trimmed.replace('_', "=")
    } else {
        input.replace('_', "=")
    }
}

/// Derive pipeline and runtime recipe names from config.
fn derive_recipe_names(config: &PkgConfig) -> (String, String) {
    let distro = config.build_env.distribution.as_short();
    let distro_family = if config.build_env.distribution.is_debian() {
        "debian"
    } else {
        "ubuntu"
    };

    let source_suffix = match &config.source {
        config::source::SourceKind::Tarball { .. } => "",
        config::source::SourceKind::Git { .. } => "-git",
        config::source::SourceKind::Virtual => "-virtual",
    };

    let pipeline = format!("{}-{}{}", distro_family, distro, source_suffix);

    let runtime = match &config.runtime {
        Some(rt) => rt.recipe.clone(),
        None => String::new(),
    };

    (pipeline, runtime)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitute_simple() {
        let mut vars = HashMap::new();
        vars.insert("name".into(), "hello".into());
        let resolver = VariableResolver { vars };
        assert_eq!(resolver.substitute("pkg-{{name}}"), "pkg-hello");
    }

    #[test]
    fn test_substitute_multiple() {
        let mut vars = HashMap::new();
        vars.insert("a".into(), "1".into());
        vars.insert("b".into(), "2".into());
        let resolver = VariableResolver { vars };
        assert_eq!(resolver.substitute("{{a}}-{{b}}"), "1-2");
    }

    #[test]
    fn test_substitute_missing_var_left_as_is() {
        let resolver = VariableResolver {
            vars: HashMap::new(),
        };
        assert_eq!(resolver.substitute("{{missing}}"), "{{missing}}");
    }

    #[test]
    fn test_get_list() {
        let mut vars = HashMap::new();
        vars.insert("packages.0.name".into(), "pkg-a".into());
        vars.insert("packages.0.url".into(), "http://a".into());
        vars.insert("packages.1.name".into(), "pkg-b".into());
        vars.insert("packages.1.url".into(), "http://b".into());
        let resolver = VariableResolver { vars };

        let list = resolver.get_list("packages");
        assert_eq!(list.len(), 2);
        assert_eq!(list[0]["name"], "pkg-a");
        assert_eq!(list[1]["name"], "pkg-b");
    }
}
