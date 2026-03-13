use crate::ir::{Operation, ToolInstall};
use crate::variables::VariableResolver;
use crate::GeneratorError;

/// Parsed pipeline recipe: preamble items + phases.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedPipeline {
    pub required_tools: Vec<String>,
    pub installable_tools: Vec<ToolInstall>,
    pub chroot_modifiers: Vec<Operation>,
    pub phases: Vec<ParsedPhase>,
}

/// A phase from a pipeline recipe.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedPhase {
    pub name: String,
    pub operations: Vec<Operation>,
}

/// Parsed runtime recipe: flat list of chroot commands.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedRuntime {
    pub operations: Vec<Operation>,
}

/// Parse a pipeline recipe (with PHASE blocks) into structured IR.
pub fn parse_pipeline(
    source: &str,
    file_name: &str,
    vars: &VariableResolver,
) -> Result<ParsedPipeline, GeneratorError> {
    let lines: Vec<&str> = source.lines().collect();
    let mut required_tools = Vec::new();
    let mut installable_tools = Vec::new();
    let mut chroot_modifiers = Vec::new();
    let mut phases = Vec::new();

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();

        if line.is_empty() || line.starts_with('#') {
            i += 1;
            continue;
        }

        // Handle PHASE blocks
        if line.starts_with("PHASE ") {
            let phase_name = line.strip_prefix("PHASE ").unwrap().trim().to_string();
            let mut phase_lines = Vec::new();
            i += 1;
            while i < lines.len() {
                let pl = lines[i].trim();
                if pl.starts_with("PHASE ") || i == lines.len() - 1 {
                    // If we hit another PHASE or end of file, check if last line is content
                    if i == lines.len() - 1 && !pl.is_empty() && !pl.starts_with("PHASE ") {
                        phase_lines.push(lines[i]);
                        i += 1;
                    }
                    break;
                }
                phase_lines.push(lines[i]);
                i += 1;
            }
            let ops = parse_operation_lines(&phase_lines, file_name, vars)?;
            phases.push(ParsedPhase {
                name: phase_name,
                operations: ops,
            });
            continue;
        }

        // Top-level commands (before any PHASE): preamble items
        let expanded = vars.substitute(line);
        let parts: Vec<&str> = expanded.splitn(2, ' ').collect();
        let keyword = parts[0];
        let args = if parts.len() > 1 { parts[1].trim() } else { "" };

        match keyword {
            "REQUIRE" => {
                let tools: Vec<String> = args.split_whitespace().map(String::from).collect();
                if tools.is_empty() {
                    return Err(parse_err(file_name, i + 1, "REQUIRE <tool1> [tool2] ..."));
                }
                for tool in tools {
                    if !required_tools.contains(&tool) {
                        required_tools.push(tool);
                    }
                }
            }
            "INSTALL" => {
                let (tool, cmd) =
                    split_two(args, file_name, i + 1, "INSTALL <tool> <install command>")?;
                if !installable_tools
                    .iter()
                    .any(|t: &ToolInstall| t.name == tool)
                {
                    installable_tools.push(ToolInstall {
                        name: tool.to_string(),
                        install_cmd: cmd.to_string(),
                    });
                }
            }
            "SNAPSHOT_WORKAROUND" => {
                chroot_modifiers.push(Operation::SnapshotWorkaround);
            }
            "SNAPSHOT_SECURITY" => {
                let (url, codename) =
                    split_two(args, file_name, i + 1, "SNAPSHOT_SECURITY <url> <codename>")?;
                chroot_modifiers.push(Operation::SnapshotSecurity {
                    url: url.to_string(),
                    codename: codename.to_string(),
                });
            }
            "NOBLE_REPOS" => {
                chroot_modifiers.push(Operation::NobleRepos);
            }
            _ => {
                // If there are no PHASE blocks yet and this is a pipeline command,
                // it's using the old flat format — parse entire file as flat pipeline
                return parse_flat_pipeline(source, file_name, vars);
            }
        }
        i += 1;
    }

    Ok(ParsedPipeline {
        required_tools,
        installable_tools,
        chroot_modifiers,
        phases,
    })
}

