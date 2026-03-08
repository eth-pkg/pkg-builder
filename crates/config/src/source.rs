use serde::Deserialize;

use crate::language::LanguageEnv;

/// The kind of source for a package.
#[derive(Debug, Clone)]
pub enum SourceKind {
    Tarball {
        url: String,
        hash: Option<String>,
        language: LanguageEnv,
    },
    Git {
        url: String,
        tag: String,
        submodules: Vec<Submodule>,
        language: LanguageEnv,
    },
    Virtual,
}

impl SourceKind {
    /// Get the language environment for this source, if any.
    pub fn language(&self) -> Option<&LanguageEnv> {
        match self {
            SourceKind::Tarball { language, .. } => Some(language),
            SourceKind::Git { language, .. } => Some(language),
            SourceKind::Virtual => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Submodule {
    pub commit: String,
    pub path: String,
}
