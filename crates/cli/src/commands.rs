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
            ActionType::Package(cmd) => cmd.config.clone(),
            ActionType::Env(cmd) => match &cmd.sub_command {
                BuildEnvSubCommand::Create(c) => c.config.clone(),
                BuildEnvSubCommand::Clean(c) => c.config.clone(),
            },
            ActionType::Lintian(cmd) => cmd.config.clone(),
            ActionType::Piuparts(cmd) => cmd.config.clone(),
            ActionType::Autopkgtest(cmd) => cmd.config.clone(),
            ActionType::Verify(cmd) => cmd.config.clone(),
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum ActionType {
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
    /// Print version
    Version,
}

#[derive(Debug, Args)]
pub struct PackageCommand {
    /// Path to pkg-builder.toml config
    pub config: Option<String>,
    /// Override: run piuparts
    #[clap(long)]
    pub run_piuparts: Option<bool>,
    /// Override: run autopkgtest
    #[clap(long)]
    pub run_autopkgtest: Option<bool>,
    /// Override: run lintian
    #[clap(long)]
    pub run_lintian: Option<bool>,
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
    #[clap(long)]
    pub config: Option<String>,
    /// Path to pkg-builder-verify.toml config
    #[clap(long)]
    pub verify_config: Option<String>,
    /// Skip rebuilding the package
    #[clap(long)]
    pub no_package: Option<bool>,
}
