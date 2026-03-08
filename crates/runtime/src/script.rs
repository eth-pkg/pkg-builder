/// A builder for constructing lists of shell commands.
/// Each command becomes a chroot-setup-command for sbuild.
pub struct ScriptBuilder {
    commands: Vec<String>,
}

impl ScriptBuilder {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn cmd(&mut self, command: impl Into<String>) -> &mut Self {
        self.commands.push(command.into());
        self
    }

    pub fn build(self) -> Vec<String> {
        self.commands
    }
}
