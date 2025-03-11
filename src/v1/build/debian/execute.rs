use std::{
    ffi::OsStr, io::{BufRead, BufReader}, path::Path, process::{Command, Stdio}
};
use eyre::{eyre,Result};

use log::info;

pub trait Execute {
    fn execute(&self) -> Result<()>;
}

pub fn execute_command<I, S>(
    cmd: &str,
    args: I,
    dir: Option<&Path>,
) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(cmd);
    if let Some(dir) = dir {
        command.current_dir(dir);
    }
    command
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .status()?;
    Ok(())
}

pub fn execute_command_with_sudo(
    cmd: &str,
    args: Vec<String>,
    target: &Path,
    dir: Option<&Path>,
) -> Result<()> {
    let mut command = Command::new("sudo");
    command
        .arg("-S")
        .arg(cmd)
        .args(args)
        .arg(target)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());
    if let Some(dir) = dir {
        command.current_dir(dir);
    }

    run_command(&mut command, &format!("sudo -S {}", cmd))
}

fn run_command(
    command: &mut Command,
    cmd_name: &str,
) -> Result<()> {
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
        Err(eyre!(
            "Command '{}' failed with status: {}",
            cmd_name,
            status
        ))
    }
}
