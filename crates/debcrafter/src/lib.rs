// Debcrafter wrapper for pkg-builder.
//
// This module embeds the debian directory generation logic from
// debcrafter (https://github.com/Kixunil/debcrafter) by Martin Habovstiak.
// The original debcrafter code is licensed under MIT.
//
// The debcrafter project only exposes data types and parsing as a library,
// while the generation logic lives in its binary crate. This module ports
// that generation logic so pkg-builder can generate debian/ directories
// directly without requiring a separate debcrafter binary installation.

#![allow(clippy::type_complexity)]

pub(crate) mod codegen;
pub(crate) mod generator;

use std::{io, fs};
use std::convert::TryInto;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::borrow::Borrow;
use codegen::LazyCreateBuilder;
use debcrafter::Set;
use debcrafter::im_repr::{Package, PackageInstance, ServiceInstance};
use debcrafter::types::VPackageName;
use debcrafter::error_report::Report;
use either::Either;
use serde::Deserialize;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DebcrafterError {
    #[error("{tool} failed with exit code {code}")]
    ExitCode { tool: String, code: i32 },
    #[error("{tool} not found: {source}")]
    NotFound { tool: String, source: std::io::Error },
    #[error("{tool}: {message}")]
    Other { tool: String, message: String },
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Deserialize)]
pub struct Source {
    pub section: String,
    #[serde(default)]
    pub build_depends: Vec<String>,
    #[serde(default, rename = "with")]
    pub with_components: Set<String>,
    #[serde(default)]
    pub buildsystem: Option<String>,
    #[serde(default)]
    pub autoconf_params: Vec<String>,
    #[serde(default)]
    pub variants: Set<debcrafter::types::Variant>,
    pub packages: Set<VPackageName>,
    #[serde(default)]
    pub skip_debug_symbols: bool,
    #[serde(default)]
    pub skip_strip: bool,
}

#[derive(Deserialize)]
pub struct SingleSource {
    pub name: String,
    pub maintainer: Option<String>,
    #[serde(flatten)]
    pub source: Source,
}

struct ServiceRule {
    unit_name: String,
    refuse_manual_start: bool,
    refuse_manual_stop: bool,
}

static FILE_GENERATORS: &[(&str, fn(&PackageInstance, LazyCreateBuilder) -> io::Result<()>)] = &[
    ("config", generator::config::generate),
    ("install", generator::install::generate),
    ("dirs", generator::dirs::generate),
    ("links", generator::links::generate),
    ("manpages", generator::manpages::generate),
    ("preinst", generator::preinst::generate),
    ("postinst", generator::postinst::generate),
    ("prerm", generator::prerm::generate),
    ("postrm", generator::postrm::generate),
    ("templates", generator::templates::generate),
    ("triggers", generator::triggers::generate),
];

/// Generate the debian/ directory from a debcrafter spec file.
///
/// This is the main entry point, equivalent to running the debcrafter binary.
pub fn generate_debian_dir(spec_file: &Path, dest_dir: &Path) -> Result<(), DebcrafterError> {
    let spec_file = fs::canonicalize(spec_file).map_err(|_| DebcrafterError::Other {
        tool: "debcrafter".to_string(),
        message: format!("spec file not found: {}", spec_file.display()),
    })?;

    let source_dir = spec_file.parent().unwrap_or(Path::new("."));

    let mut source: SingleSource = debcrafter::input::load_toml(&spec_file)
        .map_err(|e| DebcrafterError::Other {
            tool: "debcrafter".to_string(),
            message: format!("Failed to load spec: {}", e),
        })?;

    let maintainer = source.maintainer
        .or_else(|| std::env::var("DEBEMAIL").ok())
        .unwrap_or_else(|| "unknown <unknown@unknown>".to_string());

    process_source(source_dir, &source.name, &mut source.source, dest_dir, &maintainer)?;

    Ok(())
}

/// Check that dpkg-parsechangelog is available.
pub fn check_dpkg_parsechangelog() -> Result<(), DebcrafterError> {
    let output = std::process::Command::new("which")
        .arg("dpkg-parsechangelog")
        .output()
        .map_err(|e| DebcrafterError::NotFound {
            tool: "dpkg-parsechangelog".to_string(),
            source: e,
        })?;

    if !output.status.success() {
        return Err(DebcrafterError::Other {
            tool: "dpkg-parsechangelog".to_string(),
            message: "not installed, please install it".to_string(),
        });
    }
    Ok(())
}

