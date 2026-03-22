use std::path::Path;

use config::PkgConfig;

use crate::ExecutorError;

/// Validate that a profile name contains only safe characters.
pub fn validate_profile_name(name: &str) -> Result<(), ExecutorError> {
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(ExecutorError::ProfileNotFound(format!(
            "Invalid profile name: '{}' (must be alphanumeric, hyphens, or underscores)",
            name
        )));
    }
    Ok(())
}

/// Load a runtime .perl template. Checks local directory first, then built-in.
pub fn load_runtime_perl(name: &str, config_root: &Path) -> Result<String, ExecutorError> {
    validate_profile_name(name)?;

    // Check local override first
    let local_path = config_root.join("runtimes").join(format!("{}.perl", name));
    if local_path.exists() {
        return std::fs::read_to_string(&local_path).map_err(ExecutorError::Io);
    }

    match load_builtin_runtime_perl(name) {
        Some(content) => Ok(content.to_string()),
        None => Err(ExecutorError::ProfileNotFound(format!(
            "runtimes/{}.perl",
            name
        ))),
    }
}

fn load_builtin_runtime_perl(name: &str) -> Option<&'static str> {
    match name {
        "go" => Some(include_str!("../../../runtimes/go.perl")),
        "rust" => Some(include_str!("../../../runtimes/rust.perl")),
        "node" => Some(include_str!("../../../runtimes/node.perl")),
        "java" => Some(include_str!("../../../runtimes/java.perl")),
        "java-gradle" => Some(include_str!("../../../runtimes/java-gradle.perl")),
        "nim" => Some(include_str!("../../../runtimes/nim.perl")),
        "dotnet-noble" => Some(include_str!("../../../runtimes/dotnet-noble.perl")),
        "dotnet-debian" => Some(include_str!("../../../runtimes/dotnet-debian.perl")),
        "dotnet-backup" => Some(include_str!("../../../runtimes/dotnet-backup.perl")),
        "c" => Some(include_str!("../../../runtimes/c.perl")),
        _ => None,
    }
}

/// Substitute `{{var_name}}` placeholders in a runtime .perl template with config values.
pub fn substitute_template(template: &str, config: &PkgConfig) -> String {
    let mut result = template.to_string();

    if let Some(ref runtime) = config.runtime {
        // Substitute scalar vars: {{key}} → value
        for (key, value) in &runtime.vars {
            if let toml::Value::String(s) = value {
                let escaped = perl_escape_single_quote(s);
                result = result.replace(&format!("{{{{{}}}}}", key), &escaped);
            }
        }

        // Substitute {{packages_perl}} for dotnet runtimes
        if result.contains("{{packages_perl}}") {
            let packages_perl = render_packages_perl(runtime, config);
            result = result.replace("{{packages_perl}}", &packages_perl);
        }
    }

    result
}

/// Escape a value for embedding in a Perl single-quoted string.
fn perl_escape_single_quote(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "\\'")
}

/// Render dotnet packages as Perl hashref entries for the `@packages` array.
fn render_packages_perl(
    runtime: &config::runtime::RuntimeConfig,
    config: &PkgConfig,
) -> String {
    let mut entries = Vec::new();

    if let Some(toml::Value::Array(pkgs)) = runtime.vars.get("packages") {
        for item in pkgs {
            if let toml::Value::Table(table) = item {
                let name = table
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let url = table
                    .get("url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let hash = table
                    .get("hash")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let mut entry = format!(
                    "    {{ name => '{}', url => '{}', hash => '{}'",
                    perl_escape_single_quote(name),
                    perl_escape_single_quote(url),
                    perl_escape_single_quote(hash),
                );

                if runtime.profile.starts_with("dotnet") {
                    let apt_name = transform_dotnet_name(name, &config.build_env.arch);
                    entry.push_str(&format!(
                        ", apt_name => '{}'",
                        perl_escape_single_quote(&apt_name)
                    ));
                }

                entry.push_str(" }");
                entries.push(entry);
            }
        }
    }

    entries.join(",\n") + ","
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_runtime_perl_exist() {
        assert!(load_builtin_runtime_perl("go").is_some());
        assert!(load_builtin_runtime_perl("c").is_some());
        assert!(load_builtin_runtime_perl("nonexistent").is_none());
    }

    #[test]
    fn test_perl_escape_single_quote() {
        assert_eq!(perl_escape_single_quote("hello"), "hello");
        assert_eq!(perl_escape_single_quote("it's"), "it\\'s");
        assert_eq!(perl_escape_single_quote("a\\b"), "a\\\\b");
    }
}
