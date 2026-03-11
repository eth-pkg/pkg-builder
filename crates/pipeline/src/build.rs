use log::info;

use crate::context::Workspace;
use crate::PipelineError;

/// Full build pipeline: generate Makefile → make all.
pub fn build_package(ws: &Workspace) -> Result<(), PipelineError> {
    // 1. Generate Makefile
    let makefile_content =
        makefile::generate(&ws.config).map_err(|e| PipelineError::Phase {
            phase: "generate",
            message: format!("Makefile generation failed: {}", e),
        })?;

    // 2. Write Makefile to config root
    let makefile_path = ws.config.config_root.join("Makefile");
    std::fs::write(&makefile_path, &makefile_content)?;
    info!("Generated Makefile at {:?}", makefile_path);

    // 3. Run make all
    info!("Running make all...");
    let status = std::process::Command::new("make")
        .arg("all")
        .current_dir(&ws.config.config_root)
        .status()
        .map_err(|e| PipelineError::Phase {
            phase: "build",
            message: format!("Failed to run make: {}", e),
        })?;

    if !status.success() {
        return Err(PipelineError::Phase {
            phase: "build",
            message: format!("make all failed with exit code: {:?}", status.code()),
        });
    }

    Ok(())
}
