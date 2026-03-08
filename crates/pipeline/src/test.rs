use config::language::LanguageEnv;
use config::source::SourceKind;
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

    let is_dotnet = matches!(
        source_language(&ws.config.source),
        Some(LanguageEnv::Dotnet(_))
    );

    tool::piuparts::Piuparts {
        distribution: &ws.config.build_env.distribution,
        deb_file: &ws.deb_path,
        deb_dir: &ws.build_artifacts_dir,
        is_dotnet,
    }
    .run()?;

    Ok(())
}

/// Run autopkgtest on the built package.
pub fn run_autopkgtest(ws: &Workspace) -> Result<(), PipelineError> {
    info!("Running autopkgtest on {:?}", ws.changes_path);

    // Ensure QEMU image exists
    let image_path = tool::autopkgtest::ensure_autopkgtest_image(
        &ws.config.build_env.sbuild_cache_dir,
        &ws.config.build_env.distribution,
        &ws.config.build_env.arch,
    )?;

    // Build test setup commands from runtime
    let setup_commands = match source_language(&ws.config.source) {
        Some(lang) => {
            let rt = runtime::runtime_for(lang);
            rt.test_deps(&ws.config.build_env.distribution)
                .into_iter()
                .map(|cmd| format!("--setup-commands={}", cmd))
                .collect::<Vec<_>>()
        }
        None => vec![],
    };

    tool::autopkgtest::Autopkgtest {
        changes_file: &ws.changes_path,
        image_path: &image_path,
        deb_dir: &ws.build_artifacts_dir,
        setup_commands: &setup_commands,
    }
    .run()?;

    Ok(())
}

fn source_language(source: &SourceKind) -> Option<&LanguageEnv> {
    match source {
        SourceKind::Tarball { language, .. } | SourceKind::Git { language, .. } => Some(language),
        SourceKind::Virtual => None,
    }
}
