use thiserror::Error;

#[derive(Error, Debug)]
pub enum DistributionError {
    #[error("Unsupported distribution codename: {0}")]
    UnsupportedCodename(String),
}

/// Represents supported Linux distributions for VM image creation
///
/// Each variant contains the specific codename for the distribution
/// which is used to identify compatible build commands and arguments.
#[derive(Debug, Clone)]
pub enum Distribution {
    /// Debian distribution with codename (e.g., "bookworm")
    Debian(String),
    /// Ubuntu distribution with codename (e.g., "noble", "jammy")
    Ubuntu(String),
}

impl Distribution {
    /// Creates a Distribution from a codename string
    ///
    /// # Arguments
    /// * `codename` - The distribution codename (e.g., "bookworm", "noble")
    ///
    /// # Returns
    /// * `Result<Self>` - A Distribution instance or an error if unsupported
    pub fn from_codename(codename: &str) -> Result<Self, DistributionError> {
        match codename {
            "bookworm" => Ok(Distribution::Debian(codename.to_string())),
            "noble" | "noble numbat" | "jammy" | "jammy jellyfish" => {
                Ok(Distribution::Ubuntu(codename.to_string()))
            }
            _ => Err(DistributionError::UnsupportedCodename(codename.to_string())),
        }
    }
}
