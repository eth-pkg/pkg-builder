pub struct CommandBuilder {
    commands: Vec<String>,
}

impl CommandBuilder {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn add(&mut self, cmd: impl Into<String>) -> &mut Self {
        self.commands.push(cmd.into());
        self
    }

    pub fn add_with(&mut self, template: &str, replacement: &str) -> &mut Self {
        let cmd = template.replace("{}", replacement);
        self.commands.push(cmd);
        self
    }

    pub fn add_with_args(&mut self, template: &str, args: &[&str]) -> &mut Self {
        // Format the string with positional arguments
        let mut result = template.to_string();
        for arg in args {
            // Replace the first occurrence of {} with the argument
            if let Some(pos) = result.find("{}") {
                result.replace_range(pos..pos + 2, arg);
            }
        }
        self.commands.push(result);
        self
    }

    pub fn build(self) -> Vec<String> {
        self.commands
    }
}