/// Parse old-style flat pipeline (no PHASE blocks) into structured phases.
/// This maintains backwards compatibility with the existing recipe format.
/// Note: preamble keyword handling (REQUIRE, INSTALL, SNAPSHOT_*, NOBLE_REPOS)
/// is duplicated from parse_pipeline — consider extracting if more keywords are added.
fn parse_flat_pipeline(
    source: &str,
    file_name: &str,
    vars: &VariableResolver,
) -> Result<ParsedPipeline, GeneratorError> {
    let lines: Vec<&str> = source.lines().collect();
    let mut required_tools = Vec::new();
    let mut installable_tools = Vec::new();
    let mut chroot_modifiers = Vec::new();

    let mut source_ops = Vec::new();
    let mut debian_ops = Vec::new();
    let mut patch_ops = Vec::new();
    let mut build_ops = Vec::new();

    // Track which "phase" we're building
    // Flat pipelines follow this order:
    // INSTALL/REQUIRE → preamble
    // DOWNLOAD/VERIFY/GIT_CLONE/CREATE_EMPTY_TAR → source phase
    // EXTRACT/DEBCRAFTER → debian phase
    // PATCH → patch phase
    // INCLUDE/SBUILD → build phase
    // SNAPSHOT_WORKAROUND/SNAPSHOT_SECURITY/NOBLE_REPOS → chroot modifiers

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        if line.is_empty() || line.starts_with('#') {
            i += 1;
            continue;
        }

        if line.starts_with("REPEAT ") {
            let (ops, end_idx) = parse_repeat_ops(&lines, i, file_name, vars)?;
            // REPEAT in flat pipelines goes to build phase (runtime include context)
            build_ops.extend(ops);
            i = end_idx + 1;
            continue;
        }

        let expanded = vars.substitute(line);
        let parts: Vec<&str> = expanded.splitn(2, ' ').collect();
        let keyword = parts[0];
        let args = if parts.len() > 1 { parts[1].trim() } else { "" };

        match keyword {
            "REQUIRE" => {
                let tools: Vec<String> = args.split_whitespace().map(String::from).collect();
                if tools.is_empty() {
                    return Err(parse_err(file_name, i + 1, "REQUIRE <tool1> [tool2] ..."));
                }
                for tool in tools {
                    if !required_tools.contains(&tool) {
                        required_tools.push(tool);
                    }
                }
            }
            "INSTALL" => {
                let (tool, cmd) =
                    split_two(args, file_name, i + 1, "INSTALL <tool> <install command>")?;
                if !installable_tools
                    .iter()
                    .any(|t: &ToolInstall| t.name == tool)
                {
                    installable_tools.push(ToolInstall {
                        name: tool.to_string(),
                        install_cmd: cmd.to_string(),
                    });
                }
            }
            "DOWNLOAD" => {
                let (url, dest) = split_two(args, file_name, i + 1, "DOWNLOAD <url> <dest>")?;
                source_ops.push(Operation::Download {
                    url: url.to_string(),
                    dest: dest.to_string(),
                });
            }
            "VERIFY" => {
                let vparts: Vec<&str> = args.splitn(3, ' ').collect();
                if vparts.len() < 3 {
                    return Err(parse_err(file_name, i + 1, "VERIFY <algo> <hash> <file>"));
                }
                source_ops.push(Operation::Verify {
                    algo: vparts[0].to_string(),
                    hash: vparts[1].to_string(),
                    file: vparts[2].to_string(),
                });
            }
            "GIT_CLONE" => {
                let (url, tag) = split_two(args, file_name, i + 1, "GIT_CLONE <url> <tag>")?;
                source_ops.push(Operation::GitClone {
                    url: url.to_string(),
                    tag: tag.to_string(),
                });
            }
            "CREATE_EMPTY_TAR" => {
                source_ops.push(Operation::CreateEmptyTar);
            }
            "EXTRACT" => {
                let eparts: Vec<&str> = args.split_whitespace().collect();
                if eparts.len() < 2 {
                    return Err(parse_err(
                        file_name,
                        i + 1,
                        "EXTRACT <file> <dest> [strip=N]",
                    ));
                }
                let strip = eparts
                    .get(2)
                    .and_then(|s| s.strip_prefix("strip=").and_then(|n| n.parse().ok()));
                debian_ops.push(Operation::Extract {
                    file: eparts[0].to_string(),
                    dest: eparts[1].to_string(),
                    strip,
                });
            }
            "DEBCRAFTER" => {
                debian_ops.push(Operation::Debcrafter {
                    spec: args.to_string(),
                });
            }
            "PATCH" => {
                patch_ops.push(Operation::Patch);
            }
            "INCLUDE" => {
                build_ops.push(Operation::Include {
                    name: args.to_string(),
                });
            }
            "SBUILD" => {
                build_ops.push(Operation::Sbuild);
            }
            "SNAPSHOT_WORKAROUND" => {
                chroot_modifiers.push(Operation::SnapshotWorkaround);
            }
            "SNAPSHOT_SECURITY" => {
                let (url, codename) =
                    split_two(args, file_name, i + 1, "SNAPSHOT_SECURITY <url> <codename>")?;
                chroot_modifiers.push(Operation::SnapshotSecurity {
                    url: url.to_string(),
                    codename: codename.to_string(),
                });
            }
            "NOBLE_REPOS" => {
                chroot_modifiers.push(Operation::NobleRepos);
            }
            "LINTIAN" => {
                build_ops.push(Operation::Lintian);
            }
            "PIUPARTS" => {
                build_ops.push(Operation::Piuparts);
            }
            "AUTOPKGTEST" => {
                build_ops.push(Operation::Autopkgtest);
            }
            _ => {
                return Err(GeneratorError::RecipeParse {
                    file: file_name.into(),
                    line: i + 1,
                    message: format!("Unknown command: {}", keyword),
                });
            }
        }
        i += 1;
    }

    let mut phases = Vec::new();
    if !source_ops.is_empty() {
        phases.push(ParsedPhase {
            name: "source".to_string(),
            operations: source_ops,
        });
    }
    if !debian_ops.is_empty() {
        phases.push(ParsedPhase {
            name: "debian".to_string(),
            operations: debian_ops,
        });
    }
    if !patch_ops.is_empty() {
        phases.push(ParsedPhase {
            name: "patch".to_string(),
            operations: patch_ops,
        });
    }
    if !build_ops.is_empty() {
        phases.push(ParsedPhase {
            name: "build".to_string(),
            operations: build_ops,
        });
    }

    Ok(ParsedPipeline {
        required_tools,
        installable_tools,
        chroot_modifiers,
        phases,
    })
}

