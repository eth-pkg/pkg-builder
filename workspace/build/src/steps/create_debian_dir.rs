use log::info;

use crate::{
    build_pipeline::{BuildContext, BuildError, BuildStep},
    debcrafter_cmd::DebcrafterCmd,
};

#[derive(Default)]
pub struct CreateDebianDir {
}

impl CreateDebianDir {
    pub fn new() -> Self {
        Self::default()
    }
}
impl BuildStep for CreateDebianDir {
    fn step(&self, context: &mut BuildContext) -> Result<(), BuildError> {
        let debcrafter = DebcrafterCmd::new(context.debcrafter_version.as_str());
        debcrafter.check_if_dpkg_parsechangelog_installed()?;
        debcrafter.check_if_installed()?;
        debcrafter.create_debian_dir(context.spec_file.as_str(), context.build_files_dir.as_str())?;
        info!(
            "Created /debian dir under build_files_dir folder: {:?}",
            context.build_files_dir
        );
        Ok(())
    }
}