fn gen_rules<I>(deb_dir: &Path, source: &Source, systemd_services: I) -> io::Result<()>
where
    I: IntoIterator,
    <I as IntoIterator>::IntoIter: ExactSizeIterator,
    <I as IntoIterator>::Item: Borrow<ServiceRule>,
{
    let systemd_services = systemd_services.into_iter();
    let mut out = fs::File::create(deb_dir.join("rules")).expect("Failed to create rules file");

    writeln!(out, "#!/usr/bin/make -f")?;
    writeln!(out)?;
    writeln!(out, "%:")?;
    write!(out, "\tdh $@")?;
    for component in &source.with_components {
        write!(out, " --with {}", component)?;
    }
    if let Some(buildsystem) = &source.buildsystem {
        write!(out, " --buildsystem {}", buildsystem)?;
    }
    writeln!(out)?;

    if systemd_services.len() > 0 {
        writeln!(out)?;
        writeln!(out, "override_dh_installsystemd:")?;
        for service in systemd_services {
            let service = service.borrow();
            write!(out, "\tdh_installsystemd --name={}", service.unit_name)?;
            if service.refuse_manual_start {
                write!(out, " --no-start")?;
            }
            if service.refuse_manual_stop {
                write!(out, " --no-stop-on-upgrade --no-restart-after-upgrade")?;
            }
            writeln!(out)?;
        }
    }
    if !source.autoconf_params.is_empty() {
        writeln!(out)?;
        writeln!(out, "override_dh_auto_configure:")?;
        write!(out, "\tdh_auto_configure --")?;
        for param in &source.autoconf_params {
            write!(out, " {}", param)?;
        }
        writeln!(out)?;
    }
    if source.skip_debug_symbols {
        writeln!(out)?;
        writeln!(out, "override_dh_dwz:")?;
    }
    if source.skip_strip {
        writeln!(out)?;
        writeln!(out, "override_dh_strip:")?;
    }
    Ok(())
}

fn gen_control(deb_dir: &Path, name: &str, source: &Source, maintainer: &str, needs_dh_systemd: bool) -> io::Result<()> {
    let mut out = fs::File::create(deb_dir.join("control")).expect("Failed to create control file");

    writeln!(out, "Source: {}", name)?;
    writeln!(out, "Section: {}", source.section)?;
    writeln!(out, "Priority: optional")?;
    writeln!(out, "Maintainer: {}", maintainer)?;
    write!(out, "Build-Depends: debhelper (>= 9)")?;
    if needs_dh_systemd {
        write!(out, ",\n               debhelper (>= 12.1.1)")?;
    }
    for build_dep in &source.build_depends {
        write!(out, ",\n               {}", build_dep)?;
    }
    writeln!(out)
}

fn copy_changelog(deb_dir: &Path, source: &Path) {
    let dest = deb_dir.join("changelog");
    match fs::copy(source, &dest) {
        Ok(_) => (),
        Err(ref err) if err.kind() == std::io::ErrorKind::NotFound => (),
        Err(err) => panic!("Failed to copy changelog from {} to {}: {}", source.display(), dest.display(), err),
    }
}

fn load_package(source_dir: &Path, package: &VPackageName) -> (Package, PathBuf, String) {
    let filename = package.sps_path(source_dir);
    let source = std::fs::read_to_string(&filename)
        .unwrap_or_else(|error| panic!("failed to read {}: {}", filename.display(), error));
    let package = toml::from_str::<debcrafter::input::Package>(&source)
        .expect("Failed to parse package")
        .try_into()
        .unwrap_or_else(|error: debcrafter::im_repr::PackageError| error.report(filename.display().to_string(), &source));
    (package, filename, source)
}

fn create_lazy_builder(dest_dir: &Path, name: &str, extension: &str, append: bool) -> LazyCreateBuilder {
    let mut file_name = dest_dir.join(name);
    file_name.set_extension(extension);
    LazyCreateBuilder::new(file_name, append)
}

fn changelog_parse_version(changelog_path: &Path) -> Result<String, DebcrafterError> {
    let output = std::process::Command::new("dpkg-parsechangelog")
        .arg("-l")
        .arg(changelog_path)
        .args(&["-S", "Version"])
        .output()
        .map_err(|e| DebcrafterError::NotFound {
            tool: "dpkg-parsechangelog".to_string(),
            source: e,
        })?;

    if !output.status.success() {
        return Err(DebcrafterError::Other {
            tool: "dpkg-parsechangelog".to_string(),
            message: format!("failed with status {}", output.status),
        });
    }

    let mut version = String::from_utf8(output.stdout)
        .map_err(|_| DebcrafterError::Other {
            tool: "dpkg-parsechangelog".to_string(),
            message: "output is not UTF-8".to_string(),
        })?;

    if version.ends_with('\n') {
        version.pop();
    }

    Ok(version)
}

