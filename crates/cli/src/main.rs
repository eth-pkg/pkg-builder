mod commands;

use commands::{ActionType, PkgBuilderArgs};

use clap::Parser;
use env_logger::Env;
use log::error;
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
        if let Some(v) = cmd.run_piuparts {
            config.build_env.testing.run_piuparts = v;
        }
        if let Some(v) = cmd.run_autopkgtest {
            config.build_env.testing.run_autopkgtest = v;
        }
    }

    let builder = pipeline::PackageBuilder::new(config)?;

    match args.action {
        ActionType::Package(_) => builder.build()?,
        ActionType::Env(env_cmd) => match env_cmd.sub_command {
            commands::BuildEnvSubCommand::Create(_) => builder.create_env()?,
            commands::BuildEnvSubCommand::Clean(_) => builder.clean_env()?,
        },
        ActionType::Lintian(_) => builder.run_lintian()?,
        ActionType::Piuparts(_) => builder.run_piuparts()?,
        ActionType::Autopkgtest(_) => builder.run_autopkgtest()?,
        ActionType::Verify(verify_cmd) => {
            let verify_config_path = verify_cmd
                .verify_config
                .unwrap_or_else(|| config_path.clone());
            let verify_config = config::verify::PkgVerifyConfig::load(&verify_config_path)?;
            let skip_build = verify_cmd.no_package.unwrap_or(false);
            builder.verify(verify_config, skip_build)?;
        }
        ActionType::Version => unreachable!(),
    }

    Ok(())
}
