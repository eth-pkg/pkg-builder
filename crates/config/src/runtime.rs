use std::collections::HashMap;

use serde::Deserialize;

/// Runtime configuration from the [runtime] section.
/// The `recipe` field selects the runtime recipe file.
/// All other fields are passed as template variables to the recipe.
#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeConfig {
    pub recipe: String,
    #[serde(flatten)]
    pub vars: HashMap<String, toml::Value>,
}