/// Parse a runtime recipe into a flat list of operations.
pub fn parse_runtime(
    source: &str,
    file_name: &str,
    vars: &VariableResolver,
) -> Result<ParsedRuntime, GeneratorError> {
    let lines: Vec<&str> = source.lines().collect();
    let ops = parse_operation_lines(&lines, file_name, vars)?;
    Ok(ParsedRuntime { operations: ops })
}

/// Parse lines into a list of operations, handling REPEAT blocks.
fn parse_operation_lines(
    lines: &[&str],
    file_name: &str,
    vars: &VariableResolver,
) -> Result<Vec<Operation>, GeneratorError> {
    let mut ops = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();
        if line.is_empty() || line.starts_with('#') {
            i += 1;
            continue;
        }

        if line.starts_with("REPEAT ") {
            let (repeat_ops, end_idx) = parse_repeat_ops(lines, i, file_name, vars)?;
            ops.extend(repeat_ops);
            i = end_idx + 1;
            continue;
        }

        let expanded = vars.substitute(line);
        let op = parse_single_operation(&expanded, file_name, i + 1)?;
        ops.push(op);
        i += 1;
    }

    Ok(ops)
}

/// Parse a single operation from a line.
fn parse_single_operation(
    line: &str,
    file_name: &str,
    line_num: usize,
) -> Result<Operation, GeneratorError> {
    let parts: Vec<&str> = line.splitn(2, ' ').collect();
    let keyword = parts[0];
    let args = if parts.len() > 1 { parts[1].trim() } else { "" };

    match keyword {
        "DOWNLOAD" => {
            let (url, dest) = split_two(args, file_name, line_num, "DOWNLOAD <url> <dest>")?;
            Ok(Operation::Download {
                url: url.into(),
                dest: dest.into(),
            })
        }
        "VERIFY" => {
            let parts: Vec<&str> = args.splitn(3, ' ').collect();
            if parts.len() < 3 {
                return Err(parse_err(
                    file_name,
                    line_num,
                    "VERIFY <algo> <hash> <file>",
                ));
            }
            Ok(Operation::Verify {
                algo: parts[0].into(),
                hash: parts[1].into(),
                file: parts[2].into(),
            })
        }
        "EXTRACT" => {
            let parts: Vec<&str> = args.split_whitespace().collect();
            if parts.len() < 2 {
                return Err(parse_err(
                    file_name,
                    line_num,
                    "EXTRACT <file> <dest> [strip=N]",
                ));
            }
            let strip = parts
                .get(2)
                .and_then(|s| s.strip_prefix("strip=").and_then(|n| n.parse().ok()));
            Ok(Operation::Extract {
                file: parts[0].into(),
                dest: parts[1].into(),
                strip,
            })
        }
        "SYMLINK" => {
            let (src, target) = split_two(args, file_name, line_num, "SYMLINK <src> <target>")?;
            Ok(Operation::Symlink {
                src: src.into(),
                target: target.into(),
            })
        }
        "APT_INSTALL" => {
            let packages: Vec<String> = args.split_whitespace().map(String::from).collect();
            Ok(Operation::AptInstall { packages })
        }
        "APT_REMOVE" => {
            let packages: Vec<String> = args.split_whitespace().map(String::from).collect();
            Ok(Operation::AptRemove { packages })
        }
        "APT_UPDATE" => Ok(Operation::AptUpdate),
        "DPKG_INSTALL" => Ok(Operation::DpkgInstall { file: args.into() }),
        "RUN" => Ok(Operation::Run { cmd: args.into() }),
        "VERIFY_GPG" => {
            let (file, sig) = split_two(args, file_name, line_num, "VERIFY_GPG <file> <sig>")?;
            Ok(Operation::VerifyGpg {
                file: file.into(),
                sig: sig.into(),
            })
        }
        "DEBCRAFTER" => Ok(Operation::Debcrafter { spec: args.into() }),
        "PATCH" => Ok(Operation::Patch),
        "SBUILD" => Ok(Operation::Sbuild),
        "LINTIAN" => Ok(Operation::Lintian),
        "PIUPARTS" => Ok(Operation::Piuparts),
        "AUTOPKGTEST" => Ok(Operation::Autopkgtest),
        "INCLUDE" => Ok(Operation::Include { name: args.into() }),
        "SNAPSHOT_WORKAROUND" => Ok(Operation::SnapshotWorkaround),
        "SNAPSHOT_SECURITY" => {
            let (url, codename) = split_two(
                args,
                file_name,
                line_num,
                "SNAPSHOT_SECURITY <url> <codename>",
            )?;
            Ok(Operation::SnapshotSecurity {
                url: url.into(),
                codename: codename.into(),
            })
        }
        "NOBLE_REPOS" => Ok(Operation::NobleRepos),
        "GIT_CLONE" => {
            let (url, tag) = split_two(args, file_name, line_num, "GIT_CLONE <url> <tag>")?;
            Ok(Operation::GitClone {
                url: url.into(),
                tag: tag.into(),
            })
        }
        "CREATE_EMPTY_TAR" => Ok(Operation::CreateEmptyTar),
        "REQUIRE" => {
            let tools: Vec<String> = args.split_whitespace().map(String::from).collect();
            if tools.is_empty() {
                return Err(parse_err(
                    file_name,
                    line_num,
                    "REQUIRE <tool1> [tool2] ...",
                ));
            }
            // In runtime context, REQUIRE is not used but we handle gracefully
            // by returning a Run that's a no-op. Actually this shouldn't appear in runtime.
            Err(GeneratorError::RecipeParse {
                file: file_name.into(),
                line: line_num,
                message: "REQUIRE not valid in this context (use top-level in pipeline)".into(),
            })
        }
        "INSTALL" => Err(GeneratorError::RecipeParse {
            file: file_name.into(),
            line: line_num,
            message: "INSTALL not valid in this context (use top-level in pipeline)".into(),
        }),
        _ => Err(GeneratorError::RecipeParse {
            file: file_name.into(),
            line: line_num,
            message: format!("Unknown command: {}", keyword),
        }),
    }
}

