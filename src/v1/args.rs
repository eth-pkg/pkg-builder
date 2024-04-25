use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct PkgBuilderArgs {
    #[clap(subcommand)]
    pub action: ActionType,
}

#[derive(Debug, Subcommand)]
pub enum ActionType {
    /// create package
    Package(PackageCommand),
    /// clean, delete, create buildenv for package, you must run with sudo
    BuildEnv(BuildEnvCommand),
    Piuparts(PackageCommand),
    Autopkgtest(PackageCommand)
}

#[derive(Debug, Args)]
pub struct PackageCommand {
    /// location of pkg-builder config_file
    pub config_file: String,
}

#[derive(Debug, Args)]
pub struct BuildEnvCommand {
    #[clap(subcommand)]
    pub build_env_sub_command: BuildEnvSubCommand,
}
#[derive(Debug, Subcommand)]
pub enum BuildEnvSubCommand {
    Create(CreateBuildEnvCommand),
    Clean(CleanBuildEnvCommand),
}

#[derive(Debug, Args)]
pub struct CreateBuildEnvCommand {
    /// location of pkg-builder config_file
    pub config_file: String,
}
#[derive(Debug, Args)]
pub struct CleanBuildEnvCommand {
    /// location of pkg-builder config_file
    pub config_file: String,
}
