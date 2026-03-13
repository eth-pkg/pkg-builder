use crate::commands::InitCommand;
use dialoguer::{Confirm, Input, Select};
use log::warn;
use pkg_builder_init::{
    self, Distribution, InitConfig, Runtime, RuntimeSetup, SourceType, ALL_DISTRIBUTIONS,
    ALL_RUNTIMES, ALL_SOURCE_TYPES, DEBIAN_SECTIONS,
};
use std::path::Path;

pub fn run(cmd: &InitCommand) -> Result<(), Box<dyn std::error::Error>> {
    let default_name = std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "my-package".to_string());

    // Top-level fields
    let name = text_or_flag(&cmd.name, "Package name", Some(&default_name))?;
    let version = text_or_flag(&cmd.version, "Package version", Some("1.0.0"))?;
    let revision = text_or_flag(&cmd.revision, "Package revision", Some("1"))?;
    let homepage = text_or_flag(&cmd.homepage, "Homepage URL", None)?;
    let maintainer_name = text_or_flag(&cmd.maintainer_name, "Maintainer name", None)?;
    let maintainer_email = text_or_flag(&cmd.maintainer_email, "Maintainer email", None)?;
    let section = select_or_flag(&cmd.section, "Debian section", DEBIAN_SECTIONS, Some("net"))?;
    let summary = text_or_flag(&cmd.summary, "Short description/summary", None)?;

    // Source type + source-specific fields
    let source_type = enum_or_flag(&cmd.source_type, "Source type", ALL_SOURCE_TYPES)?;
    let mut source_vals = std::collections::HashMap::new();
    for field in source_type.required_fields() {
        let flag = match field.key {
            "url" => &cmd.url,
            "tag" => &cmd.tag,
            _ => &None,
        };
        source_vals.insert(field.key, text_or_flag(flag, field.label, None)?);
    }
    let output_dir = cmd.output.as_deref().unwrap_or(".");
    let source = pkg_builder_init::resolve_source(
        source_type,
        source_vals.get("url").map(|s| s.as_str()),
        source_vals.get("tag").map(|s| s.as_str()),
        Path::new(output_dir),
        cmd.upstream_hash.as_deref(),
    )?;

    // Distribution, arch
    let distribution = enum_or_flag(&cmd.distribution, "Target distribution", ALL_DISTRIBUTIONS)?;
    let arch = cmd.arch.clone().unwrap_or_else(|| "amd64".to_string());

    // Runtime + runtime vars
    let runtime = enum_or_flag(&cmd.runtime, "Runtime recipe", ALL_RUNTIMES)?;
    let runtime_vars = resolve_runtime(runtime)?;

    let config = InitConfig {
        name,
        version,
        revision,
        homepage,
        maintainer_name,
        maintainer_email,
        section,
        summary,
        source,
        distribution,
        arch,
        runtime,
        runtime_vars,
    };

    let output_path = Path::new(output_dir);
    let files = pkg_builder_init::write_init_files(&config, output_path)?;

    println!("Created:");
    println!("  {}", files.toml_path.display());
    println!("  {}", files.sss_path.display());
    println!("  {}", files.sps_path.display());

    Ok(())
}

// ---- Generic prompt helpers ----

/// Use the CLI flag value if present, otherwise prompt for text input.
fn text_or_flag(
    flag: &Option<String>,
    label: &str,
    default: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(v) = flag {
        return Ok(v.clone());
    }
    let mut input = Input::<String>::new().with_prompt(label);
    if let Some(d) = default {
        input = input.default(d.to_string());
    }
    Ok(input.interact_text()?)
}

/// Use the CLI flag value if it matches an option, otherwise show a select prompt.
fn select_or_flag(
    flag: &Option<String>,
    label: &str,
    options: &[&str],
    default: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(v) = flag {
        return Ok(v.clone());
    }
    let default_idx = default
        .and_then(|d| options.iter().position(|&o| o == d))
        .unwrap_or(0);
    let idx = Select::new()
        .with_prompt(label)
        .items(options)
        .default(default_idx)
        .interact()?;
    Ok(options[idx].to_string())
}

/// Use the CLI flag to parse an enum, otherwise show a select prompt.
fn enum_or_flag<T: Copy>(
    flag: &Option<String>,
    label: &str,
    all: &[T],
) -> Result<T, Box<dyn std::error::Error>>
where
    T: EnumSelect,
{
    if let Some(s) = flag {
        return T::parse(s).ok_or_else(|| format!("Unknown {}: {}", label, s).into());
    }
    let labels: Vec<&str> = all.iter().map(|v| v.display()).collect();
    let idx = Select::new()
        .with_prompt(label)
        .items(&labels)
        .default(0)
        .interact()?;
    Ok(all[idx])
}

trait EnumSelect {
    fn parse(s: &str) -> Option<Self>
    where
        Self: Sized;
    fn display(&self) -> &'static str;
}

impl EnumSelect for SourceType {
    fn parse(s: &str) -> Option<Self> {
        SourceType::from_str(s)
    }
    fn display(&self) -> &'static str {
        self.as_str()
    }
}

impl EnumSelect for Distribution {
    fn parse(s: &str) -> Option<Self> {
        Distribution::from_str(s)
    }
    fn display(&self) -> &'static str {
        self.as_str()
    }
}

impl EnumSelect for Runtime {
    fn parse(s: &str) -> Option<Self> {
        Runtime::from_str(s)
    }
    fn display(&self) -> &'static str {
        self.as_str()
    }
}

/// Prompt for all runtime fields based on the RuntimeSetup returned by the lib.
fn resolve_runtime(
    runtime: Runtime,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    match runtime.setup() {
        RuntimeSetup::NoVars => Ok(vec![]),
        RuntimeSetup::ManualOnly => {
            println!("This runtime has complex configuration. Please fill in runtime variables manually after init.");
            Ok(vec![])
        }
        RuntimeSetup::NeedsInput { fields } => prompt_fields(fields),
        RuntimeSetup::AutoResolved {
            latest,
            fields,
            optional_fields,
        } => {
            let use_latest = Confirm::new()
                .with_prompt(format!("Use {} {} (latest)?", runtime, latest.version))
                .default(true)
                .interact()?;

            let mut vars = if use_latest {
                latest.vars
            } else {
                let version: String = Input::new()
                    .with_prompt(format!("{} version", runtime))
                    .interact_text()?;
                runtime
                    .resolve_version(&version)
                    .unwrap_or_else(|e| {
                        warn!("Failed to resolve {} {}: {}", runtime, version, e);
                        prompt_fields(fields).unwrap_or_default()
                    })
            };

            for field in optional_fields {
                if Confirm::new()
                    .with_prompt(format!("Add {}?", field.key))
                    .default(false)
                    .interact()?
                {
                    let value: String = Input::new()
                        .with_prompt(field.label)
                        .interact_text()?;
                    vars.push((field.key.to_string(), value));
                }
            }

            Ok(vars)
        }
    }
}

fn prompt_fields(
    fields: &[pkg_builder_init::RuntimeField],
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let mut vars = Vec::new();
    for field in fields {
        let value: String = Input::new()
            .with_prompt(field.label)
            .interact_text()?;
        vars.push((field.key.to_string(), value));
    }
    Ok(vars)
}
