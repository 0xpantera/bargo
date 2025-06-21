use color_eyre::Result;

/// Ensure garaga is available on the system
pub fn ensure_available() -> Result<()> {
    which::which("garaga").map_err(|_| {
        color_eyre::eyre::eyre!(
            "garaga command not found. Please install garaga:\n\n  \
             pipx install garaga\n\n\
             Garaga requires Python 3.10 or higher."
        )
    })?;
    Ok(())
}

/// Execute a garaga command with the given arguments
pub fn run(args: &[&str]) -> Result<()> {
    // Ensure garaga is available before running
    ensure_available()?;

    // Use the common spawn_cmd function from the parent module
    super::spawn_cmd("garaga", args).map_err(|e| {
        color_eyre::eyre::eyre!(
            "{}\n\n\
             Troubleshooting:\n\
             • Ensure garaga is properly installed: pipx install garaga\n\
             • Check that Python 3.10+ is available\n\
             • Verify garaga is in your PATH",
            e
        )
    })
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
