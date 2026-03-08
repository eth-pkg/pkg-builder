use std::ffi::OsStr;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

use log::info;

use crate::ToolError;

/// Run a command, inheriting stdout/stderr.
pub fn run_command<I, S>(cmd: &str, args: I, dir: Option<&Path>) -> Result<(), ToolError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(cmd);
    command
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    if let Some(dir) = dir {
        command.current_dir(dir);
    }

    execute_command(&mut command, cmd)
}

/// Run a command with sudo -S.
pub fn run_command_sudo<I, S>(cmd: &str, args: I, dir: Option<&Path>) -> Result<(), ToolError>
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

    execute_command(&mut command, &format!("sudo -S {}", cmd))
}

/// Run a command, capturing its stdout.
pub fn run_command_capture(cmd: &str, args: &[&str]) -> Result<String, ToolError> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| ToolError::NotFound {
            tool: cmd.to_string(),
            source: e,
        })?;

    if !output.status.success() {
        let code = output.status.code().unwrap_or(-1);
        return Err(ToolError::ExitCode {
            tool: cmd.to_string(),
            code,
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn execute_command(command: &mut Command, cmd_name: &str) -> Result<(), ToolError> {
    let mut child = command.spawn().map_err(|e| ToolError::NotFound {
        tool: cmd_name.to_string(),
        source: e,
    })?;

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                info!("{}", line);
            }
        }
    }

    let status = child.wait()?;

    if status.success() {
        Ok(())
    } else {
        let code = status.code().unwrap_or(-1);
        Err(ToolError::ExitCode {
            tool: cmd_name.to_string(),
            code,
        })
    }
}
