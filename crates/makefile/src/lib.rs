pub mod builder;
pub mod ir;
pub mod renderer;
pub mod variables;

use std::path::Path;
use std::process::Command;

use config::PkgConfig;

#[derive(Debug, Error)]
pub enum GeneratorError {
    #[error("Runtime profile not found: {0}")]
    ProfileNotFound(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RunError {
    #[error("make {target} failed with exit code {code}")]
    MakeFailed { target: String, code: i32 },
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Run a make target in the given working directory.
///
/// If `install_deps` is true, passes `INSTALL_DEPS=1` to make.
pub fn run_make(target: &str, working_dir: &Path, install_deps: bool) -> Result<(), RunError> {
    let mut cmd = Command::new("make");
    cmd.arg(target).current_dir(working_dir);
    if install_deps {
        cmd.arg("INSTALL_DEPS=1");
    }

    let status = cmd.status()?;

    if !status.success() {
        return Err(RunError::MakeFailed {
            target: target.to_string(),
            code: status.code().unwrap_or(1),
        });
    }

    Ok(())
}

/// Generate a Makefile from a PkgConfig.
pub fn generate(config: &PkgConfig) -> Result<String, GeneratorError> {
    let runtime_mk = if let Some(ref rt) = config.runtime {
        Some(load_runtime_mk(&rt.profile, &config.config_root)?)
    } else {
        None
    };

    let plan = builder::PlanBuilder::new(config, runtime_mk).build();
    Ok(renderer::render_makefile(&plan))
}

/// Validate that a profile name contains only safe characters.
fn validate_profile_name(name: &str) -> Result<(), GeneratorError> {
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(GeneratorError::ProfileNotFound(format!(
            "Invalid profile name: '{}' (must be alphanumeric, hyphens, or underscores)",
            name
        )));
    }
    Ok(())
}

/// Load a runtime .mk file. Checks local directory first, then built-in.
fn load_runtime_mk(name: &str, config_root: &Path) -> Result<String, GeneratorError> {
    validate_profile_name(name)?;

    // Check local override first
    let local_path = config_root.join("runtimes").join(format!("{}.mk", name));
    if local_path.exists() {
        return std::fs::read_to_string(&local_path).map_err(GeneratorError::Io);
    }

    match load_builtin_runtime_mk(name) {
        Some(content) => Ok(content.to_string()),
        None => Err(GeneratorError::ProfileNotFound(format!(
            "runtimes/{}.mk",
            name
        ))),
    }
}

fn load_builtin_runtime_mk(name: &str) -> Option<&'static str> {
    match name {
        "go" => Some(include_str!("../../../runtimes/go.mk")),
        "rust" => Some(include_str!("../../../runtimes/rust.mk")),
        "node" => Some(include_str!("../../../runtimes/node.mk")),
        "java" => Some(include_str!("../../../runtimes/java.mk")),
        "java-gradle" => Some(include_str!("../../../runtimes/java-gradle.mk")),
        "nim" => Some(include_str!("../../../runtimes/nim.mk")),
        "dotnet-noble" => Some(include_str!("../../../runtimes/dotnet-noble.mk")),
        "dotnet-debian" => Some(include_str!("../../../runtimes/dotnet-debian.mk")),
        "dotnet-backup" => Some(include_str!("../../../runtimes/dotnet-backup.mk")),
        "c" => Some(include_str!("../../../runtimes/c.mk")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_runtime_mk_exist() {
        assert!(load_builtin_runtime_mk("go").is_some());
        assert!(load_builtin_runtime_mk("c").is_some());
        assert!(load_builtin_runtime_mk("nonexistent").is_none());
    }
}
