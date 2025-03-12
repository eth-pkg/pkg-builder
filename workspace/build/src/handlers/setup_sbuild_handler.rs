use std::fs;
use std::io::Write;
use crate::build_pipeline::{BuildContext, BuildError, BuildHandler};
use dirs::home_dir;

#[derive(Default)]
pub struct SetupSbuildHandle {}

impl SetupSbuildHandle {
    pub fn new() -> Self {
        Self::default()
    }
}

impl BuildHandler for SetupSbuildHandle {
    fn handle(&self, context: &mut BuildContext) -> Result<(), BuildError> {
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