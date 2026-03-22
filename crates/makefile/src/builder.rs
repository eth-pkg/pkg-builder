use std::path::Path;

use config::build_env::DebianCodename;
use config::PkgConfig;

use crate::ir::*;

pub struct PlanBuilder<'a> {
    config: &'a PkgConfig,
    runtime_mk: Option<String>,
}

impl<'a> PlanBuilder<'a> {
    pub fn new(config: &'a PkgConfig, runtime_mk: Option<String>) -> Self {
        Self { config, runtime_mk }
    }

    pub fn build(self) -> BuildPlan {
        let preamble = self.build_preamble();
        let phases = self.build_phases();

        BuildPlan { preamble, phases }
    }

    fn build_preamble(&self) -> Preamble {
        let variables = self.build_variables();
        let required_tools = self.required_tools();
        let installable_tools = self.installable_tools();
        let chroot_modifier_lines = self.build_chroot_modifier_lines();

        Preamble {
            variables,
            required_tools,
            installable_tools,
            runtime_mk: self.runtime_mk.clone(),
            chroot_modifier_lines,
        }
    }

    fn required_tools(&self) -> Vec<String> {
        let mut tools = Vec::new();
        match &self.config.source {
            config::source::SourceKind::Tarball { .. } => {
                tools.extend(["wget", "tar"].iter().map(|s| s.to_string()));
            }
            config::source::SourceKind::Git { .. } => {
                tools.extend(["git", "tar"].iter().map(|s| s.to_string()));
            }
            config::source::SourceKind::Virtual => {
                tools.push("tar".to_string());
            }
        }
        tools.extend(
            ["debcrafter", "dpkg-parsechangelog", "sbuild"]
                .iter()
                .map(|s| s.to_string()),
        );
        tools
    }

    fn installable_tools(&self) -> Vec<ToolInstall> {
        vec![ToolInstall {
            name: "debcrafter".to_string(),
            install_cmd:
                "cargo install --git https://github.com/Kixunil/debcrafter --rev $(DEBCRAFTER_REV)"
                    .to_string(),
        }]
    }

    fn build_chroot_modifier_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();

        // Noble repos (must come before runtime)
        for cmd in self.config.build_env.distribution.extra_chroot_commands() {
            lines.push(format!(
                "SBUILD_FLAGS += --chroot-setup-commands='{}'",
                shell_escape(&cmd)
            ));
        }

        // Snapshot workaround (must come before runtime)
        if self.config.build_env.uses_snapshot() {
            lines.insert(
                0,
                "SBUILD_FLAGS += --chroot-setup-commands='echo '\\''Acquire::Check-Valid-Until \"false\";'\\'' > /etc/apt/apt.conf.d/99snapshot'"
                    .to_string(),
            );
        }

        // Snapshot security (must come after runtime)
        if let Some(security_url) = self.config.build_env.security_repo_url() {
            let codename = match &self.config.build_env.distribution {
                config::build_env::Distribution::Debian(DebianCodename::Bookworm) => "bookworm",
                config::build_env::Distribution::Debian(DebianCodename::Trixie) => "trixie",
                _ => self.config.build_env.distribution.as_short(),
            };
            lines.push(format!(
                "SBUILD_FLAGS += --chroot-setup-commands='echo '\\''deb {} {}-security main'\\'' > /etc/apt/sources.list.d/security-snapshot.list'",
                security_url, codename
            ));
            lines.push(
                "SBUILD_FLAGS += --chroot-setup-commands='apt-get update'"
                    .to_string(),
            );
        }

