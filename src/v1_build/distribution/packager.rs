use serde::Deserialize;

use super::bookworm::{BookwormPackager, BookwormPackagerConfig};

pub trait PackagerConfig {}

pub trait Packager {
    type Config: PackagerConfig;
    fn new(config: Self::Config) -> Self;
    fn package(&self) -> Result<bool, bool>;
}

pub struct DistributionPackager {
    config: DistributionPackagerConfig,
}

#[derive(Debug, Deserialize)]
pub struct DistributionPackagerConfig {}


impl DistributionPackager {
    pub fn new(config: DistributionPackagerConfig) -> Self {
        return DistributionPackager { config };
    }
    pub fn package(&self) -> Result<bool, bool> {
        let config = BookwormPackagerConfig {};
        let packager = BookwormPackager::new(config);
        packager.package()
    }
}
