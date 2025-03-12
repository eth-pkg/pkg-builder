use crate::build_pipeline::{BuildContext, BuildError, BuildHandler};


#[derive(Default)]
pub struct ExtractSourceHandler {}

impl ExtractSourceHandler {
    pub fn new() -> Self {
        Self::default()
    }
}
impl BuildHandler for ExtractSourceHandler {
    fn handle(&self, _context: &mut BuildContext) -> Result<(), BuildError> {
        Ok(())
    }
}
