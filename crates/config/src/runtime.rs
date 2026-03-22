use std::collections::BTreeMap;

use serde::Deserialize;

/// Runtime configuration from the [runtime] section.
/// The `profile` field selects the runtime .mk profile.
/// `recipe` is accepted as a deprecated alias for `profile`.
/// All other fields are passed as variables to the generated Makefile.
#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeConfig {
    #[serde(alias = "recipe")]
    pub profile: String,
    #[serde(flatten)]
    pub vars: BTreeMap<String, toml::Value>,
}

impl RuntimeConfig {
    /// Alias for backwards compatibility — code that previously used `.recipe`
    /// can use this accessor during the migration period.
    pub fn recipe(&self) -> &str {
        &self.profile
    }
}
