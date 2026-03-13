mod commands;
mod init;
mod update;

use commands::{ActionType, EnvSubCommand, PkgBuilderArgs, TestSubCommand};

use clap::Parser;
use env_logger::Env;
use log::{error, info};
use std::path::Path;
use std::process::ExitCode;

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

    if let ActionType::Init(ref init_cmd) = args.action {
        return init::run(init_cmd);
    }

    let config_path = args.config.unwrap_or_else(|| ".".to_string());

    if let ActionType::Update(ref update_cmd) = args.action {
        return update::run(update_cmd, &config_path);
    }
    let config = config::PkgConfig::load(&config_path)?;

    let output_path = Path::new(&config_path);
    let output_dir = if output_path.is_dir() {
        output_path.to_path_buf()
    } else {
        output_path.parent().unwrap_or(Path::new(".")).to_path_buf()
    };

    // Handle verify early — no Makefile generation needed
    if let ActionType::Verify = args.action {
        config::verify::verify_hashes(&config)?;
        info!("Verification successful!");
        return Ok(());
    }

    // Generate Makefile
    let makefile_content = makefile::generate(&config)?;
    let makefile_path = output_dir.join("Makefile");
    std::fs::write(&makefile_path, &makefile_content)?;
    info!("Generated Makefile at {:?}", makefile_path);

    // Determine which make target to run
    let target = match args.action {
        ActionType::Init(_) => unreachable!(),
        ActionType::Update(_) => unreachable!(),
        ActionType::Verify => unreachable!(),
        ActionType::Generate => return Ok(()),
        ActionType::Build(ref cmd) => {
            if cmd.with_tests {
                "all"
            } else {
                "build"
            }
        }
        ActionType::Env(ref env_cmd) => match env_cmd.sub_command {
            EnvSubCommand::Create => "env",
            EnvSubCommand::Clean => "env-clean",
        },
        ActionType::Clean => "clean",
        ActionType::Test(ref test_cmd) => match test_cmd.sub_command {
            None => "test",
            Some(TestSubCommand::Lintian) => "test-lintian",
            Some(TestSubCommand::Piuparts) => "test-piuparts",
            Some(TestSubCommand::Autopkgtest) => "test-autopkgtest",
        },
    };

    // Run make
    info!("Running: make {}", target);
    makefile::run_make(target, &output_dir, args.install_deps)?;

    Ok(())
}
