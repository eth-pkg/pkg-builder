use crate::misc::build_pipeline::{BuildContext, BuildError, BuildStep};
use log::info;
use std::{fs, io, path::PathBuf, process::Command};

#[derive(Default)]
pub struct ExtractSource {
    build_files_dir: PathBuf,
    tarball_path: PathBuf,
}

impl From<BuildContext> for ExtractSource {
    fn from(context: BuildContext) -> Self {
        ExtractSource {
            build_files_dir: context.build_files_dir.clone(),
            tarball_path: context.tarball_path.clone(),
            // debcrafter_version: context.debcrafter_version.clone(),
            // spec_file: context.spec_file.clone(),
        }
    }
}

impl ExtractSource {
    fn longest_common_prefix(strings: &[&str]) -> String {
        if strings.is_empty() {
            return String::new();
        }
        if strings.len() == 1 {
            let mut path_buf = PathBuf::from(strings[0]);
            path_buf.pop();
            let common_prefix = path_buf.to_string_lossy().to_string();
            return common_prefix;
        }
        let first_string = &strings[0];
        let mut prefix = String::new();
        'outer: for (i, c) in first_string.char_indices() {
            for string in &strings[1..] {
                if let Some(next_char) = string.chars().nth(i) {
                    if next_char != c {
                        break 'outer;
                    }
                } else {
                    break 'outer;
                }
            }
            prefix.push(c);
        }
        prefix
    }

    fn components_to_strip(&self, tar_gz_file: String) -> Result<usize, io::Error> {
        let output = Command::new("tar")
            .arg("--list")
            .arg("-z")
            .arg("-f")
            .arg(tar_gz_file)
            .output()?;
        let output_str = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = output_str.lines().filter(|l| !l.ends_with('/')).collect();
        let common_prefix = Self::longest_common_prefix(&lines);
        let components_to_strip = common_prefix.split('/').filter(|&x| !x.is_empty()).count();
        Ok(components_to_strip)
    }
}

impl BuildStep for ExtractSource {
    fn step(&self) -> Result<(), BuildError> {
        info!("Extracting source {:?}", &self.build_files_dir);
        fs::create_dir_all(&self.build_files_dir).map_err(BuildError::IoError)?;

        let mut args = vec![
            "zxvf".to_string(),
            self.tarball_path.display().to_string(),
            "-C".to_string(),
            self.build_files_dir.display().to_string(),
        ];
        let numbers_to_strip = self
            .components_to_strip(self.tarball_path.display().to_string())
            .map_err(BuildError::IoError)?;

        let strip = format!("--strip-components={}", numbers_to_strip);
        if numbers_to_strip > 0 {
            args.push(strip);
        }

        info!("Stripping components: {} {:?}", numbers_to_strip, args);
        let output = Command::new("tar")
            .args(args)
            .output()
            .map_err(BuildError::IoError)?;

        if !output.status.success() {
            let error_message = String::from_utf8(output.stderr)
                .unwrap_or_else(|_| "Unknown error occurred during extraction".to_string());
            return Err(BuildError::ExtractionError(error_message));
        }

        info!(
            "Extracted source to build_files_dir: {:?}",
            self.build_files_dir
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fs::File;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn test_longest_common_prefix_empty() {
        let strings: Vec<&str> = vec![];
        let prefix = ExtractSource::longest_common_prefix(&strings);
        assert_eq!(prefix, "");
    }

    #[test]
    fn test_longest_common_prefix_single() {
        let strings = vec!["folder/file.txt"];
        let prefix = ExtractSource::longest_common_prefix(&strings);
        assert_eq!(prefix, "folder");
    }

    #[test]
    fn test_longest_common_prefix_multiple() {
        let strings = vec![
            "project/src/main.rs",
            "project/src/lib.rs",
            "project/Cargo.toml",
        ];
        let prefix = ExtractSource::longest_common_prefix(&strings);
        assert_eq!(prefix, "project/");
    }

    #[test]
    fn test_longest_common_prefix_no_common() {
        let strings = vec!["abc/def", "xyz/uvw"];
        let prefix = ExtractSource::longest_common_prefix(&strings);
        assert_eq!(prefix, "");
    }

    #[test]
    fn test_handle_success() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let build_dir = temp_dir.path().join("build");
        let tarball_path = temp_dir.path().join("source.tar.gz");

        File::create(&tarball_path)?;

        let mut context = BuildContext::default();
        context.build_files_dir = build_dir.clone();
        context.tarball_path = tarball_path;

        let handler = ExtractSource::from(context);

        if !Path::new(&build_dir).exists() {
            let result = handler.step();
            assert!(result.is_err());
        }

        assert!(Path::new(&build_dir).exists());

        Ok(())
    }

    #[test]
    fn test_handle_extraction_error() {
        let temp_dir = tempdir().unwrap();
        let build_dir = temp_dir.path().join("build");
        let tarball_path = temp_dir.path().join("nonexistent.tar.gz");

        let mut context = BuildContext::default();
        context.build_files_dir = build_dir;
        context.tarball_path = tarball_path;
        let handler = ExtractSource::from(context);

        let result = handler.step();
        assert!(result.is_err());

        // match result {
        //     Err(BuildError::IoError(_)) => (), // Expected
        //     Err(e) => panic!("Unexpected error type: {:?}", e),
        //     Ok(_) => panic!("Expected an error but got Ok"),
        // }
    }

    #[test]
    fn test_extract_source() {
        let package_name = "test_package";
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let temp_dir = temp_dir.path();
        let tarball_path: PathBuf = PathBuf::from("tests/misc/test_package.tar.gz");

        let build_files_dir = temp_dir.join(package_name);

        assert!(tarball_path.exists());

        let mut context = BuildContext::default();
        context.build_files_dir = build_files_dir.clone();
        context.tarball_path = tarball_path.to_str().unwrap().into();
        let handler = ExtractSource::from(context);

        let result = handler.step();

        assert!(result.is_ok(), "{:?}", result);
        assert!(Path::new(&build_files_dir).exists());

        let test_file_path = PathBuf::from(build_files_dir.clone()).join("empty_file.txt");

        assert!(
            test_file_path.exists(),
            "Empty file not found after extraction"
        );
    }
}
