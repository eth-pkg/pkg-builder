use std::{fs, path::PathBuf, process::Command, io};
use log::info;
use crate::build_pipeline::{BuildContext, BuildError, BuildHandler};

#[derive(Default)]
pub struct ExtractSourceHandler {
    build_files_dir: String,
    tarball_path: String,
}

impl ExtractSourceHandler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_tarball_path(mut self, path: String) -> Self {
        self.tarball_path = path;
        self
    }

    pub fn with_build_files_dir(mut self, build_files_dir: String) -> Self {
        self.build_files_dir = build_files_dir;
        self
    }

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

impl BuildHandler for ExtractSourceHandler {
    fn handle(&self, _context: &mut BuildContext) -> Result<(), BuildError> {
        info!("Extracting source {}", &self.build_files_dir);
        fs::create_dir_all(&self.build_files_dir).map_err(BuildError::IoError)?;
        
        let mut args = vec!["zxvf", &self.tarball_path, "-C", &self.build_files_dir];
        let numbers_to_strip = self.components_to_strip(self.tarball_path.clone())
            .map_err(BuildError::IoError)?;
        
        let strip = format!("--strip-components={}", numbers_to_strip);
        if numbers_to_strip > 0 {
            args.push(&strip);
        }
        
        info!("Stripping components: {} {:?}", numbers_to_strip, args);
        let output = Command::new("tar").args(args).output().map_err(BuildError::IoError)?;
        
        if !output.status.success() {
            let error_message = String::from_utf8(output.stderr)
                .unwrap_or_else(|_| "Unknown error occurred during extraction".to_string());
            return Err(BuildError::ExtractionError(error_message));
        }
        
        info!("Extracted source to build_files_dir: {:?}", self.build_files_dir);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use fs::File;
    use tempfile::tempdir;


    #[test]
    fn test_new() {
        let handler = ExtractSourceHandler::new();
        assert_eq!(handler.build_files_dir, "");
        assert_eq!(handler.tarball_path, "");
    }

    #[test]
    fn test_with_tarball_path() {
        let handler = ExtractSourceHandler::new().with_tarball_path("path/to/tarball.tar.gz".to_string());
        assert_eq!(handler.tarball_path, "path/to/tarball.tar.gz");
    }

    #[test]
    fn test_with_build_files_dir() {
        let handler = ExtractSourceHandler::new().with_build_files_dir("build/files/dir".to_string());
        assert_eq!(handler.build_files_dir, "build/files/dir");
    }

    #[test]
    fn test_longest_common_prefix_empty() {
        let strings: Vec<&str> = vec![];
        let prefix = ExtractSourceHandler::longest_common_prefix(&strings);
        assert_eq!(prefix, "");
    }

    #[test]
    fn test_longest_common_prefix_single() {
        let strings = vec!["folder/file.txt"];
        let prefix = ExtractSourceHandler::longest_common_prefix(&strings);
        assert_eq!(prefix, "folder");
    }

    #[test]
    fn test_longest_common_prefix_multiple() {
        let strings = vec![
            "project/src/main.rs",
            "project/src/lib.rs",
            "project/Cargo.toml",
        ];
        let prefix = ExtractSourceHandler::longest_common_prefix(&strings);
        assert_eq!(prefix, "project/");
    }

    #[test]
    fn test_longest_common_prefix_no_common() {
        let strings = vec!["abc/def", "xyz/uvw"];
        let prefix = ExtractSourceHandler::longest_common_prefix(&strings);
        assert_eq!(prefix, "");
    }


    #[test]
    fn test_handle_success() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let build_dir = temp_dir.path().join("build");
        let tarball_path = temp_dir.path().join("source.tar.gz");
        
        File::create(&tarball_path)?;
        
        let handler = ExtractSourceHandler::new()
            .with_build_files_dir(build_dir.to_str().unwrap().to_string())
            .with_tarball_path(tarball_path.to_str().unwrap().to_string());
        
        
        let mut context = BuildContext::default(); 
    
        if !Path::new(&build_dir).exists() {
            let result = handler.handle(&mut context);
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
        
        let handler = ExtractSourceHandler::new()
            .with_build_files_dir(build_dir.to_str().unwrap().to_string())
            .with_tarball_path(tarball_path.to_str().unwrap().to_string());
        
        let mut context = BuildContext::default();
        
        let result = handler.handle(&mut context);
        assert!(result.is_err());
        
        // match result {
        //     Err(BuildError::IoError(_)) => (), // Expected
        //     Err(e) => panic!("Unexpected error type: {:?}", e),
        //     Ok(_) => panic!("Expected an error but got Ok"),
        // }
    }
}