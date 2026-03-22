use std::process::Command;

use config::PkgConfig;

use crate::ExecutorError;

/// Check that all required external tools are available on PATH.
pub fn check_required_tools(config: &PkgConfig) -> Result<(), ExecutorError> {
    let mut tools: Vec<&str> = Vec::new();

    match &config.source {
        config::source::SourceKind::Tarball { .. } => {
            tools.push("tar");
        }
        config::source::SourceKind::Git { .. } => {
            tools.extend(["git", "tar"]);
        }
        config::source::SourceKind::Virtual => {
            tools.push("tar");
        }
    }
    tools.extend(["dpkg-parsechangelog", "sbuild"]);

    let mut missing = Vec::new();
    for tool in &tools {
        if !tool_exists(tool) {
            missing.push(tool.to_string());
        }
    }

    if !missing.is_empty() {
        return Err(ExecutorError::MissingTools(missing));
    }

    Ok(())
}

fn tool_exists(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
