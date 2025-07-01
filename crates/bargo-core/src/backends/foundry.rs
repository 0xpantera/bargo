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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_available() {
        // This test will pass if Foundry is installed, otherwise it will show
        // the helpful error message
        match ensure_available() {
            Ok(_) => println!("✓ Foundry (forge and cast) is available"),
            Err(e) => println!("✗ Foundry not available: {}", e),
        }
    }
}
