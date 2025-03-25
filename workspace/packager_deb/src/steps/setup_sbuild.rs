use crate::misc::build_pipeline::{BuildContext, BuildError, BuildStep};
use dirs::home_dir;
use std::fs;
use std::io::Write;

#[derive(Default)]
pub struct SetupSbuild {}

impl From<BuildContext> for SetupSbuild {
    fn from(_context: BuildContext) -> Self {
        SetupSbuild {}
    }
}
impl BuildStep for SetupSbuild {
    fn step(&self) -> Result<(), BuildError> {
        let home_dir = home_dir().ok_or(BuildError::HomeDirNotFound)?;
        let dest_path = home_dir.join(".sbuildrc");
        let content = include_str!("../.sbuildrc");
        let home_dir = home_dir.to_str().unwrap_or("/home/runner").to_string();
        let replaced_contents = content.replace("<HOME>", &home_dir);

        let mut file = fs::File::create(dest_path)
            .map_err(|err| BuildError::FileCreationError(err.to_string()))?;

        file.write_all(replaced_contents.as_bytes())
            .map_err(|err| BuildError::FileWriteError(err.to_string()))?;

        Ok(())
    }
}
