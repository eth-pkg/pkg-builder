use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use log::info;

use crate::build_pipeline::{BuildContext, BuildError, BuildStep};

#[derive(Default)]
pub struct PatchSource {
}

impl PatchSource {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn patch_quilt(build_files_dir: &String) -> Result<(), BuildError> {
        let debian_source_format_path = format!("{}/debian/source/format", build_files_dir);
        info!(
            "Setting up quilt format for patching. Debian source format path: {}",
            debian_source_format_path
        );
        let debian_source_dir = PathBuf::from(&build_files_dir).join("debian/source");
        if !debian_source_dir.exists() {
            fs::create_dir_all(&debian_source_dir)?;
            info!(
                "Created debian/source directory at: {:?}",
                debian_source_dir
            );
        }

        if !Path::new(&debian_source_format_path).exists() {
            fs::write(&debian_source_format_path, "3.0 (quilt)\n")?;
            info!(
                "Quilt format file created at: {}",
                debian_source_format_path
            );
        } else {
            info!(
                "Quilt format file already exists at: {}",
                debian_source_format_path
            );
        }
        Ok(())
    }

    pub fn patch_pc_dir(build_files_dir: &String) -> Result<(), BuildError> {
        let pc_version_path = format!("{}/.pc/.version", &build_files_dir);
        info!("Creating necessary directories for patching");
        fs::create_dir_all(format!("{}/.pc", &build_files_dir))?;
        let mut pc_version_file = fs::File::create(pc_version_path)?;
        writeln!(pc_version_file, "2")?;
        Ok(())
    }

    pub fn patch_standards_version(
        build_files_dir: &String,
        homepage: &String,
    ) -> Result<(), BuildError> {
        let debian_control_path = format!("{}/debian/control", build_files_dir);
        info!(
            "Adding Standards-Version to the control file. Debian control path: {}",
            debian_control_path
        );
        let input_file = fs::File::open(&debian_control_path)?;
        let reader = BufReader::new(input_file);

        let original_content: Vec<String> = reader.lines().map(|line| line.unwrap()).collect();
        let has_standards_version = original_content
            .iter()
            .any(|line| line.starts_with("Standards-Version"));
        let standards_version_line = "Standards-Version: 4.5.1";
        let homepage_line = format!("Homepage: {}", homepage);
        if !has_standards_version {
            let mut insert_index = 0;
            for (i, line) in original_content.iter().enumerate() {
                if line.starts_with("Priority:") {
                    insert_index = i + 1;
                    break;
                }
            }

            let mut updated_content = original_content.clone();
            updated_content.insert(insert_index, standards_version_line.to_string());
            updated_content.insert(insert_index + 1, homepage_line.to_string());

            let mut output_file = fs::File::create(&debian_control_path)?;
            for line in updated_content {
                writeln!(output_file, "{}", line)?;
            }

            info!("Standards-Version added to the control file.");
        } else {
            info!("Standards-Version already exists in the control file. No changes made.");
        }
        Ok(())
    }

    pub fn copy_src_dir(build_files_dir: &String, src_dir: &String) -> Result<(), BuildError> {
        let src_dir_path = Path::new(src_dir);
        if src_dir_path.exists() {
            Self::copy_directory_recursive(Path::new(src_dir), Path::new(&build_files_dir))
                .map_err(|err| BuildError::CopyDirectory(err.to_string()))?;
        }
        Ok(())
    }

    pub fn patch_rules_permission(build_files_dir: &str) -> Result<(), BuildError> {
        info!(
            "Adding executable permission for {}/debian/rules",
            build_files_dir
        );

        let debian_rules = format!("{}/debian/rules", build_files_dir);
        let mut permissions = fs::metadata(debian_rules.clone())
            .map_err(|_| BuildError::RulesPermissionGet)?
            .permissions();
        permissions.set_mode(permissions.mode() | 0o111);
        fs::set_permissions(debian_rules, permissions)
            .map_err(|_| BuildError::RulesPermissionSet)?;
        Ok(())
    }

    fn copy_directory_recursive(src_dir: &Path, dest_dir: &Path) -> Result<(), std::io::Error> {
        if !src_dir.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Source directory {:?} does not exist", src_dir),
            ));
        }

        if !dest_dir.exists() {
            fs::create_dir_all(dest_dir)?;
        }

        for entry in fs::read_dir(src_dir)? {
            let entry = entry?;
            let entry_path = entry.path();
            let file_name = entry.file_name();

            let dest_path = dest_dir.join(&file_name);

            if entry_path.is_dir() {
                Self::copy_directory_recursive(&entry_path, &dest_path)?;
            } else {
                if let Err(e) = fs::copy(&entry_path, &dest_path) {
                    eprintln!(
                        "Failed to copy file from {:?} to {:?}: {}",
                        entry_path, dest_path, e
                    );
                    return Err(e);
                }
            }
        }

        Ok(())
    }
}

impl BuildStep for PatchSource {
    fn step(&self, context: &mut BuildContext) -> Result<(), BuildError> {
        // Patch quilt
        Self::patch_quilt(&context.build_files_dir)?;

        // Patch .pc dir setup
        Self::patch_pc_dir(&context.build_files_dir)?;

        // Patch .pc patch version number
        Self::patch_standards_version(&context.build_files_dir, &context.homepage)?;

        // Only copy if src dir exists
        Self::copy_src_dir(&context.build_files_dir, &context.src_dir)?;

        Self::patch_rules_permission(&context.build_files_dir)?;

        info!("Patching finished successfully!");
        Ok(()) // Added missing return value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn patch_rules_permission_handles_nonexistent_directory() {

        let result = PatchSource::patch_rules_permission("/nonexistent/dir");

        assert!(result.is_err());
    }

    #[test]
    fn patch_quilt_creates_source_dir_and_format_file() -> Result<(), Box<dyn std::error::Error>> {

        let temp_dir = tempdir()?;
        let build_files_dir = temp_dir.path().to_str().unwrap().to_string();

        PatchSource::patch_quilt(&build_files_dir)?;

        let debian_source_dir = temp_dir.path().join("debian/source");
        assert!(debian_source_dir.exists());

        let debian_source_format_path = temp_dir.path().join("debian/source/format");
        let format_content = fs::read_to_string(debian_source_format_path)?;
        assert_eq!(format_content, "3.0 (quilt)\n");

        Ok(())
    }

    #[test]
    fn patch_quilt_skips_creation_if_already_exists() -> Result<(), Box<dyn std::error::Error>> {

        let temp_dir = tempdir()?;
        let temp_dir = temp_dir.path();
        let build_files_dir = temp_dir.to_str().unwrap().to_string();

        fs::create_dir_all(temp_dir.join("debian/source")).expect("Failed to create dir for test.");
        File::create(temp_dir.join("debian/source/format")).expect("Failed to create file.");

        let result = PatchSource::patch_quilt(&build_files_dir);
        assert!(result.is_ok());

        let entries: Vec<_> = fs::read_dir(temp_dir)?.collect();
        assert_eq!(entries.len(), 1);

        Ok(())
    }


    #[test]
    fn patch_rules_permission_adds_exec_permission() -> Result<(), Box<dyn std::error::Error>> {

        let temp_dir = tempdir()?;
        let rules_path = temp_dir.path().join("debian/rules");
        fs::create_dir_all(temp_dir.path().join("debian")).expect("Could not create dir");
        File::create(&rules_path)?;

        PatchSource::patch_rules_permission(temp_dir.path().to_str().unwrap())?;

        let permissions = fs::metadata(&rules_path)?.permissions();
        assert_ne!(permissions.mode() & 0o111, 0);

        Ok(())
    }


}