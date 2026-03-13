use std::path::Path;

use config::PkgConfig;

use crate::ir::*;
use crate::parser::{ParsedPipeline, ParsedRuntime};
use crate::variables::VariableResolver;

pub struct PlanBuilder<'a> {
    config: &'a PkgConfig,
    pipeline: ParsedPipeline,
    runtime: Option<ParsedRuntime>,
}

impl<'a> PlanBuilder<'a> {
    pub fn new(
        config: &'a PkgConfig,
        _vars: &'a VariableResolver,
        pipeline: ParsedPipeline,
        runtime: Option<ParsedRuntime>,
    ) -> Self {
        Self {
            config,
            pipeline,
            runtime,
        }
    }

    pub fn build(self) -> BuildPlan {
        let preamble = self.build_preamble();
        let phases = self.build_phases();

        BuildPlan { preamble, phases }
    }

    fn build_preamble(&self) -> Preamble {
        let variables = self.build_variables();
        let required_tools = self.pipeline.required_tools.clone();
        let installable_tools = self.pipeline.installable_tools.clone();
        let chroot_setup = self.build_chroot_setup();

        Preamble {
            variables,
            required_tools,
            installable_tools,
            chroot_setup,
        }
    }

    fn build_variables(&self) -> Vec<VarDecl> {
        let mut vars = Vec::new();
        let pkg = &self.config.package;
        let env = &self.config.build_env;
        let root = &self.config.config_root;

        // Paths — ROOT has special padding
        vars.push(raw("ROOT    := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))\n"));
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
        vars.push(simple(
            "RUN_AUTOPKGTEST",
            bool_str(testing.run_autopkgtest),
        ));
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
        vars.push(raw(
            "SRC_DIR     = $(OUT_DIR)/$(PKG_NAME)-$(PKG_VERSION)\n",
        ));
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

        for (key, value) in &runtime.vars {
            match value {
                toml::Value::String(s) => {
                    let var_name = key.to_uppercase().replace('-', "_");
                    vars.push(simple(&var_name, s));
                }
                toml::Value::Array(arr) if key == "packages" => {
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
    }

    fn build_chroot_setup(&self) -> Vec<Operation> {
        let mut chroot_ops: Vec<Operation> = Vec::new();

        // Process pipeline chroot modifiers
        let mut front_ops: Vec<Operation> = Vec::new();
        let mut back_ops: Vec<Operation> = Vec::new();

        for op in &self.pipeline.chroot_modifiers {
            match op {
                Operation::SnapshotWorkaround => {
                    front_ops.insert(
                        0,
                        Operation::Run {
                            cmd: r#"echo 'Acquire::Check-Valid-Until "false";' > /etc/apt/apt.conf.d/99snapshot"#
                                .to_string(),
                        },
                    );
                }
                Operation::SnapshotSecurity { url, codename } => {
                    back_ops.push(Operation::Run {
                        cmd: format!(
                            "echo 'deb {} {}-security main' > /etc/apt/sources.list.d/security-snapshot.list",
                            url, codename
                        ),
                    });
                    back_ops.push(Operation::Run {
                        cmd: "apt-get update".to_string(),
                    });
                }
                Operation::NobleRepos => {
                    let noble_cmds = vec![
                        "apt install -y software-properties-common",
                        "add-apt-repository universe",
                        "add-apt-repository restricted",
                        "add-apt-repository multiverse",
                        "apt update",
                    ];
                    for (i, cmd) in noble_cmds.into_iter().enumerate() {
                        front_ops.insert(i, Operation::Run {
                            cmd: cmd.to_string(),
                        });
                    }
                }
                _ => {}
            }
        }

        chroot_ops.extend(front_ops);

        // Add runtime operations
        if let Some(ref runtime) = self.runtime {
            chroot_ops.extend(runtime.operations.clone());
        }

        chroot_ops.extend(back_ops);

        chroot_ops
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

        // Source phase
        let source_phase = self.pipeline.phases.iter().find(|p| p.name == "source");
        if let Some(sp) = source_phase {
            let has_local_source = sp.operations.iter().any(|op| {
                matches!(op, Operation::Download { url, .. }
                    if !url.starts_with("http://") && !url.starts_with("https://"))
            });

            let deps = if has_local_source {
                vec!["$(SRC_URL)".to_string()]
            } else {
                vec![]
            };

            let mut operations = sp.operations.clone();

            // Add git submodule operations if applicable
            if let config::source::SourceKind::Git { submodules, .. } = &self.config.source {
                for submodule in submodules {
                    operations.push(Operation::SubmoduleCheckout {
                        path: submodule.path.clone(),
                        commit: submodule.commit.trim().to_string(),
                    });
                }
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

        // Debian phase
        let debian_phase = self.pipeline.phases.iter().find(|p| p.name == "debian");
        if let Some(dp) = debian_phase {
            let prev_output = phases
                .iter()
                .rev()
                .find(|p| p.name == "source")
                .and_then(|p| p.output.clone())
                .unwrap_or_else(|| "preflight".into());

            phases.push(Phase {
                name: "debian".into(),
                output: Some("$(SRC_DIR)/debian/rules".into()),
                deps: vec![prev_output],
                order_only_deps: vec![],
                operations: dp.operations.clone(),
                condition: None,
                is_alias: false,
            });
        }

        // Patch phase
        let patch_phase = self.pipeline.phases.iter().find(|p| p.name == "patch");
        if patch_phase.is_some() {
            let prev_output = phases
                .iter()
                .rev()
                .find(|p| p.name == "debian")
                .and_then(|p| p.output.clone())
                .unwrap_or_else(|| "preflight".into());

            phases.push(Phase {
                name: "patch".into(),
                output: Some("$(SRC_DIR)/debian/source/format".into()),
                deps: vec![prev_output],
                order_only_deps: vec![],
                operations: vec![Operation::Patch],
                condition: None,
                is_alias: false,
            });
        }

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
        if self.pipeline.phases.iter().any(|p| p.name == "build") || patch_phase.is_some() {
            let prev_output = phases
                .iter()
                .rev()
                .find(|p| {
                    p.name == "patch" || p.name == "debian" || p.name == "source"
                })
                .and_then(|p| p.output.clone())
                .unwrap_or_else(|| "preflight".into());

            phases.push(Phase {
                name: "build".into(),
                output: Some("$(OUT_DIR)/.built".into()),
                deps: vec![prev_output, "$(CHROOT_TARBALL)".into()],
                order_only_deps: vec![],
                operations: vec![Operation::Sbuild],
                condition: None,
                is_alias: false,
            });
        }

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
                vec![Operation::Run {
                    cmd: "__uses_snapshot".into(),
                }]
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
