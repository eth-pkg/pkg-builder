use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use config::PkgConfig;
use log::info;

use crate::paths::BuildPaths;
use crate::ExecutorError;

/// Step 3: Generate Debian packaging via debcrafter library + copy src/ overrides.
pub fn generate_debian(config: &PkgConfig, paths: &BuildPaths) -> Result<(), ExecutorError> {
    // Load the .sss spec file
    let mut spec: debcrafter::generate::SingleSource =
        debcrafter::input::load_toml(&paths.spec_file).map_err(|e| {
            ExecutorError::Debcrafter(format!("Failed to load spec {:?}: {}", paths.spec_file, e))
        })?;

    // Resolve maintainer: spec field → DEBEMAIL env var → error
    let maintainer = spec
        .maintainer
        .take()
        .or_else(|| std::env::var("DEBEMAIL").ok())
        .ok_or(ExecutorError::MissingMaintainer)?;

    info!("Running debcrafter on {:?}", paths.spec_file);

    let command = debcrafter::generate::Command::Generate {
        dest: &paths.out_dir,
        maintainer: &maintainer,
        homepage: spec.homepage.as_deref(),
        standards_version: &spec.standards_version,
    };

    let mut opts = debcrafter::generate::GlobalOptions { record_deps: None };

    debcrafter::generate::process_source(
        paths.spec_file.parent().unwrap_or(Path::new(".")),
        &spec.name,
        &mut spec.source,
        &command,
        &mut opts,
    );

    // Copy local src/ overrides if present
    let src_overrides = config.config_root.join("src");
    if src_overrides.is_dir() {
        info!("Copying src/ overrides from {:?}", src_overrides);
        copy_dir_contents(&src_overrides, &paths.src_dir)?;
    }

    // Ensure debian/rules is executable (src/ overrides may clobber permissions)
    let rules_path = paths.src_dir.join("debian/rules");
    if rules_path.exists() {
        fs::set_permissions(&rules_path, fs::Permissions::from_mode(0o755))?;
    }

    Ok(())
}

/// Recursively copy contents of `src` directory into `dest`.
fn copy_dir_contents(src: &Path, dest: &Path) -> Result<(), ExecutorError> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if src_path.is_dir() {
            fs::create_dir_all(&dest_path)?;
            copy_dir_contents(&src_path, &dest_path)?;
        } else {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&src_path, &dest_path)?;
        }
    }
    Ok(())
}
