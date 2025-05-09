use serde::Deserialize;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Deserialize)]
pub struct Dependency {
    pub url: String,
    pub commit_hash: String,
    pub binary_name: String,
    pub original_binary_name: String,
}

#[derive(Debug, Deserialize)]
pub struct Dependencies {
    pub binaries: Vec<Dependency>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub dependencies: Dependencies,
}

impl Config {
    pub fn load_config() -> Result<Config, ConfigError> {
        let toml_str = include_str!("dependencies.toml");
        let config = toml::from_str(toml_str).map_err(|e| ConfigError::TomlParseError(e))?;
        Ok(config)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Failed to parse TOML: {0}")]
    TomlParseError(#[from] toml::de::Error),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Other error: {0}")]
    Other(String),
}

#[derive(thiserror::Error, Debug)]
pub enum CargoError {
    #[error("Non-zero exit status: {0}")]
    StatusError(String),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Other error: {0}")]
    Other(String),
}

pub struct Cargo<'a> {
    repo_url: &'a str,
    commit_hash: &'a str,
}

impl<'a> Cargo<'a> {
    // fn install_from_crates_io<P: AsRef<Path>>(&self, bin_dir: P) -> Result<(), CargoError> {
    //     todo!()
    // }

    fn install_from_git<P: AsRef<Path>>(&self, bin_dir: P) -> Result<(), CargoError> {
        let output = Command::new("cargo")
            .arg("install")
            .arg("--git")
            .arg(self.repo_url)
            .arg("--rev")
            .arg(self.commit_hash)
            .arg("--root")
            .arg(bin_dir.as_ref())
            .output()
            .map_err(|e| CargoError::IoError(e))?;

        if output.status.success() {
            Ok(())
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(CargoError::StatusError(format!(
                "Cargo install failed with exit code {:?}\nstdout: {}\nstderr: {}",
                output.status.code(),
                stdout,
                stderr
            )))
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DependencyError {
    #[error(transparent)]
    CargoError(#[from] CargoError),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Other error: {0}")]
    Other(String),
}

impl Dependency {
    fn install_binary(&self, bin_dir: String) -> Result<(), DependencyError> {
        fs::create_dir_all(&bin_dir).map_err(|e| DependencyError::IoError(e))?;

        let binary_path = Path::new(&bin_dir.clone()).join(&self.binary_name);
        let bin_dir = Path::new(&bin_dir);
        let original_binary_name_path = bin_dir.join("bin").join(&self.original_binary_name);

        if !binary_path.exists() {
            let cargo = Cargo {
                repo_url: &self.url,
                commit_hash: &self.commit_hash,
            };

            cargo.install_from_git(bin_dir)?;

            fs::rename(original_binary_name_path, binary_path)
                .map_err(|e| DependencyError::IoError(e))?;
        }

        Ok(())
    }
}

pub fn main() {
    let bin_dir = format!("{}/bin_dependencies", ".");
    let config = Config::load_config().expect("Could not load config");
    let dependencies = config.dependencies.binaries;

    dependencies.iter().for_each(|d| {
        let _ = d.install_binary(bin_dir.clone());
    });

    println!("cargo:rerun-if-changed=build.rs");
}
