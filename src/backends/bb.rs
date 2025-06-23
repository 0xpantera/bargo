use color_eyre::Result;
use std::process::Command;
use tracing::{debug, error};

pub fn run(args: &[&str]) -> Result<()> {
    debug!("Executing bb with args: {:?}", args);

    let mut cmd = Command::new("bb");
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
        error!("bb command failed with exit code: {}", exit_code);
        return Err(color_eyre::eyre::eyre!(
            "bb {} failed with exit code {}",
            args.join(" "),
            exit_code
        ));
    }

    debug!("bb command completed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_version() {
        // Call bb --version if available; just ensure function doesn't panic
        let _ = run(&["--version"]);
    }
}
