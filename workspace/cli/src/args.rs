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
    /// clean, delete, create buildenv for package
    Env(EnvCommand),
    /// run package update, remove, install tests
    Piuparts(DefaultCommand),
    /// run tests against built deb package
    Autopkgtest(DefaultCommand),
    /// run linting against package
    Lintian(DefaultCommand),

    /// Verify package against hashes, it also rebuilds the package
    Verify(VerifyConfig),
    // pkg-builder version
    Version,
}

#[derive(Debug, Args)]
pub struct VerifyConfig {
    /// location of pkg-builder config_file, either full path
    /// or directory to pkg-builder.toml is located
    /// if not given current directory is searched for pkg-builder.toml
    #[clap(long)]
    pub config: Option<String>,

    /// location of pkg-builder config_file, either full path
    /// or directory to pkg-builder-verify.toml is located
    /// if not given current directory is searched for pkg-builder-verify.toml
    #[clap(long)]
    pub verify_config: Option<String>,

    /// if given it won't repackage it
    #[clap(long)]
    pub no_package: Option<bool>,
}

#[derive(Debug, Args)]
pub struct DefaultCommand {
    /// location of pkg-builder config_file, either full path
    /// or directory to pkg-builder.toml is located
    /// if not given current directory is searched for pkg-builder.toml
    pub config: Option<String>,
}

#[derive(Debug, Args)]
pub struct PackageCommand {
    /// location of pkg-builder config_file, either full path
    /// or directory to pkg-builder.toml is located
    /// if not given current directory is searched for pkg-builder.toml
    pub config: Option<String>,
    /// overrides config value
    /// runs piuparts or not based on supplied value
    #[clap(long)]
    pub run_piuparts: Option<bool>,
    /// overrides config value
    /// runs autopkgtest or not based on supplied value
    #[clap(long)]
    pub run_autopkgtest: Option<bool>,
    #[clap(long)]
    /// runs lintian or not, based on value, overrides config value
    pub run_lintian: Option<bool>,
}

#[derive(Debug, Args)]
pub struct EnvCommand {
    #[clap(subcommand)]
    pub build_env_sub_command: BuildEnvSubCommand,
}
#[derive(Debug, Subcommand)]
pub enum BuildEnvSubCommand {
    /// creates build env used for packaging
    Create(CreateBuildEnvCommand),
    /// removes build env
    Clean(CleanBuildEnvCommand),
}

#[derive(Debug, Args)]
pub struct CreateBuildEnvCommand {
    /// location of pkg-builder config_file, either full path
    /// or directory to pkg-builder.toml is located
    /// if not given current directory is searched for pkg-builder.toml
    pub config: Option<String>,
}
#[derive(Debug, Args)]
pub struct CleanBuildEnvCommand {
    /// location of pkg-builder config_file, either full path
    /// or directory to pkg-builder.toml is located
    /// if not given current directory is searched for pkg-builder.toml
    pub config: Option<String>,
}
