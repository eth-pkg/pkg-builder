use serde::Deserialize;
use crate::v1::pkg_config::{deserialize_option_empty_string};


#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
pub struct Verify {
    #[serde(deserialize_with = "deserialize_option_empty_string")]
    pub package_hash: Option<String>,
}
