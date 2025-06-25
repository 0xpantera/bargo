use color_eyre::Result;

/// Ensure garaga is available on the system
pub fn ensure_available() -> Result<()> {
    which::which("garaga").map_err(|_| {
        color_eyre::eyre::eyre!(
            "❌ garaga command not found\n\n\
             Cairo/Starknet features require garaga to be installed.\n\n\
             📋 Installation steps:\n\
             1. Check Python version: python3 --version (need 3.10+)\n\
             2. Install pipx: python3 -m pip install pipx\n\
             3. Install garaga: pipx install garaga\n\
             4. Verify: garaga --help\n\n\
             🔧 Alternative installation:\n\
             • With pip: pip install garaga\n\
             • From source: https://github.com/keep-starknet-strange/garaga\n\n\
             💡 You can still use all EVM features without garaga!\n\
             Run 'bargo doctor' to check all dependencies."
        )
    })?;
    Ok(())
}

/// Execute a garaga command and capture its output
pub fn run_with_output(args: &[&str]) -> Result<(String, String)> {
    // Ensure garaga is available before running
    ensure_available()?;

    let output = std::process::Command::new("garaga")
        .args(args)
        .output()
        .map_err(|e| {
            color_eyre::eyre::eyre!(
                "Failed to execute garaga command: {}\n\n\
                 Troubleshooting:\n\
                 • Ensure garaga is properly installed: pipx install garaga\n\
                 • Check that Python 3.10+ is available\n\
                 • Verify garaga is in your PATH",
                e
            )
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(color_eyre::eyre::eyre!(
            "Garaga command failed with exit code: {}\nStdout: {}\nStderr: {}",
            output.status.code().unwrap_or(-1),
            stdout,
            stderr
        ));
    }

    Ok((stdout, stderr))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_available() {
        // This test will pass if garaga is installed, otherwise it will show
        // the helpful error message
        match ensure_available() {
            Ok(_) => println!("✓ garaga is available"),
            Err(e) => println!("✗ garaga not available: {}", e),
        }
    }
}
