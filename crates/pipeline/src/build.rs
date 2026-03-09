use config::source::SourceKind;
use log::info;

use crate::context::Workspace;
use crate::{prepare, source, PipelineError};

/// Full build pipeline: source → prepare → sbuild.
pub fn build_package(ws: &Workspace) -> Result<(), PipelineError> {
    // 1. Set up build directory
    prepare::setup_build_dir(ws)?;

    // 2. Acquire source
    source::acquire_source(ws)?;

    // 3. Extract, debcrafter, patch
    prepare::extract_source(ws)?;
    prepare::setup_debian_dir(ws)?;
    prepare::patch_source(ws)?;
    prepare::setup_sbuildrc()?;

    // 4. Build with sbuild
    let chroot_cmds = build_chroot_commands(ws);

    info!("Running sbuild...");
    tool::sbuild::Sbuild {
        distribution: &ws.config.build_env.distribution,
        cache_file: &ws.cache_file,
        build_dir: &ws.build_files_dir,
        chroot_setup_commands: &chroot_cmds,
        lintian: ws.config.build_env.testing.run_lintian,
    }
    .run()?;

    // 5. Optional post-build tests
    if ws.config.build_env.testing.run_piuparts {
        crate::test::run_piuparts(ws)?;
    }
    if ws.config.build_env.testing.run_autopkgtest {
        crate::test::run_autopkgtest(ws)?;
    }

    Ok(())
}

/// Build the chroot setup commands from runtime + distribution extras.
fn build_chroot_commands(ws: &Workspace) -> Vec<String> {
    let lang = match &ws.config.source {
        SourceKind::Tarball { language, .. } | SourceKind::Git { language, .. } => Some(language),
        SourceKind::Virtual => None,
    };

    let mut commands: Vec<String> = if let Some(lang) = lang {
        let rt = runtime::runtime_for(lang);
        rt.build_deps(&ws.config.build_env.arch, &ws.config.build_env.distribution)
    } else {
        vec![]
    };

    // Add distribution-specific commands (Noble needs universe/restricted/multiverse)
    commands.extend(ws.config.build_env.distribution.extra_chroot_commands());

    // Snapshot archive workarounds
    if ws.config.build_env.uses_snapshot() {
        // Disable Valid-Until checking for expired snapshot archives
        commands.insert(
            0,
            r#"echo 'Acquire::Check-Valid-Until "false";' > /etc/apt/apt.conf.d/99snapshot"#
                .to_string(),
        );

        // Add security snapshot repo if configured
        if let Some(security_url) = ws.config.build_env.security_repo_url() {
            let codename = ws.config.build_env.distribution.as_short();
            commands.push(format!(
                "echo 'deb {} {}-security main' > /etc/apt/sources.list.d/security-snapshot.list",
                security_url, codename
            ));
            commands.push("apt-get update".to_string());
        }
    }

    // Format as sbuild --chroot-setup-commands args
    commands
        .into_iter()
        .map(|cmd| format!("--chroot-setup-commands={}", cmd))
        .collect()
}
