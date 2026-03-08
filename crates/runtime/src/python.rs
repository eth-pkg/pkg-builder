use crate::Runtime;

pub struct PythonRuntime;

impl Runtime for PythonRuntime {
    fn install_commands(&self) -> Vec<String> {
        vec![]
    }
}
