use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use log::info;

use crate::build_pipeline::{BuildContext, BuildError, BuildHandler};

#[derive(Default)]
pub struct PatchSourceHandle {
    build_files_dir: String,
    homepage: String,
    src_dir: String,
}

impl PatchSourceHandle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_build_files_dir(mut self, build_files_dir: String) -> Self {
        self.build_files_dir = build_files_dir;
        self
    }

    pub fn with_homepage(mut self, homepage: String) -> Self {
        self.homepage = homepage;
        self
    }
    pub fn with_src_dir(mut self, src_dir: String) -> Self {
        self.src_dir = src_dir;
        self
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

impl BuildHandler for PatchSourceHandle {
    fn handle(&self, _context: &mut BuildContext) -> Result<(), BuildError> {
        // Patch quilt
        Self::patch_quilt(&self.build_files_dir)?;

        // Patch .pc dir setup
        Self::patch_pc_dir(&self.build_files_dir)?;

        // Patch .pc patch version number
        Self::patch_standards_version(&self.build_files_dir, &self.homepage)?;

        // Only copy if src dir exists
        Self::copy_src_dir(&self.build_files_dir, &self.src_dir)?;

        Self::patch_rules_permission(&self.build_files_dir)?;

        info!("Patching finished successfully!");
        Ok(()) // Added missing return value
    }
}
