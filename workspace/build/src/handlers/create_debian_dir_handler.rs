use crate::build_pipeline::{BuildContext, BuildError, BuildHandler};



#[derive(Default)]
pub struct CreateDebianDirHandle {}

impl CreateDebianDirHandle {
    pub fn new() -> Self {
        Self::default()
    }
}
impl BuildHandler for CreateDebianDirHandle {
    fn handle(&self, _context: &mut BuildContext) -> Result<(), BuildError> {
        Ok(())
    }
}