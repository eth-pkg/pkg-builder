use debian::debcrafter::DebcrafterCmd;
use log::info;

use crate::build_pipeline::{BuildContext, BuildError, BuildStep};

#[derive(Default)]
pub struct CreateDebianDir {
    build_files_dir: String,
    debcrafter_version: String,
    spec_file: String,
}

impl From<BuildContext> for CreateDebianDir {
    fn from(context: BuildContext) -> Self {
        CreateDebianDir {
            build_files_dir: context.build_files_dir.clone(),
            debcrafter_version: context.debcrafter_version.clone(),
            spec_file: context.spec_file.clone(),
        }
    }
}

impl BuildStep for CreateDebianDir {
    fn step(&self) -> Result<(), BuildError> {
        let debcrafter = DebcrafterCmd::new(self.debcrafter_version.as_str());
        debcrafter.check_if_dpkg_parsechangelog_installed()?;
        debcrafter.check_if_installed()?;
        debcrafter.create_debian_dir(self.spec_file.as_str(), self.build_files_dir.as_str())?;
        info!(
            "Created /debian dir under build_files_dir folder: {:?}",
            self.build_files_dir
        );
        Ok(())
    }
}
