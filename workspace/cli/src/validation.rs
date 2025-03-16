use types::{
    pkg_config::{
        BuildEnv, DefaultPackageTypeConfig, DotnetConfig, DotnetPackage, GitPackageTypeConfig,
        GoConfig, GradleConfig, JavaConfig, JavascriptConfig, LanguageEnv, NimConfig,
        PackageFields, PackageType, PkgConfig, RustConfig, SubModule,
    },
    pkg_config_verify::{PackageHash, PkgVerifyConfig, VerifyConfig},
};
use eyre::{Report, Result, eyre};
use serde::de::DeserializeOwned;
use std::fs;
use std::path::Path;

pub trait Validation {
    fn validate(&self) -> Result<(), Vec<Report>>;
}

pub fn validate_not_empty(name: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(eyre!("field: {} cannot be empty", name));
    }
    Ok(())
}

macro_rules! validate_fields {
    ($obj:expr, $($field:ident),*) => {
        {
            let mut errors = Vec::new();
            $(
                if let Err(err) = validate_not_empty(stringify!($field), &$obj.$field) {
                    errors.push(err);
                }
            )*
            errors
        }
    };
}

macro_rules! impl_validation {
    ($struct_name:ident, $($field:ident),*) => {
        impl Validation for $struct_name {
            fn validate(&self) -> Result<(), Vec<Report>> {
                let errors = validate_fields!(self, $($field),*);

                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(errors)
                }
            }
        }
    };
}

impl_validation!(
    RustConfig,
    rust_version,
    rust_binary_url,
    rust_binary_gpg_asc
);
impl_validation!(GoConfig, go_version, go_binary_url, go_binary_checksum);
impl_validation!(
    GradleConfig,
    gradle_version,
    gradle_binary_url,
    gradle_binary_checksum
);
impl_validation!(JavaConfig, jdk_version, jdk_binary_url, jdk_binary_checksum);
impl_validation!(DotnetPackage, name, hash, url);
impl_validation!(NimConfig, nim_version, nim_binary_url, nim_version_checksum);
impl_validation!(SubModule, commit, path);
impl_validation!(GitPackageTypeConfig, git_tag, git_url);
impl_validation!(
    PackageFields,
    spec_file,
    package_name,
    version_number,
    revision_number,
    homepage
);
impl_validation!(
    BuildEnv,
    codename,
    arch,
    pkg_builder_version,
    debcrafter_version,
    lintian_version,
    piuparts_version,
    autopkgtest_version,
    sbuild_version
);
impl_validation!(PackageHash, name, hash);

