use log::info;

use crate::{
    build_pipeline::{BuildContext, BuildError, BuildHandler},
    debcrafter_cmd::DebcrafterCmd,
};

#[derive(Default)]
pub struct CreateDebianDirHandle {
    debcrafter_version: String,
    build_files_dir: String,
    spec_file: String,
}

impl CreateDebianDirHandle {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_spec_file(mut self, spec_file: String) -> Self {
        self.spec_file = spec_file;
        self
    }

    pub fn with_build_files_dir(mut self, build_files_dir: String) -> Self {
        self.build_files_dir = build_files_dir;
        self
    }

    pub fn with_debcrafter_version(mut self, debcrafter_version: String) -> Self {
        self.debcrafter_version = debcrafter_version;
        self
    }
}
impl BuildHandler for CreateDebianDirHandle {
    fn handle(&self, _context: &mut BuildContext) -> Result<(), BuildError> {
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
