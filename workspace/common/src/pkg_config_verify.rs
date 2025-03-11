use eyre::{eyre, Report};
use serde::Deserialize;

use crate::pkg_config::{validate_not_empty, Validation};


#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
pub struct PackageHash {
    pub name: String,
    pub hash: String,
}

impl Validation for PackageHash {
    fn validate(&self) -> eyre::Result<(), Vec<Report>> {
        let mut errors = Vec::new();

        if let Err(err) = validate_not_empty("name", &self.name) {
            errors.push(err);
        }

        if let Err(err) = validate_not_empty("hash", &self.hash) {
            errors.push(err);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
pub struct VerifyConfig {
    pub package_hash: Vec<PackageHash>,
}

impl Validation for VerifyConfig {
    fn validate(&self) -> eyre::Result<(), Vec<Report>> {
        if self.package_hash.is_empty() {
            let err = vec![eyre!("package_hash cannot be empty")];
            Err(err)
        } else {
            let mut errors = Vec::new();
            for packagehash in self.package_hash.iter() {
                if let Err(mut err) = packagehash.validate() {
                    if !err.is_empty() {
                        errors.append(&mut err);
                    }
                }
            }
            if errors.is_empty() {
                Ok(())
            } else {
                Err(errors)
            }
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
pub struct PkgVerifyConfig {
    pub verify: VerifyConfig,
}

impl Validation for PkgVerifyConfig {
    fn validate(&self) -> eyre::Result<(), Vec<Report>> {
        return self.verify.validate();
    }
}