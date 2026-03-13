/// The complete build plan — the single IR for the entire system.
#[derive(Debug, Clone, PartialEq)]
pub struct BuildPlan {
    pub preamble: Preamble,
    pub phases: Vec<Phase>,
}

/// Setup that applies before any build phase.
#[derive(Debug, Clone, PartialEq)]
pub struct Preamble {
    /// Variables to declare (from config).
    pub variables: Vec<VarDecl>,
    /// Tools that must be present (from REQUIRE).
    pub required_tools: Vec<String>,
    /// Tools to auto-install if missing (from INSTALL).
    pub installable_tools: Vec<ToolInstall>,
    /// Commands to run inside sbuild chroot before building.
    /// Populated from runtime recipe + SNAPSHOT_WORKAROUND/NOBLE_REPOS etc.
    pub chroot_setup: Vec<Operation>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarDecl {
    pub name: String,
    pub value: VarValue,
    pub kind: AssignKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignKind {
    Immediate,
    Deferred,
    Default,
    Append,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VarValue {
    Literal(String),
    List(Vec<String>),
    /// Raw text to be emitted verbatim (for complex formatting that doesn't fit the model).
    Raw(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToolInstall {
    pub name: String,
    pub install_cmd: String,
}

/// A named build phase with operations and dependency info.
#[derive(Debug, Clone, PartialEq)]
pub struct Phase {
    pub name: String,
    /// What artifact this phase produces (filled by builder).
    pub output: Option<String>,
    /// Hard dependencies (filled by builder).
    pub deps: Vec<String>,
    /// Order-only dependencies (filled by builder).
    pub order_only_deps: Vec<String>,
    /// The build operations in this phase.
    pub operations: Vec<Operation>,
    /// If Some, this phase is gated on a condition.
    pub condition: Option<Condition>,
    /// If true, this is a phony alias (no file produced).
    pub is_alias: bool,
}

/// A build operation — the semantic commands that recipes express.
/// This replaces the old Command enum.
#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    // Source acquisition
    Download { url: String, dest: String },
    Verify { algo: String, hash: String, file: String },
    VerifyGpg { file: String, sig: String },
    Extract { file: String, dest: String, strip: Option<u32> },
    GitClone { url: String, tag: String },
    CreateEmptyTar,

    // Package management
    AptInstall { packages: Vec<String> },
    AptRemove { packages: Vec<String> },
    AptUpdate,
    DpkgInstall { file: String },

    // Build pipeline
    Debcrafter { spec: String },
    Patch,
    Sbuild,

    // Testing
    Lintian,
    Piuparts,
    Autopkgtest,

    // File operations
    Symlink { src: String, target: String },
    Run { cmd: String },

    // Chroot environment
    SnapshotWorkaround,
    SnapshotSecurity { url: String, codename: String },
    NobleRepos,

    // Meta
    Include { name: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    Eq(String, String),
    Neq(String, String),
}
