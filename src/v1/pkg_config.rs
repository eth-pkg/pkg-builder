use std::fs;
use std::path::Path;
use eyre::{eyre, Report, Result};
use serde::{Deserialize, Deserializer};
use std::str::FromStr;
use serde::de::DeserializeOwned;

pub fn deserialize_option_empty_string<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        T: FromStr,
        T::Err: std::fmt::Display,
        D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        T::from_str(&s).map(Some).map_err(serde::de::Error::custom)
    }
}

pub trait Validation {
    fn validate(&self) -> Result<(), Vec<Report>>;
}

pub fn validate_not_empty(name: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(eyre!("field: {} cannot be empty", name));
    }
    Ok(())
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
pub struct RustConfig {
    pub rust_version: String,
    pub rust_binary_url: String,
    pub rust_binary_gpg_asc: String,
}

impl Validation for RustConfig {
    fn validate(&self) -> Result<(), Vec<Report>> {
        let mut errors = Vec::new();

        if let Err(err) = validate_not_empty("rust_version", &self.rust_version) {
            errors.push(err);
        }

        if let Err(err) = validate_not_empty("rust_binary_url", &self.rust_binary_url) {
            errors.push(err);
        }

        if let Err(err) = validate_not_empty("rust_binary_gpg_asc", &self.rust_binary_gpg_asc) {
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
pub struct GoConfig {
    pub go_version: String,
    pub go_binary_url: String,
    pub go_binary_checksum: String,
}

impl Validation for GoConfig {
    fn validate(&self) -> Result<(), Vec<Report>> {
        let mut errors = Vec::new();

        if let Err(err) = validate_not_empty("go_version", &self.go_version) {
            errors.push(err);
        }

        if let Err(err) = validate_not_empty("go_binary_url", &self.go_binary_url) {
            errors.push(err);
        }

        if let Err(err) = validate_not_empty("go_binary_checksum", &self.go_binary_checksum) {
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
pub struct JavascriptConfig {
    pub node_version: String,
    pub node_binary_url: String,
    pub node_binary_checksum: String,
    pub yarn_version: Option<String>,
}

impl Validation for JavascriptConfig {
    fn validate(&self) -> Result<(), Vec<Report>> {
        let mut errors = Vec::new();

        if let Err(err) = validate_not_empty("node_version", &self.node_version) {
            errors.push(err);
        }
        if let Err(err) = validate_not_empty("node_binary_url", &self.node_binary_url) {
            errors.push(err);
        }
        if let Err(err) = validate_not_empty("node_binary_checksum", &self.node_binary_checksum) {
            errors.push(err);
        }
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

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
pub struct GradleConfig {
    pub gradle_version: String,
    pub gradle_binary_url: String,
    pub gradle_binary_checksum: String,
}

impl Validation for GradleConfig {
    fn validate(&self) -> Result<(), Vec<Report>> {
        let mut errors = Vec::new();

        if let Err(err) = validate_not_empty("gradle_version", &self.gradle_version) {
            errors.push(err);
        }
        if let Err(err) = validate_not_empty("gradle_binary_url", &self.gradle_binary_url) {
            errors.push(err);
        }
        if let Err(err) = validate_not_empty("gradle_binary_checksum", &self.gradle_binary_checksum) {
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
pub struct JavaConfig {
    pub is_oracle: bool,
    pub jdk_version: String,
    pub jdk_binary_url: String,
    pub jdk_binary_checksum: String,
    pub gradle: Option<GradleConfig>,
}

impl Validation for JavaConfig {
    fn validate(&self) -> Result<(), Vec<Report>> {
        let mut errors = Vec::new();

        if let Err(err) = validate_not_empty("jdk_version", &self.jdk_version) {
            errors.push(err);
        }
        if let Err(err) = validate_not_empty("jdk_binary_url", &self.jdk_binary_url) {
            errors.push(err);
        }
        if let Err(err) = validate_not_empty("jdk_binary_checksum", &self.jdk_binary_checksum) {
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
pub struct DotnetPackage {
    pub name: String,
    pub hash: String,
    pub url: String,
}

impl Validation for DotnetPackage {
    fn validate(&self) -> Result<(), Vec<Report>> {
        let mut errors = Vec::new();

        if let Err(err) = validate_not_empty("name", &self.name) {
            errors.push(err);
        }

        if let Err(err) = validate_not_empty("hash", &self.hash) {
            errors.push(err);
        }

        if let Err(err) = validate_not_empty("url", &self.url) {
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
pub struct DotnetConfig {
    pub use_backup_version: bool,
    pub dotnet_packages: Vec<DotnetPackage>,
}

impl Validation for DotnetConfig {
    fn validate(&self) -> Result<(), Vec<Report>> {
        let errors = Vec::new();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
pub struct NimConfig {
    pub nim_version: String,
    pub nim_binary_url: String,
    pub nim_version_checksum: String,
}

impl Validation for NimConfig {
    fn validate(&self) -> Result<(), Vec<Report>> {
        let mut errors = Vec::new();

        if let Err(err) = validate_not_empty("nim_version", &self.nim_version) {
            errors.push(err);
        }
        if let Err(err) = validate_not_empty("nim_binary_url", &self.nim_binary_url) {
            errors.push(err);
        }
        if let Err(err) = validate_not_empty("nim_version_checksum", &self.nim_version_checksum) {
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

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
pub struct DefaultPackageTypeConfig {
    pub tarball_url: String,
    pub tarball_hash: Option<String>,
    pub language_env: LanguageEnv,
}

impl Validation for DefaultPackageTypeConfig {
    fn validate(&self) -> Result<(), Vec<Report>> {
        let mut errors = Vec::new();

        if let Err(err) = validate_not_empty("tarball_url", &self.tarball_url) {
            errors.push(err);
        }
        if let Some(value) = &self.tarball_hash {
            if let Err(err) = validate_not_empty("tarball_hash", value) {
                errors.push(err);
            }
        }
        let language_errors = self.language_env.validate();

        if let Err(mut language_errors) = language_errors {
            errors.append(&mut language_errors);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Clone, Default)]
pub struct SubModule {
    pub commit: String,
    pub path: String,
}

impl Validation for SubModule {
    fn validate(&self) -> Result<(), Vec<Report>> {
        let mut errors = Vec::new();

        if let Err(err) = validate_not_empty("commit", &self.commit) {
            errors.push(err);
        }

        if let Err(err) = validate_not_empty("path", &self.path) {
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
pub struct GitPackageTypeConfig {
    pub git_tag: String,
    pub git_url: String,
    pub submodules: Vec<SubModule>,
    pub language_env: LanguageEnv,
}

impl Validation for GitPackageTypeConfig {
    fn validate(&self) -> Result<(), Vec<Report>> {
        let mut errors = Vec::new();

        if let Err(err) = validate_not_empty("git_tag", &self.git_tag) {
            errors.push(err);
        }

        if let Err(err) = validate_not_empty("git_url", &self.git_url) {
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
#[serde(tag = "package_type", rename_all = "lowercase")]
pub enum PackageType {
    Default(DefaultPackageTypeConfig),
    Git(GitPackageTypeConfig),
    #[default]
    Virtual,
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

#[derive(Debug, Deserialize, PartialEq, Default, Clone)]
pub struct PackageFields {
    pub spec_file: String,
    pub package_name: String,
    pub version_number: String,
    pub revision_number: String,
    pub homepage: String,
}

impl Validation for PackageFields {
    fn validate(&self) -> Result<(), Vec<Report>> {
        let mut errors = Vec::new();

        if let Err(err) = validate_not_empty("spec_file", &self.spec_file) {
            errors.push(err);
        }
        if let Err(err) = validate_not_empty("package_name", &self.package_name) {
            errors.push(err);
        }
        if let Err(err) = validate_not_empty("version_number", &self.version_number) {
            errors.push(err);
        }
        if let Err(err) = validate_not_empty("revision_number", &self.revision_number) {
            errors.push(err);
        }
        if let Err(err) = validate_not_empty("homepage", &self.homepage) {
            errors.push(err);
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Default, Clone)]
pub struct BuildEnv {
    pub codename: String,
    pub arch: String,
    pub pkg_builder_version: String,
    pub debcrafter_version: String,
    pub sbuild_cache_dir: Option<String>,
    pub docker: Option<bool>,
    pub run_lintian: Option<bool>,
    pub run_piuparts: Option<bool>,
    pub run_autopkgtest: Option<bool>,
    pub lintian_version: String,
    pub piuparts_version: String,
    pub autopkgtest_version: String,
    pub sbuild_version: String,
    #[serde(deserialize_with = "deserialize_option_empty_string")]
    pub workdir: Option<String>,
}

impl Validation for BuildEnv {
    fn validate(&self) -> Result<(), Vec<Report>> {
        let mut errors = Vec::new();

        if let Err(err) = validate_not_empty("codename", &self.codename) {
            errors.push(err);
        }
        if let Err(err) = validate_not_empty("arch", &self.arch) {
            errors.push(err);
        }
        if let Err(err) = validate_not_empty("pkg_builder_version", &self.pkg_builder_version) {
            errors.push(err);
        }

        if let Err(err) = validate_not_empty("debcrafter_version", &self.debcrafter_version) {
            errors.push(err);
        }
        if let Err(err) = validate_not_empty("lintian_version", &self.lintian_version) {
            errors.push(err);
        }
        if let Err(err) = validate_not_empty("piuparts_version", &self.piuparts_version) {
            errors.push(err);
        }
        if let Err(err) = validate_not_empty("autopkgtest_version", &self.autopkgtest_version) {
            errors.push(err);
        }
        if let Err(err) = validate_not_empty("sbuild_version", &self.sbuild_version) {
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
pub struct PkgConfig {
    pub package_fields: PackageFields,
    pub package_type: PackageType,
    pub build_env: BuildEnv,
}

impl Validation for PkgConfig {
    fn validate(&self) -> Result<(), Vec<Report>> {
        let mut errors = Vec::new();
        let package_field_errors = self.package_fields.validate();
        let package_type_errors = self.package_type.validate();
        let build_env_errors = self.build_env.validate();
        if let Err(mut package_field_errors) = package_field_errors {
            errors.append(&mut package_field_errors);
        }

        if let Err(mut package_type_errors) = package_type_errors {
            errors.append(&mut package_type_errors);
        }

        if let Err(mut build_env_errors) = build_env_errors {
            errors.append(&mut build_env_errors);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
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

    let config: T =
        parse(&toml_content)?;


    Ok(config)
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
pkg_builder_version="0.2.9"
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
                pkg_builder_version: "0.2.9".to_string(),
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
                let expected_errors = [
                    "field: tarball_url cannot be empty",
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
