use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct PkgBuilderArgs {
    #[clap(subcommand)]
    pub action: ActionType,
}

impl PkgBuilderArgs {
    pub fn config_path(&self) -> Option<String> {
        match &self.action {
            ActionType::Version => None,
            ActionType::Generate(cmd) => cmd.config.clone(),
            ActionType::Package(cmd) => cmd.config.clone(),
            ActionType::Env(cmd) => match &cmd.sub_command {
                BuildEnvSubCommand::Create(c) => c.config.clone(),
                BuildEnvSubCommand::Clean(c) => c.config.clone(),
            },
            ActionType::Lintian(cmd) => cmd.config.clone(),
            ActionType::Piuparts(cmd) => cmd.config.clone(),
            ActionType::Autopkgtest(cmd) => cmd.config.clone(),
            ActionType::Verify(cmd) => cmd.config.clone(),
            ActionType::Clean(cmd) => cmd.config.clone(),
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum ActionType {
    /// Generate Makefile without building
    Generate(DefaultCommand),
    /// Create package
    Package(PackageCommand),
    /// Clean, delete, create build environment
    Env(EnvCommand),
    /// Run piuparts tests
    Piuparts(DefaultCommand),
    /// Run autopkgtest tests
    Autopkgtest(DefaultCommand),
    /// Run lintian checks
    Lintian(DefaultCommand),
    /// Verify package hashes
    Verify(VerifyCommand),
    /// Clean build artifacts
    Clean(DefaultCommand),
    /// Print version
    Version,
}

#[derive(Debug, Args)]
pub struct PackageCommand {
    /// Path to pkg-builder.toml config
    pub config: Option<String>,
    /// Enable piuparts testing (disabled by default)
    #[clap(long)]
    pub run_piuparts: bool,
    /// Enable autopkgtest testing (disabled by default)
    #[clap(long)]
    pub run_autopkgtest: bool,
    /// Override: run lintian
    #[clap(long)]
    pub run_lintian: Option<bool>,
    /// Install missing dependencies instead of erroring
    #[clap(long)]
    pub install_deps: bool,
}

#[derive(Debug, Args)]
pub struct DefaultCommand {
    /// Path to pkg-builder.toml config
    pub config: Option<String>,
}

#[derive(Debug, Args)]
pub struct EnvCommand {
    #[clap(subcommand)]
    pub sub_command: BuildEnvSubCommand,
}

#[derive(Debug, Subcommand)]
pub enum BuildEnvSubCommand {
    /// Create build environment
    Create(DefaultCommand),
    /// Clean build environment
    Clean(DefaultCommand),
}

#[derive(Debug, Args)]
pub struct VerifyCommand {
    /// Path to pkg-builder.toml config
    pub config: Option<String>,
}
