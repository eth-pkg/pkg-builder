pub mod builder;
pub mod ir;
pub mod renderer;
pub mod sbuild_conf;
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

/// Output of the generate step: Makefile + sbuild.conf.
pub struct GenerateOutput {
    pub makefile: String,
    pub sbuild_conf: String,
}

/// Generate a Makefile and sbuild.conf from a PkgConfig.
pub fn generate(config: &PkgConfig) -> Result<GenerateOutput, GeneratorError> {
    let runtime_perl = if let Some(ref rt) = config.runtime {
        Some(load_runtime_perl(&rt.profile, &config.config_root)?)
    } else {
        None
    };

    let plan = builder::PlanBuilder::new(config, runtime_perl).build();
    let makefile = renderer::render_makefile(&plan);
    let sbuild_conf = sbuild_conf::render_sbuild_conf(config, &plan.preamble);

    Ok(GenerateOutput {
        makefile,
        sbuild_conf,
    })
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

/// Load a runtime .perl template. Checks local directory first, then built-in.
fn load_runtime_perl(name: &str, config_root: &Path) -> Result<String, GeneratorError> {
    validate_profile_name(name)?;

    // Check local override first
    let local_path = config_root.join("runtimes").join(format!("{}.perl", name));
    if local_path.exists() {
        return std::fs::read_to_string(&local_path).map_err(GeneratorError::Io);
    }

    match load_builtin_runtime_perl(name) {
        Some(content) => Ok(content.to_string()),
        None => Err(GeneratorError::ProfileNotFound(format!(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_runtime_perl_exist() {
        assert!(load_builtin_runtime_perl("go").is_some());
        assert!(load_builtin_runtime_perl("c").is_some());
        assert!(load_builtin_runtime_perl("nonexistent").is_none());
    }
}
