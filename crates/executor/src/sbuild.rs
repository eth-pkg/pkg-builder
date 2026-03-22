use std::fs;
use std::process::Command;

use config::PkgConfig;
use log::info;

use crate::chroot_setup::ChrootSetup;
use crate::paths::BuildPaths;
use crate::sbuild_conf;
use crate::ExecutorError;

/// Step 4: Generate sbuild.conf and invoke sbuild.
pub fn run_sbuild(config: &PkgConfig, paths: &BuildPaths) -> Result<(), ExecutorError> {
    let chroot_setup = ChrootSetup::from_config(config)?;
    let conf_content = sbuild_conf::render_sbuild_conf(config, &chroot_setup);

    let conf_path = paths.config_root.join("sbuild.conf");
    fs::write(&conf_path, &conf_content)?;
    info!("Generated sbuild.conf at {:?}", conf_path);

    let status = Command::new("sbuild")
        .arg("-c")
        .arg(&paths.chroot_tarball)
        .arg(&paths.src_dir)
        .env("SBUILD_CONFIG", &conf_path)
        .status()?;

    if !status.success() {
        return Err(ExecutorError::CommandFailed {
            command: "sbuild".to_string(),
            code: status.code().unwrap_or(1),
        });
    }

    // Touch .built marker
    let built_marker = paths.out_dir.join(".built");
    fs::write(&built_marker, "")?;

    Ok(())
}
