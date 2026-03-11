pub mod emitter;
pub mod recipe;
pub mod variables;

use std::path::Path;

use config::PkgConfig;
use thiserror::Error;

use emitter::MakefileEmitter;
use recipe::RecipeParser;
use variables::VariableResolver;

#[derive(Debug, Error)]
pub enum GeneratorError {
    #[error("Recipe not found: {0}")]
    RecipeNotFound(String),
    #[error("Recipe parse error in {file} line {line}: {message}")]
    RecipeParse {
        file: String,
        line: usize,
        message: String,
    },
    #[error("Variable not found: {0}")]
    VariableNotFound(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Generate a Makefile from a PkgConfig.
pub fn generate(config: &PkgConfig) -> Result<String, GeneratorError> {
    let vars = VariableResolver::from_config(config);
    let pipeline_name = vars.get("pipeline").cloned().unwrap_or_default();
    let runtime_name = vars.get("runtime_recipe").cloned().unwrap_or_default();

    // Load pipeline recipe
    let pipeline_source = load_recipe("pipelines", &pipeline_name, &config.config_root)?;
    let pipeline_cmds = RecipeParser::parse(&pipeline_source, &pipeline_name, &vars)?;

    // Load runtime recipe (may be empty for c/python)
    let runtime_cmds = if !runtime_name.is_empty() {
        let runtime_source = load_recipe("runtimes", &runtime_name, &config.config_root)?;
        RecipeParser::parse(&runtime_source, &runtime_name, &vars)?
    } else {
        vec![]
    };

    // Emit Makefile
    let emitter = MakefileEmitter::new(config, &vars, &pipeline_cmds, &runtime_cmds);
    Ok(emitter.emit())
}

/// Load a recipe file. Checks local directory first, then built-in.
fn load_recipe(
    kind: &str,
    name: &str,
    config_root: &Path,
) -> Result<String, GeneratorError> {
    // Check local override: <config_root>/<kind>/<name>.recipe
    let local_path = config_root.join(kind).join(format!("{}.recipe", name));
    if local_path.exists() {
        return std::fs::read_to_string(&local_path).map_err(GeneratorError::Io);
    }

    // Check built-in recipes
    match load_builtin_recipe(kind, name) {
        Some(content) => Ok(content.to_string()),
        None => Err(GeneratorError::RecipeNotFound(format!(
            "{}/{}.recipe",
            kind, name
        ))),
    }
}

fn load_builtin_recipe(kind: &str, name: &str) -> Option<&'static str> {
    match (kind, name) {
        // Pipelines
        ("pipelines", "debian-bookworm") => Some(include_str!("../../../pipelines/debian-bookworm.recipe")),
        ("pipelines", "debian-trixie") => Some(include_str!("../../../pipelines/debian-trixie.recipe")),
        ("pipelines", "ubuntu-noble") => Some(include_str!("../../../pipelines/ubuntu-noble.recipe")),
        ("pipelines", "ubuntu-jammy") => Some(include_str!("../../../pipelines/ubuntu-jammy.recipe")),
        ("pipelines", "debian-bookworm-git") => Some(include_str!("../../../pipelines/debian-bookworm-git.recipe")),
        ("pipelines", "debian-trixie-git") => Some(include_str!("../../../pipelines/debian-trixie-git.recipe")),
        ("pipelines", "ubuntu-noble-git") => Some(include_str!("../../../pipelines/ubuntu-noble-git.recipe")),
        ("pipelines", "ubuntu-jammy-git") => Some(include_str!("../../../pipelines/ubuntu-jammy-git.recipe")),
        ("pipelines", "debian-bookworm-virtual") => Some(include_str!("../../../pipelines/debian-bookworm-virtual.recipe")),
        ("pipelines", "debian-trixie-virtual") => Some(include_str!("../../../pipelines/debian-trixie-virtual.recipe")),
        ("pipelines", "ubuntu-noble-virtual") => Some(include_str!("../../../pipelines/ubuntu-noble-virtual.recipe")),
        ("pipelines", "ubuntu-jammy-virtual") => Some(include_str!("../../../pipelines/ubuntu-jammy-virtual.recipe")),
        // Runtimes
        ("runtimes", "go") => Some(include_str!("../../../runtimes/go.recipe")),
        ("runtimes", "rust") => Some(include_str!("../../../runtimes/rust.recipe")),
        ("runtimes", "node") => Some(include_str!("../../../runtimes/node.recipe")),
        ("runtimes", "java") => Some(include_str!("../../../runtimes/java.recipe")),
        ("runtimes", "java-gradle") => Some(include_str!("../../../runtimes/java-gradle.recipe")),
        ("runtimes", "nim") => Some(include_str!("../../../runtimes/nim.recipe")),
        ("runtimes", "dotnet-noble") => Some(include_str!("../../../runtimes/dotnet-noble.recipe")),
        ("runtimes", "dotnet-debian") => Some(include_str!("../../../runtimes/dotnet-debian.recipe")),
        ("runtimes", "dotnet-backup") => Some(include_str!("../../../runtimes/dotnet-backup.recipe")),
        ("runtimes", "c") => Some(include_str!("../../../runtimes/c.recipe")),
        ("runtimes", "python") => Some(include_str!("../../../runtimes/python.recipe")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_recipes_exist() {
        assert!(load_builtin_recipe("pipelines", "debian-bookworm").is_some());
        assert!(load_builtin_recipe("runtimes", "go").is_some());
        assert!(load_builtin_recipe("runtimes", "c").is_some());
        assert!(load_builtin_recipe("runtimes", "nonexistent").is_none());
    }
}
