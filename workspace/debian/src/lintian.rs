use std::path::Path;
use log::info;
use eyre::Result;
use super::execute::{execute_command, Execute};

/// Represents a wrapper for the Lintian Debian package checker tool.
/// 
/// Lintian is used to check Debian packages for compliance with the Debian policy
/// and other quality assurance checks. This struct provides a builder pattern
/// interface to configure and execute Lintian commands.
/// 
/// # Examples
/// 
/// ```
/// use debian::lintian::Lintian;
/// use debian::execute::Execute;
/// let result = Lintian::new()
///     .info()
///     .suppress_tag("malformed-deb-archive")
///     .changes_file("package.changes")
///     .execute();
/// ```
#[derive(Debug, Clone, Default)]
pub struct Lintian {
    /// Tags to suppress during linting
    suppress_tags: Vec<String>,
    /// Whether to show informational tags
    show_info: bool,
    /// Whether to show extended information
    extended_info: bool,
    /// Path to the changes file to check
    changes_file_path: Option<String>,
    /// Maximum number of tags to display
    tag_display_limit: Option<u32>,
    /// Severity levels to fail on (warning, error)
    fail_on: Vec<String>,
    /// Ubuntu/Debian codename for version-specific checks
    codename: Option<String>,
}

impl Lintian {
    /// Creates a new Lintian instance with default configuration.
    ///
    /// This method initializes a Lintian object with empty or default values,
    /// ready to be configured using the builder pattern.
    ///
    /// # Returns
    ///
    /// A new instance of `Lintian`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Suppresses the specified lintian tag.
    ///
    /// This will add the given tag to the list of tags that Lintian
    /// should not report.
    ///
    /// # Arguments
    ///
    /// * `tag` - The tag to suppress.
    ///
    /// # Returns
    ///
    /// Self with the tag added to the suppress list.
    pub fn suppress_tag<S: AsRef<str>>(mut self, tag: S) -> Self {
        self.suppress_tags.push(tag.as_ref().to_string());
        self
    }

    /// Enables display of informational tags.
    ///
    /// Corresponds to the `-i` flag in the Lintian command.
    ///
    /// # Returns
    ///
    /// Self with the info flag enabled.
    pub fn info(mut self) -> Self {
        self.show_info = true;
        self
    }

    /// Enables display of extended information.
    ///
    /// Corresponds to the `-I` flag in the Lintian command.
    ///
    /// # Returns
    ///
    /// Self with the extended info flag enabled.
    pub fn extended_info(mut self) -> Self {
        self.extended_info = true;
        self
    }

    /// Specifies a changes file to check.
    ///
    /// # Arguments
    ///
    /// * `file` - Path to the changes file.
    ///
    /// # Returns
    ///
    /// Self with the changes file path set.
    pub fn changes_file<P: AsRef<Path>>(mut self, file: P) -> Self {
        self.changes_file_path = Some(format!("{:?}", file.as_ref()));
        self
    }

    /// Sets the maximum number of tags to display.
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of tags to display.
    ///
    /// # Returns
    ///
    /// Self with the tag display limit set.
    pub fn tag_display_limit(mut self, limit: u32) -> Self {
        self.tag_display_limit = Some(limit);
        self
    }

    /// Configures Lintian to fail on warnings.
    /// 
    /// Can be combined with `fail_on_error()`.
    ///
    /// # Returns
    ///
    /// Self with fail-on-warning configured.
    pub fn fail_on_warning(mut self) -> Self {
        self.fail_on.push("warning".to_string());
        self
    }

    /// Configures Lintian to fail on errors.
    /// 
    /// Can be combined with `fail_on_warning()`.
    ///
    /// # Returns
    ///
    /// Self with fail-on-error configured.
    pub fn fail_on_error(mut self) -> Self {
        self.fail_on.push("error".to_string());
        self
    }

    /// Configures Lintian for a specific Ubuntu/Debian codename.
    ///
    /// For certain codenames (jammy, noble), this automatically
    /// suppresses the "malformed-deb-archive" tag.
    ///
    /// # Arguments
    ///
    /// * `codename` - Ubuntu/Debian codename (e.g., "jammy", "noble").
    ///
    /// # Returns
    ///
    /// Self with codename-specific configuration.
    pub fn with_codename<S: AsRef<str>>(mut self, codename: S) -> Self {
        let codename_str = codename.as_ref().to_string();
        self.codename = Some(codename_str.clone());
        
        if codename_str == "jammy" || codename_str == "noble" {
            self.suppress_tags.push("malformed-deb-archive".to_string());
        }
        self
    }
    
    /// Builds the command arguments from struct fields.
    ///
    /// This method constructs a vector of command-line arguments
    /// based on the current configuration of the Lintian object.
    ///
    /// # Returns
    ///
    /// A vector of strings representing the command-line arguments.
    fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        
        // Add suppress tags
        for tag in &self.suppress_tags {
            args.push("--suppress-tags".to_string());
            args.push(tag.clone());
        }
        
        // Add info flag
        if self.show_info {
            args.push("-i".to_string());
        }
        
        // Add extended info flag
        if self.extended_info {
            args.push("-I".to_string());
        }
        
        // Add changes file
        if let Some(file) = &self.changes_file_path {
            args.push(file.clone());
        }
        
