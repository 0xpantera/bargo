use color_eyre::Result;
use std::process::Command;
use tracing::{debug, error};

pub mod bb;
pub mod foundry;
pub mod garaga;
pub mod nargo;

/// Shared utility function to spawn and execute external commands
pub fn spawn_cmd(cmd_name: &str, args: &[&str]) -> Result<()> {
    debug!("Executing {} with args: {:?}", cmd_name, args);

    let mut cmd = Command::new(cmd_name);
    cmd.args(args);

    let output = cmd.output()?;

    // Print stdout and stderr
    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }

    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }

    // Check if command was successful
    if !output.status.success() {
        let exit_code = output.status.code().unwrap_or(-1);
        error!("{} command failed with exit code: {}", cmd_name, exit_code);
        return Err(color_eyre::eyre::eyre!(
            "{} {} failed with exit code {}",
            cmd_name,
            args.join(" "),
            exit_code
        ));
    }

    debug!("{} command completed successfully", cmd_name);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_cmd_success() {
        // Should succeed running a simple echo command
        assert!(spawn_cmd("echo", &["hello"]).is_ok());
    }

    #[test]
    fn test_spawn_cmd_failure() {
        // The `false` command exits with a non-zero status
        assert!(spawn_cmd("false", &[]).is_err());
    }
}
