use std::fs;
use std::path::Path;
use std::process::Command;

use log::info;
use tempfile::tempdir;

use crate::ToolError;

/// Debcrafter command wrapper.
pub struct Debcrafter<'a> {
    pub version: &'a str,
}

impl Debcrafter<'_> {
    fn binary_name(&self) -> String {
        format!("debcrafter_{}", self.version)
    }

    pub fn check_installed(&self) -> Result<(), ToolError> {
        let binary = self.binary_name();
        let output = Command::new("which")
            .arg(&binary)
            .output()
            .map_err(|e| ToolError::NotFound {
                tool: binary.clone(),
                source: e,
            })?;

        if !output.status.success() {
            return Err(ToolError::Other {
                tool: binary,
                message: "not installed".to_string(),
            });
        }
        Ok(())
    }

    pub fn check_dpkg_parsechangelog(&self) -> Result<(), ToolError> {
        let output = Command::new("which")
            .arg("dpkg-parsechangelog")
            .output()
            .map_err(|e| ToolError::NotFound {
                tool: "dpkg-parsechangelog".to_string(),
                source: e,
            })?;

        if !output.status.success() {
            return Err(ToolError::Other {
                tool: "dpkg-parsechangelog".to_string(),
                message: "not installed, please install it".to_string(),
            });
        }
        Ok(())
    }

    pub fn create_debian_dir(
        &self,
        spec_file: &Path,
        target_dir: &Path,
    ) -> Result<(), ToolError> {
        let debcrafter_dir = tempdir()?;

        let spec_file_path = fs::canonicalize(spec_file).map_err(|_| ToolError::Other {
            tool: self.binary_name(),
            message: format!("{:?} spec_file doesn't exist", spec_file),
        })?;

        let spec_dir = spec_file_path.parent().ok_or_else(|| ToolError::Other {
            tool: self.binary_name(),
            message: "Invalid specification file path".to_string(),
        })?;

        let spec_file_name = spec_file_path.file_name().ok_or_else(|| ToolError::Other {
            tool: self.binary_name(),
            message: "Invalid specification file name".to_string(),
        })?;

        info!("Spec directory: {:?}", spec_dir);
        info!("Spec file: {:?}", spec_file_name);
        info!("Debcrafter directory: {:?}", debcrafter_dir.path());

        let output = Command::new(self.binary_name())
            .arg(spec_file_name)
            .current_dir(spec_dir)
            .arg(debcrafter_dir.path())
            .output()
            .map_err(|e| ToolError::NotFound {
                tool: self.binary_name(),
                source: e,
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ToolError::Other {
                tool: self.binary_name(),
                message: format!("Debcrafter execution failed: {}", stderr),
            });
        }

        // Find the first directory in the debcrafter output
        let first_dir = fs::read_dir(debcrafter_dir.path())?
            .filter_map(|e| e.ok())
            .find(|e| e.path().is_dir())
            .map(|e| e.path())
            .ok_or_else(|| ToolError::Other {
                tool: self.binary_name(),
                message: "No output directory found from debcrafter".to_string(),
            })?;

        let tmp_debian_dir = first_dir.join("debian");
        let dest_dir = target_dir.join("debian");

        copy_dir_recursive(&tmp_debian_dir, &dest_dir)?;

        Ok(())
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), ToolError> {
    if !src.is_dir() {
        return Err(ToolError::Other {
            tool: "debcrafter".to_string(),
            message: format!("Source path is not a directory: {}", src.display()),
        });
    }

    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
