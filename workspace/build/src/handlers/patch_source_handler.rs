use crate::build_pipeline::{BuildContext, BuildError, BuildHandler};






#[derive(Default)]
pub struct PatchSourceHandle {}

impl PatchSourceHandle {
    pub fn new() -> Self {
        Self::default()
    }
}
impl BuildHandler for PatchSourceHandle {
    fn handle(&self, context: &mut BuildContext) -> Result<(), BuildError> {
        Ok(())
    }
}
