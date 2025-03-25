use std::{fs::create_dir_all, path::PathBuf};

use debian::{
    autopkgtest::Autopkgtest, autopkgtest_image::AutopkgtestImageBuilder, execute::Execute,
};
use log::info;
use types::{distribution::Distribution, version::Version};

use crate::{misc::distribution::DistributionTrait, sbuild::SbuildError};

use super::tool_runner::{BuildTool, ToolRunner};

pub struct AutopkgtestTool {
    version: Version,
    changes_file: PathBuf,
    codename: Distribution,
    deb_dir: PathBuf,
    test_deps: Vec<String>,
    image_path: Option<PathBuf>,
    cache_dir: PathBuf,
    arch: String,
}

impl AutopkgtestTool {
    pub fn new(
        version: Version,
        changes_file: PathBuf,
        codename: Distribution,
        deb_dir: PathBuf,
        test_deps: Vec<String>,
        cache_dir: PathBuf,
        arch: String,
    ) -> Self {
        AutopkgtestTool {
            version,
            changes_file,
            codename,
            image_path: None,
            deb_dir,
            test_deps,
            cache_dir,
            arch,
        }
    }
}

impl BuildTool for AutopkgtestTool {
    fn name(&self) -> &str {
        "autopkgtests"
    }
    fn version(&self) -> &Version {
        &self.version
    }
    fn configure(&mut self, _runner: &mut ToolRunner) -> Result<(), SbuildError> {
        info!("Running prepare_autopkgtest_image");
        let builder = AutopkgtestImageBuilder::new()
            .codename(&self.codename)?
            .image_path(
                &self.cache_dir.display().to_string(),
                &self.codename,
                &self.arch,
            )
            .mirror(self.codename.get_repo_url())
            .arch(&self.arch);
        let image_path = builder.get_image_path().unwrap();
        let image_path_parent = image_path.parent().unwrap();
        if !image_path.exists() {
            create_dir_all(image_path_parent)?;

            builder.execute()?;
        }

        self.image_path = Some(image_path.clone());
        Ok(())
    }
    fn execute(&self) -> Result<(), SbuildError> {
        Autopkgtest::new()
            .changes_file(self.changes_file.to_str().ok_or(SbuildError::GenericError(
                "Invalid changes file path".to_string(),
            ))?)
            .no_built_binaries()
            .apt_upgrade()
            .test_deps_not_in_debian(&&self.test_deps)
            .qemu(self.image_path.clone().unwrap())
            .working_dir(&self.deb_dir)
            .execute()?;
        Ok(())
    }
}
