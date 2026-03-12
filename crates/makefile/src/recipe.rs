use crate::variables::VariableResolver;
use crate::GeneratorError;

/// A parsed recipe command.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Download { url: String, dest: String },
    Verify { algo: String, hash: String, file: String },
    Extract { file: String, dest: String, strip: Option<u32> },
    Symlink { src: String, target: String },
    AptInstall { packages: Vec<String> },
    AptRemove { packages: Vec<String> },
    AptUpdate,
    DpkgInstall { file: String },
    Run { cmd: String },
    VerifyGpg { file: String, sig: String },
    Debcrafter { spec: String },
    Patch,
    Sbuild,
    Lintian,
    Piuparts,
    Autopkgtest,
    Include { name: String },
    /// Snapshot-specific: disable Valid-Until
    SnapshotWorkaround,
    /// Snapshot-specific: add security repo
    SnapshotSecurity { url: String, codename: String },
    /// Noble-specific: add extra repos
    NobleRepos,
    /// Git clone
    GitClone { url: String, tag: String },
    /// Create empty tarball for virtual packages
    CreateEmptyTar,
    /// Declare a required host tool (checked in preflight)
    Require { tools: Vec<String> },
    /// Install a tool if not already present
    Install { tool: String, cmd: String },
}

pub struct RecipeParser;

impl RecipeParser {
    /// Parse a recipe string into a list of commands, expanding variables and REPEAT blocks.
    pub fn parse(
        source: &str,
        file_name: &str,
        vars: &VariableResolver,
    ) -> Result<Vec<Command>, GeneratorError> {
        let lines: Vec<&str> = source.lines().collect();
        Self::parse_lines(&lines, file_name, vars)
    }

    fn parse_lines(
        lines: &[&str],
        file_name: &str,
        vars: &VariableResolver,
    ) -> Result<Vec<Command>, GeneratorError> {
        let mut commands = Vec::new();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                i += 1;
                continue;
            }

            // Handle REPEAT blocks
            if line.starts_with("REPEAT ") {
                let (repeat_cmds, end_idx) =
                    Self::parse_repeat(lines, i, file_name, vars)?;
                commands.extend(repeat_cmds);
                i = end_idx + 1;
                continue;
            }

