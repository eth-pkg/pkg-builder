use crate::build_pipeline::{BuildContext, BuildError, BuildHandler};

#[derive(Default)]
pub struct SetupSbuildHandle {}

impl SetupSbuildHandle {
    pub fn new() -> Self {
        Self::default()
    }
}
impl BuildHandler for SetupSbuildHandle {
    fn handle(&self, context: &mut BuildContext) -> Result<(), BuildError> {
        Ok(())
    }
}