        lines
    }

    fn build_variables(&self) -> Vec<VarDecl> {
        let mut vars = Vec::new();
        let pkg = &self.config.package;
        let env = &self.config.build_env;
        let root = &self.config.config_root;

        // Paths — ROOT has special padding
        vars.push(raw(
            "ROOT    := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))\n",
        ));
        vars.push(simple(
            "WORK_DIR",
            &portabilize_path(&env.workdir.display().to_string(), root),
        ));
        vars.push(simple(
            "CHROOT_DIR",
            &portabilize_path(&env.chroot_dir.display().to_string(), root),
        ));
        vars.push(raw("\n"));

        // Package
        vars.push(simple("PKG_NAME", &pkg.name));
        vars.push(simple("PKG_VERSION", &pkg.version));
        vars.push(simple("PKG_REVISION", &pkg.revision));
        vars.push(simple("PKG_HOMEPAGE", &pkg.homepage));
        vars.push(simple(
            "PKG_SPEC",
            &portabilize_path(&pkg.spec.display().to_string(), root),
        ));
        vars.push(raw("\n"));

        // Build environment
        vars.push(simple("DISTRIBUTION", env.distribution.as_short()));
        vars.push(simple("ARCH", &env.arch.to_string()));
        vars.push(simple("REPO_URL", &env.repo_url()));
        if let Some(ref date) = env.snapshot_date {
            vars.push(simple("SNAPSHOT_DATE", date));
        }
        if let Some(ref date) = env.snapshot_security_date {
            vars.push(simple("SNAPSHOT_SECURITY_DATE", date));
        }
        vars.push(raw("\n"));

        // Testing flags
        let testing = &env.testing;
        vars.push(simple("RUN_LINTIAN", bool_str(testing.run_lintian)));
        vars.push(simple("RUN_PIUPARTS", bool_str(testing.run_piuparts)));
        vars.push(simple("RUN_AUTOPKGTEST", bool_str(testing.run_autopkgtest)));
        vars.push(raw(concat!(
            "\nTEST_TARGETS :=\n",
            "ifeq ($(RUN_PIUPARTS),true)\nTEST_TARGETS += test-piuparts\nendif\n",
            "ifeq ($(RUN_AUTOPKGTEST),true)\nTEST_TARGETS += test-autopkgtest\nendif\n",
            "\n",
        )));

        // Source
        match &self.config.source {
            config::source::SourceKind::Tarball { url, hash, .. } => {
                vars.push(simple("SRC_URL", &portabilize_path(url, root)));
                if let Some(h) = hash {
                    let algo = if h.len() == 128 { "sha512" } else { "sha256" };
                    vars.push(simple("SRC_HASH_ALGO", algo));
                    vars.push(simple("SRC_HASH", h));
                }
                vars.push(raw("\n"));
            }
            config::source::SourceKind::Git { url, tag, .. } => {
                vars.push(simple("GIT_URL", url));
                vars.push(simple("GIT_TAG", tag));
                vars.push(raw("\n"));
            }
            config::source::SourceKind::Virtual => {}
        }

        // Tools
        vars.push(simple("DEBCRAFTER_REV", &env.tool_versions.debcrafter));
        vars.push(raw("DEBCRAFTER  := debcrafter-$(DEBCRAFTER_REV)\n"));
        vars.push(raw("\n"));

        // Runtime-specific variables
        self.emit_runtime_variables(&mut vars);

        // Derived paths — special formatting with padding
        vars.push(raw(
            "OUT_DIR     = $(WORK_DIR)/$(PKG_NAME)-$(PKG_VERSION)-$(PKG_REVISION)\n",
        ));
        vars.push(raw("SRC_DIR     = $(OUT_DIR)/$(PKG_NAME)-$(PKG_VERSION)\n"));
        vars.push(raw(
            "SRC_TARBALL = $(OUT_DIR)/$(PKG_NAME)_$(PKG_VERSION).orig.tar.gz\n",
        ));
        if env.snapshot_date.is_some() {
            vars.push(raw(
                "CHROOT_TARBALL = $(CHROOT_DIR)/$(DISTRIBUTION)-$(ARCH)-$(SNAPSHOT_DATE).tar.gz\n",
            ));
        } else {
            vars.push(raw(
                "CHROOT_TARBALL = $(CHROOT_DIR)/$(DISTRIBUTION)-$(ARCH).tar.gz\n",
            ));
        }
        vars.push(raw("\n"));

        vars
    }

    fn emit_runtime_variables(&self, vars: &mut Vec<VarDecl>) {
        let runtime = match &self.config.runtime {
            Some(rt) => rt,
            None => return,
        };

        let mut has_packages = false;
        let mut pkg_count = 0;

        for (key, value) in &runtime.vars {
            match value {
                toml::Value::String(s) => {
                    let var_name = key.to_uppercase().replace('-', "_");
                    vars.push(simple(&var_name, s));
                }
                toml::Value::Array(arr) if key == "packages" => {
                    has_packages = true;
                    pkg_count = arr.len();
                    for (i, item) in arr.iter().enumerate() {
                        if let toml::Value::Table(table) = item {
                            let n = i + 1;
                            if let Some(toml::Value::String(s)) = table.get("name") {
                                vars.push(simple(&format!("PKG{}_NAME", n), s));
                            }
                            if let Some(toml::Value::String(s)) = table.get("url") {
                                vars.push(simple(&format!("PKG{}_URL", n), s));
                            }
                            if let Some(toml::Value::String(s)) = table.get("hash") {
                                vars.push(simple(&format!("PKG{}_HASH", n), s));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Emit RUNTIME_PKG_INDICES for dotnet .mk foreach/eval loops
        if has_packages && pkg_count > 0 {
            let indices: Vec<String> = (1..=pkg_count).map(|i| i.to_string()).collect();
            vars.push(simple("RUNTIME_PKG_INDICES", &indices.join(" ")));

            // Emit APT_NAME variants for dotnet packages
            if runtime.profile.starts_with("dotnet") {
                if let Some(toml::Value::Array(pkgs)) = runtime.vars.get("packages") {
                    for (i, item) in pkgs.iter().enumerate() {
                        if let toml::Value::Table(table) = item {
                            if let Some(toml::Value::String(name)) = table.get("name") {
                                let apt_name = transform_dotnet_name(name, &self.config.build_env.arch);
                                vars.push(simple(&format!("PKG{}_APT_NAME", i + 1), &apt_name));
                            }
                        }
                    }
                }
            }
        }
    }

    fn build_phases(&self) -> Vec<Phase> {
        let mut phases = Vec::new();

        // Directories
        phases.push(Phase {
            name: "directories".into(),
            output: Some("$(OUT_DIR)".into()),
            deps: vec![],
            order_only_deps: vec![],
            operations: vec![],
            condition: None,
            is_alias: false,
        });

        // Source phase — built from config source kind
        match &self.config.source {
            config::source::SourceKind::Tarball { url, hash, .. } => {
                let is_local = !url.starts_with("http://") && !url.starts_with("https://");
                let deps = if is_local {
                    vec!["$(SRC_URL)".to_string()]
                } else {
                    vec![]
                };

                let algo = hash.as_ref().map(|h| {
                    if h.len() == 128 { "sha512" } else { "sha256" }
                });

                let mut operations = vec![Operation::Download {
                    url: url.clone(),
                    dest: "$(SRC_TARBALL)".into(),
                }];
                if let (Some(algo), Some(hash)) = (algo, hash) {
                    operations.push(Operation::Verify {
                        algo: algo.to_string(),
                        hash: hash.clone(),
                        file: "$(SRC_TARBALL)".into(),
                    });
                }

                phases.push(Phase {
                    name: "source".into(),
                    output: Some("$(SRC_TARBALL)".into()),
                    deps,
                    order_only_deps: vec!["$(OUT_DIR)".into()],
                    operations,
                    condition: None,
                    is_alias: false,
                });
            }
            config::source::SourceKind::Git { url, tag, submodules } => {
                let mut operations = vec![Operation::GitClone {
                    url: url.clone(),
                    tag: tag.clone(),
                }];

                for submodule in submodules {
                    operations.push(Operation::SubmoduleCheckout {
                        path: submodule.path.clone(),
                        commit: submodule.commit.trim().to_string(),
                    });
                }

                phases.push(Phase {
                    name: "source".into(),
                    output: Some("$(SRC_TARBALL)".into()),
                    deps: vec![],
                    order_only_deps: vec!["$(OUT_DIR)".into()],
                    operations,
                    condition: None,
                    is_alias: false,
                });
            }
            config::source::SourceKind::Virtual => {
                phases.push(Phase {
                    name: "source".into(),
                    output: Some("$(SRC_TARBALL)".into()),
                    deps: vec![],
                    order_only_deps: vec!["$(OUT_DIR)".into()],
                    operations: vec![Operation::CreateEmptyTar],
                    condition: None,
                    is_alias: false,
                });
            }
        }

        // Debian phase
        let prev_output = "$(SRC_TARBALL)".to_string();
        let mut debian_ops = vec![];

        // For tarball and virtual: extract first, then debcrafter
        match &self.config.source {
            config::source::SourceKind::Tarball { .. } | config::source::SourceKind::Virtual => {
                debian_ops.push(Operation::Extract {
                    file: "$(SRC_TARBALL)".into(),
                    dest: "$(SRC_DIR)".into(),
                    strip: None,
                });
            }
            config::source::SourceKind::Git { .. } => {
                // Git: source was already extracted during clone
            }
        }
        debian_ops.push(Operation::Debcrafter {
            spec: "$(PKG_SPEC)".into(),
        });

        phases.push(Phase {
            name: "debian".into(),
            output: Some("$(SRC_DIR)/debian/rules".into()),
            deps: vec![prev_output],
            order_only_deps: vec![],
            operations: debian_ops,
            condition: None,
            is_alias: false,
        });

        // Patch phase
        phases.push(Phase {
            name: "patch".into(),
            output: Some("$(SRC_DIR)/debian/source/format".into()),
            deps: vec!["$(SRC_DIR)/debian/rules".into()],
            order_only_deps: vec![],
            operations: vec![Operation::Patch],
            condition: None,
            is_alias: false,
        });

        // SBUILD_FLAGS (pseudo-phase for rendering)
        phases.push(Phase {
            name: "sbuild_flags".into(),
            output: None,
            deps: vec![],
            order_only_deps: vec![],
            operations: vec![],
            condition: None,
            is_alias: false,
        });

        // Build phase
        phases.push(Phase {
            name: "build".into(),
            output: Some("$(OUT_DIR)/.built".into()),
            deps: vec![
                "$(SRC_DIR)/debian/source/format".into(),
                "$(CHROOT_TARBALL)".into(),
            ],
            order_only_deps: vec![],
            operations: vec![Operation::Sbuild],
            condition: None,
            is_alias: false,
        });

        // Phony aliases
        phases.push(Phase {
            name: "phony_aliases".into(),
            output: None,
            deps: vec![],
            order_only_deps: vec![],
            operations: vec![],
            condition: None,
            is_alias: true,
        });

        // Environment
        phases.push(Phase {
            name: "env".into(),
            output: Some("$(CHROOT_TARBALL)".into()),
            deps: vec![],
            order_only_deps: vec!["preflight".into()],
            operations: if self.config.build_env.uses_snapshot() {
                vec![Operation::UsesSnapshot]
            } else {
                vec![]
            },
            condition: None,
            is_alias: false,
        });

        // Test
        phases.push(Phase {
            name: "test".into(),
            output: None,
            deps: vec![],
            order_only_deps: vec![],
            operations: vec![],
            condition: None,
            is_alias: false,
        });

        // Clean
        phases.push(Phase {
            name: "clean".into(),
            output: None,
            deps: vec![],
            order_only_deps: vec![],
            operations: vec![],
            condition: None,
            is_alias: false,
        });

        phases
    }
}

fn simple(name: &str, value: &str) -> VarDecl {
    VarDecl {
        name: name.into(),
        value: VarValue::Literal(value.into()),
        kind: AssignKind::Immediate,
    }
}

fn raw(text: &str) -> VarDecl {
    VarDecl {
        name: String::new(),
        value: VarValue::Raw(text.into()),
        kind: AssignKind::Immediate,
    }
}

fn bool_str(b: bool) -> &'static str {
    if b {
        "true"
    } else {
        "false"
    }
}

/// Convert an absolute path to a portable Makefile-friendly path.
fn portabilize_path(path: &str, config_root: &Path) -> String {
    let config_root_str = config_root.display().to_string();

    if let Some(rel) = path.strip_prefix(&config_root_str) {
        let rel = rel.strip_prefix('/').unwrap_or(rel);
        if rel.is_empty() {
            return "$(ROOT)".to_string();
        }
        return format!("$(ROOT)/{}", rel);
    }

    if let Ok(home) = std::env::var("HOME") {
        if let Some(rel) = path.strip_prefix(&home) {
            let rel = rel.strip_prefix('/').unwrap_or(rel);
            return format!("$(HOME)/{}", rel);
        }
    }

    path.to_string()
}

/// Escape a string for safe use inside single-quoted shell arguments in Makefile.
fn shell_escape(s: &str) -> String {
    s.replace('\'', "'\\''")
}

/// Transform dotnet package name to apt name format.
fn transform_dotnet_name(input: &str, arch: &config::build_env::Architecture) -> String {
    let arch_str = format!("_{}", arch);
    if let Some(pos) = input.find(&arch_str) {
        let trimmed = &input[..pos];
        trimmed.replace('_', "=")
    } else {
        input.replace('_', "=")
    }
}
