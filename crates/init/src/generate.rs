use crate::{InitConfig, Runtime, SourceConfig};

/// Generate the contents of `pkg-builder.toml`.
pub fn generate_toml(config: &InitConfig) -> String {
    let mut out = String::new();

    // [package]
    out.push_str("[package]\n");
    out.push_str(&format!("name = \"{}\"\n", config.name));
    out.push_str(&format!("version = \"{}\"\n", config.version));
    out.push_str(&format!("revision = \"{}\"\n", config.revision));
    out.push_str(&format!("homepage = \"{}\"\n", config.homepage));
    out.push_str(&format!("spec = \"{}.sss\"\n", config.name));

    // [source]
    out.push_str("\n[source]\n");
    match &config.source {
        SourceConfig::Virtual => {
            out.push_str("type = \"virtual\"\n");
        }
        SourceConfig::Git { url, tag } => {
            out.push_str("type = \"git\"\n");
            out.push_str(&format!("url = \"{}\"\n", url));
            out.push_str(&format!("tag = \"{}\"\n", tag));
            out.push_str("submodules = []\n");
        }
        SourceConfig::Tarball { url, hash } => {
            out.push_str("type = \"tarball\"\n");
            let filename = url.rsplit('/').next().unwrap_or(url);
            out.push_str(&format!("url = \"{}\"\n", filename));
            if let Some(hash) = hash {
                out.push_str(&format!("hash = \"{}\"\n", hash));
            }
        }
    }

    // [build]
    out.push_str("\n[build]\n");
    out.push_str(&format!("distribution = \"{}\"\n", config.distribution));
    out.push_str(&format!("arch = \"{}\"\n", config.arch));
    out.push_str(&format!(
        "workdir = \"~/.pkg-builder/packages/{}\"\n",
        config.distribution
    ));

    // [runtime]
    if config.runtime != Runtime::None {
        out.push_str("\n[runtime]\n");
        out.push_str(&format!("recipe = \"{}\"\n", config.runtime));
        for (key, value) in &config.runtime_vars {
            if value.contains('\n') {
                out.push_str(&format!("{} = \"\"\"{}\"\"\"", key, value));
                if !value.ends_with('\n') {
                    out.push('\n');
                }
            } else {
                out.push_str(&format!("{} = \"{}\"\n", key, value));
            }
        }
    }

    // [testing]
    out.push_str("\n[testing]\n");
    out.push_str("lintian = false\n");
    out.push_str("piuparts = false\n");
    out.push_str("autopkgtest = false\n");

    // [tools]
    out.push_str("\n[tools]\n");
    out.push_str("pkg_builder = \"0.3.1\"\n");
    out.push_str("debcrafter = \"8189263\"\n");
    out.push_str("sbuild = \"0.85.6\"\n");
    out.push_str("lintian = \"2.116.3\"\n");
    out.push_str("piuparts = \"1.1.7\"\n");
    out.push_str("autopkgtest = \"5.28\"\n");

    out
}

/// Generate the contents of `{name}.sss` (source service spec).
pub fn generate_sss(config: &InitConfig) -> String {
    let mut out = String::new();
    out.push_str(&format!("name = \"{}\"\n", config.name));
    out.push_str(&format!(
        "maintainer = \"{} <{}>\"\n",
        config.maintainer_name, config.maintainer_email
    ));
    out.push_str(&format!("section = \"{}\"\n", config.section));
    out.push_str("variants = []\n");
    out.push_str("build_depends = []\n");
    out.push_str(&format!("packages = [\"{}\"]\n", config.name));
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
    out.push_str(&format!("name = \"{}\"\n", config.name));
    out.push_str("architecture = \"any\"\n");
    out.push_str(&format!("summary = \"\"\"{}\"\"\"\n", summary));
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
        summary
    ));
    out
}
