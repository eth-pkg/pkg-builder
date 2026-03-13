use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct PkgBuilderArgs {
    /// Path to pkg-builder.toml config (default: current directory)
    #[clap(long, global = true)]
    pub config: Option<String>,

    /// Install missing dependencies instead of erroring
    #[clap(long, global = true)]
    pub install_deps: bool,

    #[clap(subcommand)]
    pub action: ActionType,
}

#[derive(Debug, Subcommand)]
pub enum ActionType {
    /// Interactive project setup wizard
    Init,
    /// Build the package
    Build(BuildCommand),
    /// Generate Makefile without building
    Generate,
    /// Manage build environment (sbuild chroot)
    Env(EnvCommand),
    /// Run tests
    Test(TestCommand),
    /// Verify package hashes
    Verify,
    /// Clean build artifacts
    Clean,
}

#[derive(Debug, Args)]
pub struct BuildCommand {
    /// Also run enabled tests after building
    #[clap(long)]
    pub with_tests: bool,
}

#[derive(Debug, Args)]
pub struct EnvCommand {
    #[clap(subcommand)]
    pub sub_command: EnvSubCommand,
}

#[derive(Debug, Subcommand)]
pub enum EnvSubCommand {
    /// Create build environment (sbuild chroot)
    Create,
    /// Remove build environment (chroot tarball)
    Clean,
}

#[derive(Debug, Args)]
pub struct TestCommand {
    #[clap(subcommand)]
    pub sub_command: Option<TestSubCommand>,
}

#[derive(Debug, Subcommand)]
pub enum TestSubCommand {
    /// Run lintian checks on host
    Lintian,
    /// Run piuparts tests
    Piuparts,
    /// Run autopkgtest tests
    Autopkgtest,
}
