mod commands;
mod init;
mod update;

use commands::{ActionType, EnvSubCommand, PkgBuilderArgs};

use clap::Parser;
use env_logger::Env;
use log::{error, info};
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

    match args.action {
        ActionType::Init(_) => unreachable!(),
        ActionType::Update(_) => unreachable!(),
        ActionType::Verify => {
            config::verify::verify_hashes(&config)?;
            info!("Verification successful!");
        }
        ActionType::Build => {
            executor::build(&config, args.resume)?;
        }
        ActionType::Env(ref env_cmd) => match env_cmd.sub_command {
            EnvSubCommand::Create => executor::create_env(&config)?,
            EnvSubCommand::Clean => executor::clean_env(&config)?,
        },
        ActionType::Clean => {
            executor::clean(&config)?;
        }
    }

    Ok(())
}
