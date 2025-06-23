use color_eyre::Result;

/// Ensure Foundry (forge and cast) is available on the system
pub fn ensure_available() -> Result<()> {
    // Check for forge
    which::which("forge").map_err(|_| {
        color_eyre::eyre::eyre!(
            "❌ forge command not found\n\n\
             EVM features require Foundry to be installed.\n\n\
             📋 Installation steps:\n\
             1. Install Foundry: curl -L https://foundry.paradigm.xyz | bash\n\
             2. Restart your terminal or run: source ~/.bashrc\n\
             3. Update Foundry: foundryup\n\
             4. Verify: forge --version && cast --version\n\n\
             🔧 Alternative installation:\n\
             • From source: https://github.com/foundry-rs/foundry\n\
             • Via package manager (brew, etc.)\n\n\
             💡 You can still use all Cairo/Starknet features without Foundry!\n\
             Run 'bargo doctor' to check all dependencies."
        )
    })?;

    // Check for cast
    which::which("cast").map_err(|_| {
        color_eyre::eyre::eyre!(
            "❌ cast command not found\n\n\
             EVM features require Foundry (including cast) to be installed.\n\n\
             📋 Installation steps:\n\
             1. Install Foundry: curl -L https://foundry.paradigm.xyz | bash\n\
             2. Restart your terminal or run: source ~/.bashrc\n\
             3. Update Foundry: foundryup\n\
             4. Verify: forge --version && cast --version\n\n\
             🔧 Troubleshooting:\n\
             • Try: foundryup (to update/reinstall)\n\
             • Check PATH includes ~/.foundry/bin\n\
             • Restart terminal after installation\n\n\
             💡 You can still use all Cairo/Starknet features without Foundry!\n\
             Run 'bargo doctor' to check all dependencies."
        )
    })?;

    Ok(())
}

/// Execute a forge command with the given arguments
pub fn run_forge(args: &[&str]) -> Result<()> {
    // Ensure Foundry is available before running
    ensure_available()?;

    // Use the common spawn_cmd function from the parent module
    super::spawn_cmd("forge", args).map_err(|e| {
        color_eyre::eyre::eyre!(
            "{}\n\n\
             Troubleshooting:\n\
             • Ensure Foundry is properly installed: foundryup\n\
             • Check that forge is in your PATH\n\
             • Verify .env file has required variables (RPC_URL, PRIVATE_KEY)\n\
             • Try running the forge command directly to see more details",
            e
        )
    })
}

/// Execute a cast command with the given arguments
pub fn run_cast(args: &[&str]) -> Result<()> {
    // Ensure Foundry is available before running
    ensure_available()?;

    // Use the common spawn_cmd function from the parent module
    super::spawn_cmd("cast", args).map_err(|e| {
        color_eyre::eyre::eyre!(
            "{}\n\n\
             Troubleshooting:\n\
             • Ensure Foundry is properly installed: foundryup\n\
             • Check that cast is in your PATH\n\
             • Verify .env file has required variables (RPC_URL, PRIVATE_KEY)\n\
             • Try running the cast command directly to see more details",
            e
        )
    })
}

/// Execute a forge command and capture its output
pub fn run_forge_with_output(args: &[&str]) -> Result<(String, String)> {
    // Ensure Foundry is available before running
    ensure_available()?;

    let output = std::process::Command::new("forge")
        .args(args)
        .output()
        .map_err(|e| {
            color_eyre::eyre::eyre!(
                "Failed to execute forge command: {}\n\n\
                 Troubleshooting:\n\
                 • Ensure Foundry is properly installed: foundryup\n\
                 • Check that forge is in your PATH\n\
                 • Verify .env file has required variables (RPC_URL, PRIVATE_KEY)\n\
                 • Try running the forge command directly to see more details",
                e
            )
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(color_eyre::eyre::eyre!(
            "Forge command failed with exit code: {}\nStdout: {}\nStderr: {}",
            output.status.code().unwrap_or(-1),
            stdout,
            stderr
        ));
    }

    Ok((stdout, stderr))
}

/// Execute a cast command and capture its output
pub fn run_cast_with_output(args: &[&str]) -> Result<(String, String)> {
    // Ensure Foundry is available before running
    ensure_available()?;

    let output = std::process::Command::new("cast")
        .args(args)
        .output()
        .map_err(|e| {
            color_eyre::eyre::eyre!(
                "Failed to execute cast command: {}\n\n\
                 Troubleshooting:\n\
                 • Ensure Foundry is properly installed: foundryup\n\
                 • Check that cast is in your PATH\n\
                 • Verify .env file has required variables (RPC_URL, PRIVATE_KEY)\n\
                 • Try running the cast command directly to see more details",
                e
            )
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(color_eyre::eyre::eyre!(
            "Cast command failed with exit code: {}\nStdout: {}\nStderr: {}",
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
        // This test will pass if Foundry is installed, otherwise it will show
        // the helpful error message
        match ensure_available() {
            Ok(_) => println!("{}", crate::util::success("Foundry (forge and cast) is available")),
            Err(e) => println!("{}", crate::util::error(&format!("Foundry not available: {}", e))),
        }
    }
}
