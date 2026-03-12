mod commands;

use commands::{ActionType, PkgBuilderArgs};

use clap::Parser;
use env_logger::Env;
use log::{error, info};
use std::path::Path;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            error!("{e:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = PkgBuilderArgs::parse();

    if let ActionType::Version = &args.action {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let config_path = args.config_path();
    let config_path = config_path.unwrap_or_else(|| ".".to_string());

    let mut config = config::PkgConfig::load(&config_path)?;

    // Apply CLI overrides
    if let ActionType::Package(ref cmd) = args.action {
        if let Some(v) = cmd.run_lintian {
            config.build_env.testing.run_lintian = v;
        }
        config.build_env.testing.run_piuparts = cmd.run_piuparts;
        config.build_env.testing.run_autopkgtest = cmd.run_autopkgtest;
    }

    // Generate Makefile
    let makefile_content = makefile::generate(&config)?;

    let output_path = Path::new(&config_path);
    let output_dir = if output_path.is_dir() {
        output_path.to_path_buf()
    } else {
        output_path
            .parent()
            .unwrap_or(Path::new("."))
            .to_path_buf()
    };
    let makefile_path = output_dir.join("Makefile");
    std::fs::write(&makefile_path, &makefile_content)?;
    info!("Generated Makefile at {:?}", makefile_path);

    // Determine which make target to run
    let target = match args.action {
        ActionType::Generate(_) => return Ok(()),
        ActionType::Package(_) => "all",
        ActionType::Env(ref env_cmd) => match env_cmd.sub_command {
            commands::BuildEnvSubCommand::Create(_) => "env",
            commands::BuildEnvSubCommand::Clean(_) => "clean",
        },
        ActionType::Clean(_) => "clean",
        ActionType::Lintian(_) => "test-lintian",
        ActionType::Piuparts(_) => "test-piuparts",
        ActionType::Autopkgtest(_) => "test-autopkgtest",
        ActionType::Verify(_) => {
            let verify_config = config.verify.clone().ok_or("No [verify] section in pkg-builder.toml")?;
            let builder = pipeline::PackageBuilder::new(config)?;
            builder.verify(verify_config)?;
            return Ok(());
        }
        ActionType::Version => unreachable!(),
    };

    // Run make
    info!("Running: make {}", target);
    let status = Command::new("make")
        .arg(target)
        .current_dir(&output_dir)
        .status()?;

    if !status.success() {
        let code = status.code().unwrap_or(1);
        return Err(format!("make {} failed with exit code {}", target, code).into());
    }

    Ok(())
}
