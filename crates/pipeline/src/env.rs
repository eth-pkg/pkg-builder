use std::{env, fs};

use log::info;
use rand::random;

use crate::context::Workspace;
use crate::PipelineError;

/// Create the sbuild chroot environment.
pub fn create_env(ws: &Workspace) -> Result<(), PipelineError> {
    let temp_dir = env::temp_dir().join(format!("temp_{}", random::<u32>()));
    fs::create_dir(&temp_dir)?;

    // Ensure cache dir parent exists
    if let Some(parent) = ws.cache_file.parent() {
        fs::create_dir_all(parent)?;
    }

    let repo_url = ws.config.build_env.repo_url();

    tool::sbuild::SbuildCreateChroot {
        distribution: &ws.config.build_env.distribution,
        cache_file: &ws.cache_file,
        temp_dir: &temp_dir,
        repo_url: &repo_url,
        snapshot: ws.config.build_env.uses_snapshot(),
    }
    .run()?;

    Ok(())
}

/// Clean (remove) the cached sbuild chroot.
pub fn clean_env(ws: &Workspace) -> Result<(), PipelineError> {
    info!("Cleaning cached build: {:?}", ws.cache_file);
    if ws.cache_file.exists() {
        fs::remove_file(&ws.cache_file)?;
    }
    Ok(())
}
