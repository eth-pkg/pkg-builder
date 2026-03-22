use crate::commands::UpdateCommand;
use log::info;
use pkg_builder_update::{run_update, UpdateConfig};
use std::path::Path;

pub fn run(cmd: &UpdateCommand, config_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = Path::new(config_path);
    let config_dir = if config_dir.is_file() {
        config_dir.parent().unwrap_or(Path::new("."))
    } else {
        config_dir
    };

    let update_cfg = UpdateConfig {
        version: cmd.version.clone(),
        revision: cmd.revision.clone().unwrap_or_else(|| "1".to_string()),
        changelog_msg: cmd.changelog_msg.clone(),
        github_repo: cmd.github_repo.clone(),
        in_place: cmd.in_place,
        output: cmd.output.clone(),
        hash: cmd.hash.clone(),
        git_commit: cmd.git_commit.clone(),
        skip_download: cmd.skip_download,
        update_runtime: cmd.update_runtime,
        skip_patch_check: cmd.skip_patch_check,
    };

    let summary = run_update(config_dir, &update_cfg)?;

    println!();
    println!(
        "Updated {} -> {} (revision {})",
        summary.old_version, summary.new_version, summary.new_revision
    );

    if let Some(ref hash) = summary.source_hash {
        println!("Source hash: {}", hash);
    }
    if let Some(ref commit) = summary.git_commit {
        println!("Git commit: {}", commit);
    }
    if summary.runtime_updated {
        println!("Runtime: updated");
    }

    if !summary.files_modified.is_empty() {
        println!("Files updated:");
        for f in &summary.files_modified {
            println!("  {}", f);
        }
    }

    if let Some(ref patch_summary) = summary.patch_summary {
        println!("Patches: {}", patch_summary);
    }

    info!("Output directory: {}", summary.output_dir.display());

    Ok(())
}
