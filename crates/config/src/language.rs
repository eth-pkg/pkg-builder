use serde::Deserialize;

/// Language environment configuration for a package.
#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(tag = "language_env", rename_all = "lowercase")]
pub enum LanguageEnv {
    Rust(RustConfig),
    Go(GoConfig),
    JavaScript(JavascriptConfig),
    Java(JavaConfig),
    Dotnet(DotnetConfig),
    TypeScript(JavascriptConfig),
    Nim(NimConfig),
    #[default]
    C,
    Python,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RustConfig {
    pub rust_version: String,
    pub rust_binary_url: String,
    pub rust_binary_gpg_asc: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct GoConfig {
    pub go_version: String,
    pub go_binary_url: String,
    pub go_binary_checksum: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct JavascriptConfig {
    pub node_version: String,
    pub node_binary_url: String,
    pub node_binary_checksum: String,
    pub yarn_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct JavaConfig {
    pub is_oracle: bool,
    pub jdk_version: String,
    pub jdk_binary_url: String,
    pub jdk_binary_checksum: String,
    pub gradle: Option<GradleConfig>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct GradleConfig {
    pub gradle_version: String,
    pub gradle_binary_url: String,
    pub gradle_binary_checksum: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DotnetPackage {
    pub name: String,
    pub hash: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct DotnetConfig {
    pub use_backup_version: bool,
    #[serde(default)]
    pub dotnet_packages: Vec<DotnetPackage>,
    pub deps: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct NimConfig {
    pub nim_version: String,
    pub nim_binary_url: String,
    pub nim_version_checksum: String,
}
