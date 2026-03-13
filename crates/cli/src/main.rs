mod commands;

use commands::{ActionType, EnvSubCommand, PkgBuilderArgs, TestSubCommand};

use clap::Parser;
use env_logger::Env;
use log::{error, info};
use sha1::{Digest, Sha1};
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

    if let ActionType::Init = &args.action {
        println!("pkg-builder init wizard coming soon!");
        return Ok(());
    }

    let config_path = args.config.unwrap_or_else(|| ".".to_string());
    let config = config::PkgConfig::load(&config_path)?;

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
        ActionType::Init => unreachable!(),
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
        ActionType::Verify => {
            let verify_config = config
                .verify
                .clone()
                .ok_or("No [verify] section in pkg-builder.toml")?;
            verify_hashes(&config, &verify_config)?;
            return Ok(());
        }
    };

    // Run make
    let install_deps = args.install_deps;
    let mut make_cmd = Command::new("make");
    make_cmd.arg(target).current_dir(&output_dir);
    if install_deps {
        make_cmd.arg("INSTALL_DEPS=1");
    }
    info!("Running: make {}", target);
    let status = make_cmd.status()?;

    if !status.success() {
        let code = status.code().unwrap_or(1);
        return Err(format!("make {} failed with exit code {}", target, code).into());
    }

    Ok(())
}

fn verify_hashes(
    config: &config::PkgConfig,
    verify_config: &config::verify::VerifyConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let pkg = &config.package;
    let artifacts_dir = config.build_env.workdir.join(format!(
        "{}-{}-{}",
        pkg.name, pkg.version, pkg.revision
    ));

    let mut errors = Vec::new();

    for pkg_hash in &verify_config.package_hash {
        let file_path = artifacts_dir.join(&pkg_hash.name);

        if !file_path.exists() {
            errors.push(format!("Verification file missing: {}", pkg_hash.name));
            continue;
        }

        let buffer = std::fs::read(&file_path)?;
        let mut hasher = Sha1::new();
        hasher.update(&buffer);
        let actual: String = hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect();

        if actual != pkg_hash.hash {
            errors.push(format!(
                "SHA1 mismatch for {}: expected {}, got {}",
                pkg_hash.name, pkg_hash.hash, actual
            ));
        }
    }

    if errors.is_empty() {
        info!("Verification successful!");
        Ok(())
    } else {
        Err(errors.join("; ").into())
    }
}
