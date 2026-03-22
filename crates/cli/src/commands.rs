use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct PkgBuilderArgs {
    /// Path to pkg-builder.toml config (default: current directory)
    #[clap(long, global = true)]
    pub config: Option<String>,

    /// Resume a previous build (skip steps whose artifacts already exist)
    #[clap(long, global = true)]
    pub resume: bool,

    #[clap(subcommand)]
    pub action: ActionType,
}

#[derive(Debug, Subcommand)]
pub enum ActionType {
    /// Interactive project setup wizard
    Init(InitCommand),
    /// Update package to a new upstream version
    Update(UpdateCommand),
    /// Build the package
    Build,
    /// Manage build environment (sbuild chroot)
    Env(EnvCommand),
    /// Verify package hashes
    Verify,
    /// Clean build artifacts
    Clean,
}

#[derive(Debug, Args)]
pub struct InitCommand {
    /// Package name
    #[clap(long)]
    pub name: Option<String>,
    /// Package version
    #[clap(long)]
    pub version: Option<String>,
    /// Package revision
    #[clap(long)]
    pub revision: Option<String>,
    /// Homepage URL
    #[clap(long)]
    pub homepage: Option<String>,
    /// Maintainer name
    #[clap(long)]
    pub maintainer_name: Option<String>,
    /// Maintainer email
    #[clap(long)]
    pub maintainer_email: Option<String>,
    /// Debian section (e.g., net, utils, devel, admin, libs, web, etc.)
    #[clap(long)]
    pub section: Option<String>,
    /// Short package description/summary
    #[clap(long)]
    pub summary: Option<String>,
    /// Source type: tarball, git, or virtual
    #[clap(long)]
    pub source_type: Option<String>,
    /// Source URL (tarball URL or git repo URL)
    #[clap(long)]
    pub url: Option<String>,
    /// Git tag (for git source type)
    #[clap(long)]
    pub tag: Option<String>,
    /// Target distribution: bookworm, trixie, or noble
    #[clap(long)]
    pub distribution: Option<String>,
    /// Target architecture
    #[clap(long)]
    pub arch: Option<String>,
    /// Runtime recipe: go, rust, node, java, java-gradle, nim, c, etc. Use "none" for no runtime
    #[clap(long)]
    pub runtime: Option<String>,
    /// Output directory (default: current directory)
    #[clap(long)]
    pub output: Option<String>,
    /// Expected upstream hash for tarball verification
    #[clap(long)]
    pub upstream_hash: Option<String>,
}

#[derive(Debug, Args)]
pub struct UpdateCommand {
    /// New upstream version (omit to auto-detect from GitHub)
    #[clap(long)]
    pub version: Option<String>,
    /// Package revision (default: "1")
    #[clap(long)]
    pub revision: Option<String>,
    /// Changelog message (default: "New upstream version {version}")
    #[clap(long)]
    pub changelog_msg: Option<String>,
    /// GitHub owner/repo (inferred from source URL if omitted)
    #[clap(long)]
    pub github_repo: Option<String>,
    /// Update files in the package directory pointed to by --config
    #[clap(long)]
    pub in_place: bool,
    /// Write to a new directory (copies all files + applies updates)
    #[clap(long)]
    pub output: Option<String>,
    /// Skip download, use this source hash directly
    #[clap(long)]
    pub hash: Option<String>,
    /// Skip GitHub lookup, use this commit hash
    #[clap(long)]
    pub git_commit: Option<String>,
    /// Don't download source (leave hash empty/unchanged, skip patch test)
    #[clap(long)]
    pub skip_download: bool,
    /// Also update runtime to latest version
    #[clap(long)]
    pub update_runtime: bool,
    /// Skip patch application testing
    #[clap(long)]
    pub skip_patch_check: bool,
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