            // Substitute variables
            let expanded = vars.substitute(line);
            let cmd = Self::parse_command(&expanded, file_name, i + 1)?;
            commands.push(cmd);
            i += 1;
        }

        Ok(commands)
    }

    fn parse_repeat(
        lines: &[&str],
        start: usize,
        file_name: &str,
        vars: &VariableResolver,
    ) -> Result<(Vec<Command>, usize), GeneratorError> {
        let header = lines[start].trim();
        // Parse: REPEAT <var> IN {{list_name}}
        let parts: Vec<&str> = header.split_whitespace().collect();
        if parts.len() < 4 || parts[2] != "IN" {
            return Err(GeneratorError::RecipeParse {
                file: file_name.into(),
                line: start + 1,
                message: "Invalid REPEAT syntax. Expected: REPEAT <var> IN {{list}}".into(),
            });
        }

        let item_var = parts[1];
        let list_ref = parts[3];
        // Extract list name from {{list_name}}
        let list_name = list_ref
            .strip_prefix("{{")
            .and_then(|s| s.strip_suffix("}}"))
            .ok_or_else(|| GeneratorError::RecipeParse {
                file: file_name.into(),
                line: start + 1,
                message: format!("Expected {{{{list_name}}}}, got {}", list_ref),
            })?;

        // Find END
        let mut end_idx = start + 1;
        let mut depth = 1;
        while end_idx < lines.len() {
            let l = lines[end_idx].trim();
            if l.starts_with("REPEAT ") {
                depth += 1;
            } else if l == "END" {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            end_idx += 1;
        }

        if depth != 0 {
            return Err(GeneratorError::RecipeParse {
                file: file_name.into(),
                line: start + 1,
                message: "REPEAT block has no matching END".into(),
            });
        }

        // Get the body lines
        let body_lines: Vec<&str> = lines[start + 1..end_idx]
            .iter()
            .copied()
            .collect();

        // Get the list items from variables
        let items = vars.get_list(list_name);

        // Expand: for each item, substitute {{item_var.field}} and parse
        let mut all_cmds = Vec::new();
        for item in &items {
            let expanded_body: Vec<String> = body_lines
                .iter()
                .map(|line| {
                    let mut expanded = vars.substitute(line);
                    // Replace {{item_var.field}} with item values
                    for (field, value) in item {
                        expanded = expanded.replace(
                            &format!("{{{{{}.{}}}}}", item_var, field),
                            value,
                        );
                    }
                    expanded
                })
                .collect();

            let expanded_refs: Vec<&str> = expanded_body.iter().map(|s| s.as_str()).collect();
            let cmds = Self::parse_lines(&expanded_refs, file_name, vars)?;
            all_cmds.extend(cmds);
        }

        Ok((all_cmds, end_idx))
    }

    fn parse_command(
        line: &str,
        file_name: &str,
        line_num: usize,
    ) -> Result<Command, GeneratorError> {
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        let keyword = parts[0];
        let args = if parts.len() > 1 { parts[1].trim() } else { "" };

        match keyword {
            "DOWNLOAD" => {
                let (url, dest) = split_two(args, file_name, line_num, "DOWNLOAD <url> <dest>")?;
                Ok(Command::Download {
                    url: url.into(),
                    dest: dest.into(),
                })
            }
            "VERIFY" => {
                let parts: Vec<&str> = args.splitn(3, ' ').collect();
                if parts.len() < 3 {
                    return Err(parse_err(file_name, line_num, "VERIFY <algo> <hash> <file>"));
                }
                Ok(Command::Verify {
                    algo: parts[0].into(),
                    hash: parts[1].into(),
                    file: parts[2].into(),
                })
            }
            "EXTRACT" => {
                let parts: Vec<&str> = args.split_whitespace().collect();
                if parts.len() < 2 {
                    return Err(parse_err(file_name, line_num, "EXTRACT <file> <dest> [strip=N]"));
                }
                let strip = parts.get(2).and_then(|s| {
                    s.strip_prefix("strip=").and_then(|n| n.parse().ok())
                });
                Ok(Command::Extract {
                    file: parts[0].into(),
                    dest: parts[1].into(),
                    strip,
                })
            }
            "SYMLINK" => {
                let (src, target) = split_two(args, file_name, line_num, "SYMLINK <src> <target>")?;
                Ok(Command::Symlink {
                    src: src.into(),
                    target: target.into(),
                })
            }
            "APT_INSTALL" => {
                let packages: Vec<String> = args.split_whitespace().map(String::from).collect();
                Ok(Command::AptInstall { packages })
            }
            "APT_REMOVE" => {
                let packages: Vec<String> = args.split_whitespace().map(String::from).collect();
                Ok(Command::AptRemove { packages })
            }
            "APT_UPDATE" => Ok(Command::AptUpdate),
            "DPKG_INSTALL" => Ok(Command::DpkgInstall { file: args.into() }),
            "RUN" => Ok(Command::Run { cmd: args.into() }),
            "VERIFY_GPG" => {
                let (file, sig) = split_two(args, file_name, line_num, "VERIFY_GPG <file> <sig>")?;
                Ok(Command::VerifyGpg {
                    file: file.into(),
                    sig: sig.into(),
                })
            }
            "DEBCRAFTER" => Ok(Command::Debcrafter { spec: args.into() }),
            "PATCH" => Ok(Command::Patch),
            "SBUILD" => Ok(Command::Sbuild),
            "LINTIAN" => Ok(Command::Lintian),
            "PIUPARTS" => Ok(Command::Piuparts),
            "AUTOPKGTEST" => Ok(Command::Autopkgtest),
            "INCLUDE" => Ok(Command::Include { name: args.into() }),
            "SNAPSHOT_WORKAROUND" => Ok(Command::SnapshotWorkaround),
            "SNAPSHOT_SECURITY" => {
                let (url, codename) =
                    split_two(args, file_name, line_num, "SNAPSHOT_SECURITY <url> <codename>")?;
                Ok(Command::SnapshotSecurity {
                    url: url.into(),
                    codename: codename.into(),
                })
            }
            "NOBLE_REPOS" => Ok(Command::NobleRepos),
            "GIT_CLONE" => {
                let (url, tag) = split_two(args, file_name, line_num, "GIT_CLONE <url> <tag>")?;
                Ok(Command::GitClone {
                    url: url.into(),
                    tag: tag.into(),
                })
            }
            "CREATE_EMPTY_TAR" => Ok(Command::CreateEmptyTar),
            "REQUIRE" => {
                let tools: Vec<String> = args.split_whitespace().map(String::from).collect();
                if tools.is_empty() {
                    return Err(parse_err(file_name, line_num, "REQUIRE <tool1> [tool2] ..."));
                }
                Ok(Command::Require { tools })
            }
            "INSTALL" => {
                let (tool, cmd) = split_two(args, file_name, line_num, "INSTALL <tool> <install command>")?;
                Ok(Command::Install {
                    tool: tool.into(),
                    cmd: cmd.into(),
                })
            }
            _ => Err(GeneratorError::RecipeParse {
                file: file_name.into(),
                line: line_num,
                message: format!("Unknown command: {}", keyword),
            }),
        }
    }
}

