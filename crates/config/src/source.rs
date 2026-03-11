use serde::Deserialize;

/// The kind of source for a package.
#[derive(Debug, Clone)]
pub enum SourceKind {
    Tarball {
        url: String,
        hash: Option<String>,
    },
    Git {
        url: String,
        tag: String,
        submodules: Vec<Submodule>,
    },
    Virtual,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Submodule {
    pub commit: String,
    pub path: String,
}
