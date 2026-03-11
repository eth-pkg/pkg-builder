use log::info;

use crate::context::Workspace;
use crate::PipelineError;

/// Run lintian on the built package.
pub fn run_lintian(ws: &Workspace) -> Result<(), PipelineError> {
    info!("Running lintian on {:?}", ws.changes_path);

    tool::lintian::Lintian {
        changes_file: &ws.changes_path,
        distribution: &ws.config.build_env.distribution,
    }
    .run()?;

    Ok(())
}

/// Run piuparts on the built package.
pub fn run_piuparts(ws: &Workspace) -> Result<(), PipelineError> {
    info!("Running piuparts on {:?}", ws.deb_path);

    let is_dotnet = ws
        .config
        .runtime
        .as_ref()
        .map(|r| r.recipe.starts_with("dotnet"))
        .unwrap_or(false);

    let repo_url = ws.config.build_env.repo_url();

    tool::piuparts::Piuparts {
        distribution: &ws.config.build_env.distribution,
        deb_file: &ws.deb_path,
        deb_dir: &ws.build_artifacts_dir,
        is_dotnet,
        repo_url: &repo_url,
    }
    .run()?;

    Ok(())
}

/// Run autopkgtest on the built package.
pub fn run_autopkgtest(ws: &Workspace) -> Result<(), PipelineError> {
    info!("Running autopkgtest on {:?}", ws.changes_path);

    // Ensure QEMU image exists
    let repo_url = ws.config.build_env.repo_url();
    let image_path = tool::autopkgtest::ensure_autopkgtest_image(
        &ws.config.build_env.sbuild_cache_dir,
        &ws.config.build_env.distribution,
        &ws.config.build_env.arch,
        &repo_url,
    )?;

    // With the new declarative approach, test setup is in the Makefile.
    // For now, pass no extra setup commands.
    let setup_commands: Vec<String> = vec![];

    tool::autopkgtest::Autopkgtest {
        changes_file: &ws.changes_path,
        image_path: &image_path,
        deb_dir: &ws.build_artifacts_dir,
        setup_commands: &setup_commands,
    }
    .run()?;

    Ok(())
}