fn split_two<'a>(
    args: &'a str,
    file_name: &str,
    line_num: usize,
    expected: &str,
) -> Result<(&'a str, &'a str), GeneratorError> {
    let parts: Vec<&str> = args.splitn(2, ' ').collect();
    if parts.len() < 2 {
        return Err(parse_err(file_name, line_num, expected));
    }
    Ok((parts[0], parts[1]))
}

fn parse_err(file_name: &str, line_num: usize, expected: &str) -> GeneratorError {
    GeneratorError::RecipeParse {
        file: file_name.into(),
        line: line_num,
        message: format!("Expected: {}", expected),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn empty_vars() -> VariableResolver {
        VariableResolver {
            vars: HashMap::new(),
        }
    }

    fn vars_with(pairs: &[(&str, &str)]) -> VariableResolver {
        let mut vars = HashMap::new();
        for (k, v) in pairs {
            vars.insert(k.to_string(), v.to_string());
        }
        VariableResolver { vars }
    }

    #[test]
    fn test_parse_simple_commands() {
        let source = r#"
APT_INSTALL wget
DOWNLOAD https://example.com/go.tar.gz /tmp/go.tar.gz
VERIFY sha256 abc123 /tmp/go.tar.gz
EXTRACT /tmp/go.tar.gz /usr/local
SYMLINK /usr/local/go/bin/go /usr/bin/go
RUN go version
APT_REMOVE wget
"#;
        let cmds = RecipeParser::parse(source, "test", &empty_vars()).unwrap();
        assert_eq!(cmds.len(), 7);
        assert!(matches!(cmds[0], Command::AptInstall { .. }));
        assert!(matches!(cmds[1], Command::Download { .. }));
        assert!(matches!(cmds[6], Command::AptRemove { .. }));
    }

    #[test]
    fn test_parse_with_variables() {
        let vars = vars_with(&[("binary_url", "https://go.dev/go.tar.gz")]);
        let source = "DOWNLOAD {{binary_url}} /tmp/go.tar.gz";
        let cmds = RecipeParser::parse(source, "test", &vars).unwrap();
        match &cmds[0] {
            Command::Download { url, .. } => {
                assert_eq!(url, "https://go.dev/go.tar.gz");
            }
            _ => panic!("Expected Download"),
        }
    }

    #[test]
    fn test_parse_repeat() {
        let vars = vars_with(&[
            ("packages.0.name", "pkg-a"),
            ("packages.0.url", "http://a"),
            ("packages.1.name", "pkg-b"),
            ("packages.1.url", "http://b"),
        ]);
        let source = r#"
REPEAT pkg IN {{packages}}
  DOWNLOAD {{pkg.url}} /tmp/{{pkg.name}}.deb
END
"#;
        let cmds = RecipeParser::parse(source, "test", &vars).unwrap();
        assert_eq!(cmds.len(), 2);
        match &cmds[0] {
            Command::Download { url, dest } => {
                assert_eq!(url, "http://a");
                assert_eq!(dest, "/tmp/pkg-a.deb");
            }
            _ => panic!("Expected Download"),
        }
    }

    #[test]
    fn test_parse_comments_and_empty_lines() {
        let source = r#"
# This is a comment
APT_INSTALL wget

# Another comment
APT_REMOVE wget
"#;
        let cmds = RecipeParser::parse(source, "test", &empty_vars()).unwrap();
        assert_eq!(cmds.len(), 2);
    }

    #[test]
    fn test_parse_pipeline_commands() {
        let source = "DEBCRAFTER spec.sss\nPATCH\nSBUILD\nLINTIAN";
        let cmds = RecipeParser::parse(source, "test", &empty_vars()).unwrap();
        assert_eq!(cmds.len(), 4);
        assert!(matches!(cmds[0], Command::Debcrafter { .. }));
        assert!(matches!(cmds[1], Command::Patch));
        assert!(matches!(cmds[2], Command::Sbuild));
        assert!(matches!(cmds[3], Command::Lintian));
    }

    #[test]
    fn test_parse_extract_with_strip() {
        let source = "EXTRACT /tmp/file.tar.gz /dest strip=1";
        let cmds = RecipeParser::parse(source, "test", &empty_vars()).unwrap();
        match &cmds[0] {
            Command::Extract { strip, .. } => assert_eq!(*strip, Some(1)),
            _ => panic!("Expected Extract"),
        }
    }

    #[test]
    fn test_parse_unknown_command_errors() {
        let result = RecipeParser::parse("UNKNOWN_CMD foo", "test", &empty_vars());
        assert!(result.is_err());
    }
}
