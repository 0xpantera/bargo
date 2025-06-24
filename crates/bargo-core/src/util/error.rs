/// Create a smart error with context and suggestions
pub fn create_smart_error(message: &str, suggestions: &[&str]) -> color_eyre::eyre::Error {
    let mut error_msg = format!("❌ {}", message);

    if !suggestions.is_empty() {
        error_msg.push_str("\n\n💡 Suggestions:");
        for suggestion in suggestions {
            error_msg.push_str(&format!("\n   • {}", suggestion));
        }
    }

    color_eyre::eyre::eyre!(error_msg)
}

/// Enhanced error with suggestions for common issues
pub fn enhance_error_with_suggestions(error: color_eyre::eyre::Error) -> color_eyre::eyre::Error {
    let error_msg = format!("{}", error);

    // Check for common error patterns and add suggestions
    if error_msg.contains("Required files are missing") {
        if error_msg.contains(".json") || error_msg.contains(".gz") {
            return color_eyre::eyre::eyre!(
                "{}\n\n💡 Suggestions:\n   • Run `bargo build` to generate bytecode and witness files\n   • Check if you're in the correct Noir project directory\n   • Verify that Nargo.toml exists in the current directory",
                error
            );
        }
        if error_msg.contains("proof") || error_msg.contains("vk") {
            return color_eyre::eyre::eyre!(
                "{}\n\n💡 Suggestions:\n   • Run `bargo prove` to generate proof and verification key\n   • Check if the proving step completed successfully\n   • Try running `bargo clean` and rebuilding from scratch",
                error
            );
        }
    }

    if error_msg.contains("Could not find Nargo.toml") {
        return color_eyre::eyre::eyre!(
            "{}\n\n💡 Suggestions:\n   • Make sure you're in a Noir project directory\n   • Initialize a new project with `nargo new <project_name>`\n   • Check if you're in a subdirectory - try running from the project root",
            error
        );
    }

    if error_msg.contains("Failed to parse Nargo.toml") {
        return color_eyre::eyre::eyre!(
            "{}\n\n💡 Suggestions:\n   • Check Nargo.toml syntax - it should be valid TOML format\n   • Ensure the [package] section has a 'name' field\n   • Compare with a working Nargo.toml from another project",
            error
        );
    }

    if error_msg.contains("nargo")
        && (error_msg.contains("not found") || error_msg.contains("command not found"))
    {
        return color_eyre::eyre::eyre!(
            "{}\n\n💡 Suggestions:\n   • Install nargo: `curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash`\n   • Add nargo to your PATH\n   • Verify installation with `nargo --version`",
            error
        );
    }

    if error_msg.contains("bb")
        && (error_msg.contains("not found") || error_msg.contains("command not found"))
    {
        return color_eyre::eyre::eyre!(
            "{}\n\n💡 Suggestions:\n   • Install bb (Barretenberg): check Aztec installation docs\n   • Add bb to your PATH\n   • Verify installation with `bb --version`",
            error
        );
    }

    // Return original error if no patterns match
    error
}