fn get_upstream_version(version: &str) -> &str {
    version.rfind('-').map(|pos| &version[..pos]).unwrap_or(version)
}

fn import_files(instance: &PackageInstance<'_>, debian_dir: &Path, source_dir: &Path) {
    use debcrafter::im_repr::PackageOps;

    let files_dir = debian_dir.join("debcrafter/external-files");

    for source_dest in instance.import_files {
        let source = source_dest[0].expand_to_cow(instance.constants_by_variant());
        let source = source_dir.join(&*source);
        let dest = source_dest[1].expand_to_cow(instance.constants_by_variant());
        validate_import_file_dest(&dest);
        let dest = files_dir.join(dest.trim_start_matches("/"));

        std::fs::create_dir_all(dest.parent().unwrap_or(".".as_ref())).expect("failed to create imported files directory");
        std::fs::copy(&source, &dest)
            .unwrap_or_else(|error| panic!("failed to import file {} into {}: {:?}", source.display(), dest.display(), error));
    }
}

fn validate_import_file_dest(dest: &str) {
    assert!(dest.starts_with('/'), "destination of import_files ({}) must be absolute", dest);
    assert!(!dest.ends_with('/'), "destination of import_files ({}) must not be a directory", dest);
    assert!(dest.find(' ').is_none(), "destination of import_files ({}) must not contain spaces", dest);
}

fn process_source(source_dir: &Path, name: &str, source: &mut Source, dest: &Path, maintainer: &str) -> Result<(), DebcrafterError> {
    let mut changelog_path = source_dir.join(name);
    changelog_path.set_extension("changelog");
    let version = changelog_parse_version(&changelog_path)?;
    let upstream_version = get_upstream_version(&version);

    let dir = dest.join(format!("{}-{}", name, upstream_version));
    let deb_dir = dir.join("debian");
    fs::create_dir_all(&deb_dir).expect("Failed to create debian directory");
    copy_changelog(&deb_dir, &changelog_path);

    gen_control(&deb_dir, name, source, maintainer, true).expect("Failed to generate control");
    std::fs::write(deb_dir.join("compat"), "12\n").expect("Failed to write debian/compat");

    let mut services = Vec::new();

    let packages = source.packages
        .iter()
        .map(|package| load_package(source_dir, package));

    for (package, filename, package_source) in packages {
        use debcrafter::im_repr::PackageOps;
        let includes = package.load_includes(source_dir, None);

        let instances = if source.variants.is_empty() || !package.name.is_templated() {
            let instance = package.instantiate(None, Some(&includes));
            instance
                .validate()
                .unwrap_or_else(|error| error.report(filename.display().to_string(), package_source));
            Either::Left(std::iter::once(instance))
        } else {
            Either::Right(source.variants.iter()
                .map(|variant| {
                    let instance = package.instantiate(Some(variant), Some(&includes));
                    instance
                        .validate()
                        .unwrap_or_else(|error| error.report(filename.display().to_string(), &package_source));
                    instance
                }))
        };

        services.extend(instances
            .into_iter()
            .filter_map(|instance| {
                for &(extension, gen) in FILE_GENERATORS {
                    let out = create_lazy_builder(&deb_dir, &instance.name, extension, false);
                    gen(&instance, out).expect("Failed to generate file");
                }

                if let Some(service_name) = instance.service_name() {
                    let out = create_lazy_builder(&deb_dir, &instance.name, &format!("{}.service", service_name), false);
                    generator::service::generate(&instance, out).expect("Failed to generate file");
                }

                let out = create_lazy_builder(&deb_dir, "control", "", true);
                generator::control::generate(&instance, out, upstream_version, source.buildsystem.as_ref().map(AsRef::as_ref)).expect("Failed to generate file");
                generator::static_files::generate(&instance, &dir).expect("Failed to generate static files");

                import_files(&instance, &deb_dir, source_dir);

                instance.as_service().map(|service| ServiceRule {
                    unit_name: ServiceInstance::service_name(&service).to_owned(),
                    refuse_manual_start: service.spec.refuse_manual_start,
                    refuse_manual_stop: service.spec.refuse_manual_stop,
                })
            }));
    }

    gen_rules(&deb_dir, source, &services).expect("Failed to generate rules");

    Ok(())
}
