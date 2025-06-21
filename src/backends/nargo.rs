use color_eyre::Result;
use std::process::Command;
use tracing::{debug, error};

pub fn run(args: &[&str]) -> Result<()> {
    debug!("Executing nargo with args: {:?}", args);

    let mut cmd = Command::new("nargo");
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
        error!("nargo command failed with exit code: {}", exit_code);
        return Err(color_eyre::eyre::eyre!(
            "nargo {} failed with exit code {}",
            args.join(" "),
            exit_code
        ));
    }

    debug!("nargo command completed successfully");
    Ok(())
}