        // Add tag display limit
        if let Some(limit) = self.tag_display_limit {
            args.push(format!("--tag-display-limit={}", limit));
        }
        
        // Add fail-on options
        for level in &self.fail_on {
            args.push(format!("--fail-on={}", level));
        }
        
        args
    }
}

impl Execute for Lintian {
    /// Executes the lintian command with the configured options.
    ///
    /// This method builds the arguments based on the current configuration
    /// and executes the lintian command.
    ///
    /// # Returns
    ///
    /// A result indicating success or an error.
    fn execute(&self) -> Result<()> {
        let args = self.build_args();
        info!("Running: lintian {}", args.join(" "));
        execute_command("lintian", &args, None)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;


    #[test]
    fn test_new() {
        let lintian = Lintian::new();
        assert!(lintian.suppress_tags.is_empty());
        assert!(!lintian.show_info);
        assert!(!lintian.extended_info);
        assert!(lintian.changes_file_path.is_none());
        assert!(lintian.tag_display_limit.is_none());
        assert!(lintian.fail_on.is_empty());
        assert!(lintian.codename.is_none());
    }

    #[test]
    fn test_suppress_tag() {
        let lintian = Lintian::new().suppress_tag("test-tag");
        assert_eq!(lintian.suppress_tags, vec!["test-tag"]);
    }

    #[test]
    fn test_info() {
        let lintian = Lintian::new().info();
        assert!(lintian.show_info);
    }

    #[test]
    fn test_extended_info() {
        let lintian = Lintian::new().extended_info();
        assert!(lintian.extended_info);
    }

    #[test]
    fn test_changes_file() {
        let path = PathBuf::from("/path/to/file.changes");
        let lintian = Lintian::new().changes_file(&path);
        assert_eq!(lintian.changes_file_path, Some(format!("{:?}", path)));
    }

    #[test]
    fn test_tag_display_limit() {
        let lintian = Lintian::new().tag_display_limit(10);
        assert_eq!(lintian.tag_display_limit, Some(10));
    }

    #[test]
    fn test_fail_on_warning() {
        let lintian = Lintian::new().fail_on_warning();
        assert_eq!(lintian.fail_on, vec!["warning".to_string()]);
    }

    #[test]
    fn test_fail_on_error() {
        let lintian = Lintian::new().fail_on_error();
        assert_eq!(lintian.fail_on, vec!["error".to_string()]);
    }
    
    #[test]
    fn test_fail_on_both() {
        let lintian = Lintian::new().fail_on_warning().fail_on_error();
        assert_eq!(lintian.fail_on, vec!["warning".to_string(), "error".to_string()]);
    }

    #[test]
    fn test_with_codename_jammy() {
        let lintian = Lintian::new().with_codename("jammy");
        assert_eq!(lintian.codename, Some("jammy".to_string()));
        assert!(lintian.suppress_tags.contains(&"malformed-deb-archive".to_string()));
    }

    #[test]
    fn test_with_codename_other() {
        let lintian = Lintian::new().with_codename("focal");
        assert_eq!(lintian.codename, Some("focal".to_string()));
        assert!(!lintian.suppress_tags.contains(&"malformed-deb-archive".to_string()));
    }

    #[test]
    fn test_build_args() {
        let lintian = Lintian::new()
            .suppress_tag("tag1")
            .suppress_tag("tag2")
            .info()
            .extended_info()
            .changes_file("/path/to/file.changes")
            .tag_display_limit(5)
            .fail_on_warning();
        
        let args = lintian.build_args();
        
        assert!(args.contains(&"--suppress-tags".to_string()));
        assert!(args.contains(&"tag1".to_string()));
        assert!(args.contains(&"tag2".to_string()));
        assert!(args.contains(&"-i".to_string()));
        assert!(args.contains(&"-I".to_string()));
        assert!(args.contains(&format!("{:?}", PathBuf::from("/path/to/file.changes"))));
        assert!(args.contains(&"--tag-display-limit=5".to_string()));
        assert!(args.contains(&"--fail-on=warning".to_string()));
    }

    // #[test]
    // fn test_execute() {
    //     // Setup mock
    //     let mut mock = MockExecuteCommand::new();
    //     mock.expect_call()
    //         .with(
    //             eq("lintian"),
    //             eq(vec!["-i".to_string()]),
    //             eq(None)
    //         )
    //         .times(1)
    //         .returning(|_, _, _| Ok(()));
        
    //     // We'd normally replace the actual execute_command with our mock
    //     // For this test, we're just verifying the arguments
        
    //     let lintian = Lintian::new().info();
    //     let args = lintian.build_args();
        
    //     assert_eq!(args, vec!["-i".to_string()]);
    // }

    #[test]
    fn test_chaining() {
        let lintian = Lintian::new()
            .info()
            .extended_info()
            .suppress_tag("tag1")
            .changes_file("file.changes")
            .fail_on_error()
            .with_codename("noble");
        
        assert!(lintian.show_info);
        assert!(lintian.extended_info);
        assert_eq!(lintian.suppress_tags, vec!["tag1", "malformed-deb-archive"]);
        assert_eq!(lintian.changes_file_path, Some(format!("{:?}", PathBuf::from("file.changes"))));
        assert_eq!(lintian.fail_on, vec!["error".to_string()]);
        assert_eq!(lintian.codename, Some("noble".to_string()));
    }
}