/// Parse a REPEAT block, expanding variables and producing operations.
fn parse_repeat_ops(
    lines: &[&str],
    start: usize,
    file_name: &str,
    vars: &VariableResolver,
) -> Result<(Vec<Operation>, usize), GeneratorError> {
    let header = lines[start].trim();
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

    let body_lines: Vec<&str> = lines[start + 1..end_idx].iter().copied().collect();
    let items = vars.get_list(list_name);

    let mut all_ops = Vec::new();
    for item in &items {
        let expanded_body: Vec<String> = body_lines
            .iter()
            .map(|line| {
                let mut expanded = vars.substitute(line);
                for (field, value) in item {
                    expanded = expanded.replace(&format!("{{{{{}.{}}}}}", item_var, field), value);
                }
                expanded
            })
            .collect();

        let expanded_refs: Vec<&str> = expanded_body.iter().map(|s| s.as_str()).collect();
        let ops = parse_operation_lines(&expanded_refs, file_name, vars)?;
        all_ops.extend(ops);
    }

    Ok((all_ops, end_idx))
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
    fn test_parse_runtime_simple() {
        let source = r#"
APT_INSTALL wget
DOWNLOAD https://example.com/go.tar.gz /tmp/go.tar.gz
RUN go version
APT_REMOVE wget
"#;
        let rt = parse_runtime(source, "test", &empty_vars()).unwrap();
        assert_eq!(rt.operations.len(), 4);
        assert!(matches!(rt.operations[0], Operation::AptInstall { .. }));
        assert!(matches!(rt.operations[1], Operation::Download { .. }));
        assert!(matches!(rt.operations[2], Operation::Run { .. }));
        assert!(matches!(rt.operations[3], Operation::AptRemove { .. }));
    }

    #[test]
    fn test_parse_runtime_with_variables() {
        let vars = vars_with(&[("binary_url", "https://go.dev/go.tar.gz")]);
        let source = "DOWNLOAD {{binary_url}} /tmp/go.tar.gz";
        let rt = parse_runtime(source, "test", &vars).unwrap();
        match &rt.operations[0] {
            Operation::Download { url, .. } => {
                assert_eq!(url, "https://go.dev/go.tar.gz");
            }
            _ => panic!("Expected Download"),
        }
    }

    #[test]
    fn test_parse_runtime_repeat() {
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
        let rt = parse_runtime(source, "test", &vars).unwrap();
        assert_eq!(rt.operations.len(), 2);
        match &rt.operations[0] {
            Operation::Download { url, dest } => {
                assert_eq!(url, "http://a");
                assert_eq!(dest, "/tmp/pkg-a.deb");
            }
            _ => panic!("Expected Download"),
        }
    }

    #[test]
    fn test_parse_flat_pipeline() {
        let vars = vars_with(&[
            ("source_url", "https://example.com/src.tar.gz"),
            ("source_hash_algo", "sha256"),
            ("source_hash", "abc123"),
            ("spec_file", "test.sss"),
            ("runtime_recipe", "go"),
        ]);
        let source = r#"
INSTALL debcrafter cargo install --git https://github.com/Kixunil/debcrafter --rev abc
REQUIRE wget tar debcrafter dpkg-parsechangelog sbuild
DOWNLOAD https://example.com/src.tar.gz $(TARBALL_PATH)
VERIFY sha256 abc123 $(TARBALL_PATH)
EXTRACT $(TARBALL_PATH) $(BUILD_FILES_DIR)
DEBCRAFTER test.sss
PATCH
INCLUDE go
SBUILD
"#;
        let pl = parse_pipeline(source, "test", &empty_vars()).unwrap();
        assert_eq!(
            pl.required_tools,
            vec!["wget", "tar", "debcrafter", "dpkg-parsechangelog", "sbuild"]
        );
        assert_eq!(pl.installable_tools.len(), 1);
        assert_eq!(pl.installable_tools[0].name, "debcrafter");
        assert_eq!(pl.phases.len(), 4);
        assert_eq!(pl.phases[0].name, "source");
        assert_eq!(pl.phases[1].name, "debian");
        assert_eq!(pl.phases[2].name, "patch");
        assert_eq!(pl.phases[3].name, "build");
    }

    #[test]
    fn test_parse_flat_pipeline_noble() {
        let source = r#"
INSTALL debcrafter cargo install --git https://github.com/Kixunil/debcrafter --rev abc
REQUIRE wget tar debcrafter dpkg-parsechangelog sbuild
DOWNLOAD https://example.com/src.tar.gz $(TARBALL_PATH)
VERIFY sha256 abc123 $(TARBALL_PATH)
EXTRACT $(TARBALL_PATH) $(BUILD_FILES_DIR)
DEBCRAFTER test.sss
PATCH
NOBLE_REPOS
INCLUDE go
SBUILD
"#;
        let pl = parse_pipeline(source, "test", &empty_vars()).unwrap();
        assert_eq!(pl.chroot_modifiers.len(), 1);
        assert!(matches!(pl.chroot_modifiers[0], Operation::NobleRepos));
    }

    #[test]
    fn test_parse_comments_and_empty_lines() {
        let source = r#"
# This is a comment
APT_INSTALL wget

# Another comment
APT_REMOVE wget
"#;
        let rt = parse_runtime(source, "test", &empty_vars()).unwrap();
        assert_eq!(rt.operations.len(), 2);
    }

    #[test]
    fn test_parse_extract_with_strip() {
        let source = "EXTRACT /tmp/file.tar.gz /dest strip=1";
        let rt = parse_runtime(source, "test", &empty_vars()).unwrap();
        match &rt.operations[0] {
            Operation::Extract { strip, .. } => assert_eq!(*strip, Some(1)),
            _ => panic!("Expected Extract"),
        }
    }

    #[test]
    fn test_parse_unknown_command_errors() {
        let result = parse_runtime("UNKNOWN_CMD foo", "test", &empty_vars());
        assert!(result.is_err());
    }
}
