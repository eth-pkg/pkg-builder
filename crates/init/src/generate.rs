use crate::{InitConfig, Runtime, SourceConfig};

/// Escape a string value for safe embedding in TOML basic strings.
/// Handles backslashes, double quotes, and control characters.
fn escape_toml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                out.push_str(&format!("\\u{:04X}", c as u32));
            }
            _ => out.push(c),
        }
    }
    out
}

/// Generate the contents of `pkg-builder.toml`.
pub fn generate_toml(config: &InitConfig) -> String {
    let mut out = String::new();

    // [package]
    out.push_str("[package]\n");
    out.push_str(&format!("name = \"{}\"\n", escape_toml(&config.name)));
    out.push_str(&format!("version = \"{}\"\n", escape_toml(&config.version)));
    out.push_str(&format!(
        "revision = \"{}\"\n",
        escape_toml(&config.revision)
    ));
    out.push_str(&format!(
        "homepage = \"{}\"\n",
        escape_toml(&config.homepage)
    ));
    out.push_str(&format!("spec = \"{}.sss\"\n", escape_toml(&config.name)));

    // [source]
    out.push_str("\n[source]\n");
    match &config.source {
        SourceConfig::Virtual => {
            out.push_str("type = \"virtual\"\n");
        }
        SourceConfig::Git { url, tag } => {
            out.push_str("type = \"git\"\n");
            out.push_str(&format!("url = \"{}\"\n", escape_toml(url)));
            out.push_str(&format!("tag = \"{}\"\n", escape_toml(tag)));
            out.push_str("submodules = []\n");
        }
        SourceConfig::Tarball { url, hash } => {
            out.push_str("type = \"tarball\"\n");
            let filename = url.rsplit('/').next().unwrap_or(url);
            out.push_str(&format!("url = \"{}\"\n", escape_toml(filename)));
            if let Some(hash) = hash {
                out.push_str(&format!("hash = \"{}\"\n", escape_toml(hash)));
            }
        }
    }

    // [build]
    out.push_str("\n[build]\n");
    out.push_str(&format!(
        "distribution = \"{}\"\n",
        escape_toml(&config.distribution.to_string())
    ));
    out.push_str(&format!("arch = \"{}\"\n", escape_toml(&config.arch)));
    out.push_str(&format!(
        "workdir = \"~/.pkg-builder/packages/{}\"\n",
        escape_toml(&config.distribution.to_string())
    ));

    // [runtime]
    if config.runtime != Runtime::None {
        out.push_str("\n[runtime]\n");
        out.push_str(&format!(
            "profile = \"{}\"\n",
            escape_toml(&config.runtime.to_string())
        ));
        for (key, value) in &config.runtime_vars {
            if value.contains('\n') {
                out.push_str(&format!(
                    "{} = \"\"\"{}\"\"\"",
                    escape_toml(key),
                    escape_toml(value)
                ));
                if !value.ends_with('\n') {
                    out.push('\n');
                }
            } else {
                out.push_str(&format!(
                    "{} = \"{}\"\n",
                    escape_toml(key),
                    escape_toml(value)
                ));
            }
        }
    }

    // [tools]
    out.push_str("\n[tools]\n");
    out.push_str("pkg_builder = \"0.3.1\"\n");
    out.push_str("debcrafter = \"8189263\"\n");
    out.push_str("sbuild = \"0.85.6\"\n");

    out
}

/// Generate the contents of `{name}.sss` (source service spec).
pub fn generate_sss(config: &InitConfig) -> String {
    let mut out = String::new();
    out.push_str(&format!("name = \"{}\"\n", escape_toml(&config.name)));
    out.push_str(&format!(
        "maintainer = \"{} <{}>\"\n",
        escape_toml(&config.maintainer_name),
        escape_toml(&config.maintainer_email)
    ));
    out.push_str(&format!("section = \"{}\"\n", escape_toml(&config.section)));
    out.push_str("variants = []\n");
    out.push_str("build_depends = []\n");
    out.push_str(&format!("packages = [\"{}\"]\n", escape_toml(&config.name)));
    out.push_str("skip_debug_symbols = true\n");
    out
}

/// Generate the contents of `{name}.sps` (package spec).
pub fn generate_sps(config: &InitConfig) -> String {
    let summary = if config.summary.is_empty() {
        &config.name
    } else {
        &config.summary
    };
    let mut out = String::new();
    out.push_str(&format!("name = \"{}\"\n", escape_toml(&config.name)));
    out.push_str("architecture = \"any\"\n");
    out.push_str(&format!("summary = \"\"\"{}\"\"\"\n", escape_toml(summary)));
    out.push_str("conflicts = []\n");
    out.push_str("recommends = []\n");
    out.push_str("provides = []\n");
    out.push_str("suggests = []\n");
    out.push_str("depends = []\n");
    out.push_str("add_files = []\n");
    out.push_str("add_links = []\n");
    out.push_str("add_manpages = []\n");
    out.push_str(&format!(
        "long_doc = \"\"\"\n{}\n Long Description:\n  TODO: Add a detailed description of the package.\n\"\"\"\n",
        escape_toml(summary)
    ));
    out
}
