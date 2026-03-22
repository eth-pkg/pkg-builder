use std::fs;
use std::process::Command;

use config::PkgConfig;
use log::info;

use crate::paths::BuildPaths;
use crate::ExecutorError;

/// Create the sbuild chroot environment.
pub fn create_env(config: &PkgConfig, paths: &BuildPaths) -> Result<(), ExecutorError> {
    info!("Creating chroot at {:?}", paths.chroot_tarball);

    // Ensure chroot_dir exists
    if let Some(parent) = paths.chroot_tarball.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut cmd = Command::new("sbuild-createchroot");
    cmd.args(["--chroot-mode=unshare", "--make-sbuild-tarball"])
        .arg(&paths.chroot_tarball);

    if config.build_env.uses_snapshot() {
        cmd.arg("--debootstrapopts=--no-check-gpg");
    }

    cmd.arg(config.build_env.distribution.as_short())
        .arg("/tmp/sbuild-createchroot")
        .arg(config.build_env.repo_url());

    let status = cmd.status()?;
    if !status.success() {
        return Err(ExecutorError::CommandFailed {
            command: "sbuild-createchroot".to_string(),
            code: status.code().unwrap_or(1),
        });
    }

    Ok(())
}

/// Remove the chroot tarball.
pub fn clean_env(paths: &BuildPaths) -> Result<(), ExecutorError> {
    if paths.chroot_tarball.exists() {
        info!("Removing chroot tarball {:?}", paths.chroot_tarball);
        fs::remove_file(&paths.chroot_tarball)?;
    }
    Ok(())
}
