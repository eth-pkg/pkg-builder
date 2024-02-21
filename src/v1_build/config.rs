// src/config.rs
pub struct PackageBuildConfig {
    pub arch: Vec<String>,
    pub source_url: String,
    pub previous_build_hash: String,
    pub source_is_git: bool,
}
