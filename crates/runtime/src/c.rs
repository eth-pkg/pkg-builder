use crate::Runtime;

pub struct CRuntime;

impl Runtime for CRuntime {
    fn install_commands(&self) -> Vec<String> {
        vec![]
    }
}
