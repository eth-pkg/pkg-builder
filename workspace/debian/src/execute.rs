use std::{
    ffi::OsStr,
    io::{BufRead, BufReader},
    path::Path,
    process::{Command, Stdio},
};
use thiserror::Error;
use log::info;

#[derive(Error, Debug)]
pub enum ExecuteError {
    #[error("Command execution failed: {0}")]
    CommandFailed(#[from] std::io::Error),
    
    #[error("Failed to change working directory: {0}")]
    WorkingDirectoryError(String),
    
    #[error("Command '{0}' failed with status: {1}")]
    CommandStatusError(String, i32),
}

pub trait Execute {
    type Error;
    fn execute(&self) -> Result<(), Self::Error>;
}

pub fn execute_command<I, S>(cmd: &str, args: I, dir: Option<&Path>) -> Result<(), ExecuteError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(cmd);
    command.args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    
    if let Some(dir) = dir {
        command.current_dir(dir);
    }
    
    run_command(&mut command, cmd)
}

pub fn execute_command_with_sudo<I, S>(cmd: &str, args: I, dir: Option<&Path>) -> Result<(), ExecuteError> 
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new("sudo");
    command
        .arg("-S")
        .arg(cmd)
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    
    if let Some(dir) = dir {
        command.current_dir(dir);
    }
    
    run_command(&mut command, &format!("sudo -S {}", cmd))
}

fn run_command(command: &mut Command, cmd_name: &str) -> Result<(), ExecuteError> {
    let mut child = command.spawn()?;
    
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            info!("{}", line?);
        }
    }
    
    let status = child.wait()?;
    
    if status.success() {
        Ok(())
    } else {
        let code = status.code().unwrap_or(-1);
        Err(ExecuteError::CommandStatusError(cmd_name.to_string(), code))
    }
}