impl Validation for JavascriptConfig {
    fn validate(&self) -> Result<(), Vec<Report>> {
        let mut errors =
            validate_fields!(self, node_version, node_binary_url, node_binary_checksum);

        if let Some(yarn_version) = &self.yarn_version {
            if let Err(err) = validate_not_empty("yarn_version", yarn_version) {
                errors.push(err);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Validation for DotnetConfig {
    fn validate(&self) -> Result<(), Vec<Report>> {
        Ok(()) // No validation rules for DotnetConfig fields
    }
}

impl Validation for LanguageEnv {
    fn validate(&self) -> Result<(), Vec<Report>> {
        match self {
            LanguageEnv::Rust(config) => config.validate(),
            LanguageEnv::Go(config) => config.validate(),
            LanguageEnv::JavaScript(config) => config.validate(),
            LanguageEnv::Java(config) => config.validate(),
            LanguageEnv::Dotnet(config) => config.validate(),
            LanguageEnv::TypeScript(config) => config.validate(),
            LanguageEnv::Nim(config) => config.validate(),
            LanguageEnv::C => Ok(()),
            LanguageEnv::Python => Ok(()),
        }
    }
}

impl Validation for DefaultPackageTypeConfig {
    fn validate(&self) -> Result<(), Vec<Report>> {
        let mut errors = validate_fields!(self, tarball_url);

        if let Some(value) = &self.tarball_hash {
            if let Err(err) = validate_not_empty("tarball_hash", value) {
                errors.push(err);
            }
        }

        if let Err(mut language_errors) = self.language_env.validate() {
            errors.append(&mut language_errors);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Validation for PackageType {
    fn validate(&self) -> Result<(), Vec<Report>> {
        match self {
            PackageType::Default(config) => config.validate(),
            PackageType::Git(config) => config.validate(),
            PackageType::Virtual => Ok(()),
        }
    }
}

impl Validation for PkgConfig {
    fn validate(&self) -> Result<(), Vec<Report>> {
        let mut errors = Vec::new();

        // Collect validation errors from all components
        if let Err(mut field_errors) = self.package_fields.validate() {
            errors.append(&mut field_errors);
        }

        if let Err(mut type_errors) = self.package_type.validate() {
            errors.append(&mut type_errors);
        }

        if let Err(mut env_errors) = self.build_env.validate() {
            errors.append(&mut env_errors);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
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

impl Validation for PkgVerifyConfig {
    fn validate(&self) -> eyre::Result<(), Vec<Report>> {
        return self.verify.validate();
    }
}

pub fn parse<T>(config_str: &str) -> Result<T>
where
    T: Validation + DeserializeOwned,
{
    let configuration = toml::from_str::<T>(config_str)?;
    configuration
        .validate()
        .map_err(|errors| eyre!("Validation failed: {:?}", errors))?;
    Ok(configuration)
}

pub fn read_config<T>(path: &Path) -> Result<T>
where
    T: Validation + DeserializeOwned,
{
    let toml_content = fs::read_to_string(path)?;
    parse(&toml_content)
}

pub fn get_config<T>(config_file: String) -> Result<T>
where
    T: Validation + DeserializeOwned,
{
    let path = Path::new(&config_file);
    read_config(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_config() {
        let config_str = r#"
[package_fields]
spec_file = "hello-world.sss"
package_name = "hello-world"
version_number = "1.0.0"
revision_number = "1"
homepage="https://github.com/eth-pkg/pkg-builder#examples"

[package_type]
package_type="default"
tarball_url = "hello-world-1.0.0.tar.gz"
git_source = ""
git_commit=""

[package_type.language_env]
language_env = "rust"
rust_version = "1.22"
rust_binary_url = "http:://example.com"
rust_binary_gpg_asc = "binary_key"
go_version = "1.22"


[build_env]
codename="bookworm"
arch = "amd64"
pkg_builder_version="0.3.0"
debcrafter_version = "8189263"
run_lintian=false
run_piuparts=false
run_autopkgtest=false
lintian_version="2.116.3"
piuparts_version="1.1.7"
autopkgtest_version="5.28"
sbuild_version="0.85.6"
workdir="~/.pkg-builder/packages/jammy"
"#;
        let config = PkgConfig {
            package_fields: PackageFields {
                spec_file: "hello-world.sss".to_string(),
                package_name: "hello-world".to_string(),
                version_number: "1.0.0".to_string(),
                revision_number: "1".to_string(),
                homepage: "https://github.com/eth-pkg/pkg-builder#examples".to_string(),
            },
            package_type: PackageType::Default(DefaultPackageTypeConfig {
                tarball_url: "hello-world-1.0.0.tar.gz".to_string(),
                tarball_hash: None,
                language_env: LanguageEnv::Rust(RustConfig {
                    rust_version: "1.22".to_string(),
                    rust_binary_url: "http:://example.com".to_string(),
                    rust_binary_gpg_asc: "binary_key".to_string(),
                }),
            }),
            build_env: BuildEnv {
                codename: "bookworm".to_string(),
                arch: "amd64".to_string(),
                pkg_builder_version: "0.3.0".to_string(),
                debcrafter_version: "8189263".to_string(),
                sbuild_cache_dir: None,
                docker: None,
                run_lintian: Some(false),
                run_piuparts: Some(false),
                run_autopkgtest: Some(false),
                lintian_version: "2.116.3".to_string(),
                piuparts_version: "1.1.7".to_string(),
                autopkgtest_version: "5.28".to_string(),
                sbuild_version: "0.85.6".to_string(),
                workdir: Some("~/.pkg-builder/packages/jammy".to_string()),
            },
        };
        assert_eq!(parse::<PkgConfig>(config_str).unwrap(), config);
    }

    #[test]
    fn test_empty_strings_are_error_rust_config() {
        let config = RustConfig::default();
        match config.validate() {
            Err(validation_errors) => {
                let expected_errors = [
                    "field: rust_version cannot be empty",
                    "field: rust_binary_url cannot be empty",
                    "field: rust_binary_gpg_asc cannot be empty",
                ];
                assert_eq!(
                    validation_errors.len(),
                    expected_errors.len(),
                    "Number of errors is different"
                );
                for (actual, expected) in validation_errors.iter().zip(expected_errors.iter()) {
                    assert_eq!(actual.to_string(), *expected);
                }
            }
            Ok(_) => panic!("Validation should have failed."),
        }
    }

    #[test]
    fn test_empty_strings_are_error_go_config() {
        let config = GoConfig::default();
        match config.validate() {
            Err(validation_errors) => {
                let expected_errors = [
                    "field: go_version cannot be empty",
                    "field: go_binary_url cannot be empty",
                    "field: go_binary_checksum cannot be empty",
                ];
                assert_eq!(
                    validation_errors.len(),
                    expected_errors.len(),
                    "Number of errors is different"
                );
                for (actual, expected) in validation_errors.iter().zip(expected_errors.iter()) {
                    assert_eq!(actual.to_string(), *expected);
                }
            }
            Ok(_) => panic!("Validation should have failed."),
        }
    }

    #[test]
    fn test_empty_strings_are_error_javascript_config() {
        let config = JavascriptConfig::default();
        match config.validate() {
            Err(validation_errors) => {
                let expected_errors = [
                    "field: node_version cannot be empty",
                    "field: node_binary_url cannot be empty",
                    "field: node_binary_checksum cannot be empty",
                ];
                assert_eq!(
                    validation_errors.len(),
                    expected_errors.len(),
                    "Number of errors is different"
                );
                for (actual, expected) in validation_errors.iter().zip(expected_errors.iter()) {
                    assert_eq!(actual.to_string(), *expected);
                }
            }
            Ok(_) => panic!("Validation should have failed."),
        }
    }

    #[test]
    fn test_empty_strings_are_error_java_config() {
        let config = JavaConfig::default();
        match config.validate() {
            Err(validation_errors) => {
                let expected_errors = [
                    "field: jdk_version cannot be empty",
                    "field: jdk_binary_url cannot be empty",
                    "field: jdk_binary_checksum cannot be empty",
                ];
                assert_eq!(
                    validation_errors.len(),
                    expected_errors.len(),
                    "Number of errors is different"
                );
                for (actual, expected) in validation_errors.iter().zip(expected_errors.iter()) {
                    assert_eq!(actual.to_string(), *expected);
                }
            }
            Ok(_) => panic!("Validation should have failed."),
        }
    }

    // #[test]
    // fn test_empty_strings_are_error_dotnet_config() {
    //     let config = DotnetConfig::default();
    //     match config.validate() {
    //         Err(validation_errors) => {
    //             let expected_errors: Vec<String>= vec![];
    //             assert_eq!(
    //                 validation_errors.len(),
    //                 expected_errors.len(),
    //                 "Number of errors is different"
    //             );
    //             for (actual, expected) in validation_errors.iter().zip(expected_errors.iter()) {
    //                 assert_eq!(actual.to_string(), *expected);
    //             }
    //         }
    //         Ok(_) => panic!("Validation should have failed."),
    //     }
    // }

    // #[test]
    // fn test_empty_strings_are_error_typescript_config() {
    //     let config = TypescriptConfig {
    //         node_version: "".to_string(),
    //         yarn_version: "".to_string(),
    //     };
    //     match config.validate() {
    //         Err(validation_errors) => {
    //             let expected_errors = [
    //                 "field: node_version cannot be empty",
    //                 "field: yarn_version cannot be empty",
    //             ];
    //             assert_eq!(
    //                 validation_errors.len(),
    //                 expected_errors.len(),
    //                 "Number of errors is different"
    //             );
    //             for (actual, expected) in validation_errors.iter().zip(expected_errors.iter()) {
    //                 assert_eq!(actual.to_string(), *expected);
    //             }
    //         }
    //         Ok(_) => panic!("Validation should have failed."),
    //     }
    // }

    #[test]
    fn test_empty_strings_are_error_nim_config() {
        let config = NimConfig::default();
        match config.validate() {
            Err(validation_errors) => {
                let expected_errors = [
                    "field: nim_version cannot be empty",
                    "field: nim_binary_url cannot be empty",
                    "field: nim_version_checksum cannot be empty",
                ];
                assert_eq!(
                    validation_errors.len(),
                    expected_errors.len(),
                    "Number of errors is different"
                );
                for (actual, expected) in validation_errors.iter().zip(expected_errors.iter()) {
                    assert_eq!(actual.to_string(), *expected);
                }
            }
            Ok(_) => panic!("Validation should have failed."),
        }
    }

    #[test]
    fn test_empty_strings_are_error_default_package_type_config() {
        let config = DefaultPackageTypeConfig::default();
        match config.validate() {
            Err(validation_errors) => {
                let expected_errors = ["field: tarball_url cannot be empty"];
                assert_eq!(
                    validation_errors.len(),
                    expected_errors.len(),
                    "Number of errors is different"
                );
                for (actual, expected) in validation_errors.iter().zip(expected_errors.iter()) {
                    assert_eq!(actual.to_string(), *expected);
                }
            }
            Ok(_) => panic!("Validation should have failed."),
        }
    }

    #[test]
    fn test_empty_strings_are_error_git_package_type_config() {
        let config = GitPackageTypeConfig::default();
        match config.validate() {
            Err(validation_errors) => {
                let expected_errors = [
                    "field: git_tag cannot be empty",
                    "field: git_url cannot be empty",
                ];
                assert_eq!(
                    validation_errors.len(),
                    expected_errors.len(),
                    "Number of errors is different"
                );
                for (actual, expected) in validation_errors.iter().zip(expected_errors.iter()) {
                    assert_eq!(actual.to_string(), *expected);
                }
            }
            Ok(_) => panic!("Validation should have failed."),
        }
    }

    #[test]
    fn test_empty_strings_are_error_gradle_config() {
        let config = GradleConfig::default();
        match config.validate() {
            Err(validation_errors) => {
                let expected_errors = [
                    "field: gradle_version cannot be empty",
                    "field: gradle_binary_url cannot be empty",
                    "field: gradle_binary_checksum cannot be empty",
                ];
                assert_eq!(
                    validation_errors.len(),
                    expected_errors.len(),
                    "Number of errors is different"
                );
                for (actual, expected) in validation_errors.iter().zip(expected_errors.iter()) {
                    assert_eq!(actual.to_string(), *expected);
                }
            }
            Ok(_) => panic!("Validation should have failed."),
        }
    }

    #[test]
    fn test_empty_strings_are_error_package_fields() {
        let config = PackageFields::default();
        match config.validate() {
            Err(validation_errors) => {
                let expected_errors = [
                    "field: spec_file cannot be empty",
                    "field: package_name cannot be empty",
                    "field: version_number cannot be empty",
                    "field: revision_number cannot be empty",
                    "field: homepage cannot be empty",
                ];
                assert_eq!(
                    validation_errors.len(),
                    expected_errors.len(),
                    "Number of errors is different"
                );
                for (actual, expected) in validation_errors.iter().zip(expected_errors.iter()) {
                    assert_eq!(actual.to_string(), *expected);
                }
            }
            Ok(_) => panic!("Validation should have failed."),
        }
    }

    #[test]
    fn test_empty_strings_are_error_build_env() {
        let config = BuildEnv::default();
        match config.validate() {
            Err(validation_errors) => {
                let expected_errors = [
                    "field: codename cannot be empty",
                    "field: arch cannot be empty",
                    "field: pkg_builder_version cannot be empty",
                    "field: debcrafter_version cannot be empty",
                    "field: lintian_version cannot be empty",
                    "field: piuparts_version cannot be empty",
                    "field: autopkgtest_version cannot be empty",
                    "field: sbuild_version cannot be empty",
                ];
                assert_eq!(
                    validation_errors.len(),
                    expected_errors.len(),
                    "Number of errors is different"
                );
                for (actual, expected) in validation_errors.iter().zip(expected_errors.iter()) {
                    assert_eq!(actual.to_string(), *expected);
                }
            }
            Ok(_) => panic!("Validation should have failed."),
        }
    }

    #[test]
    fn test_validate_with_all_empty_values_pkg_config() {
        let config = PkgConfig::default();
        match config.validate() {
            Err(validation_errors) => {
                let expected_errors = [
                    "field: spec_file cannot be empty",
                    "field: package_name cannot be empty",
                    "field: version_number cannot be empty",
                    "field: revision_number cannot be empty",
                    "field: homepage cannot be empty",
                    "field: codename cannot be empty",
                    "field: arch cannot be empty",
                    "field: pkg_builder_version cannot be empty",
                    "field: debcrafter_version cannot be empty",
                    "field: lintian_version cannot be empty",
                    "field: piuparts_version cannot be empty",
                    "field: autopkgtest_version cannot be empty",
                    "field: sbuild_version cannot be empty",
                ];
                assert_eq!(
                    validation_errors.len(),
                    expected_errors.len(),
                    "Number of errors is different"
                );
                for (actual, expected) in validation_errors.iter().zip(expected_errors.iter()) {
                    assert_eq!(actual.to_string(), *expected);
                }
            }
            Ok(_) => panic!("Validation should have failed."),
        }
    }
}
