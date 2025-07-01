//! Logging and output formatting utilities for bargo
//!
//! This module provides utilities for colored terminal output, logging messages,
//! and formatted banners used throughout the bargo workflow.
//!
//! ## Key Features
//!
//! - Colored terminal output with automatic NO_COLOR support
//! - Success, info, and error message formatting
//! - ASCII art banners for different operations
//! - TTY detection for proper color handling
//!
//! ## Examples
//!
//! ```ignore
//! use bargo_core::util::log::{success, info, print_banner};
//!
//! // Print colored success message
//! println!("{}", success("Build completed successfully"));
//!
//! // Print info message
//! println!("{}", info("Starting verification process"));
//!
//! // Print operation banner
//! print_banner("build");
//! ```

/// ANSI color codes for terminal output
pub mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const GREEN: &str = "\x1b[32m";
    pub const GRAY: &str = "\x1b[90m";
    pub const BRIGHT_GREEN: &str = "\x1b[92m";
    pub const BRIGHT_BLUE: &str = "\x1b[94m";
    pub const BRIGHT_CYAN: &str = "\x1b[96m";
}

// Placeholder functions - these will be moved here from other modules
// in Checkpoint B

/// Format text with color
pub fn colorize(text: &str, color: &str) -> String {
    if std::env::var("NO_COLOR").is_ok() || !atty::is(atty::Stream::Stdout) {
        text.to_string()
    } else {
        format!("{}{}{}", color, text, colors::RESET)
    }
}

/// Create success message with green color
pub fn success(text: &str) -> String {
    colorize(&format!("✅ {}", text), colors::BRIGHT_GREEN)
}

/// Create info message with blue color
pub fn info(text: &str) -> String {
    colorize(&format!("ℹ️ {}", text), colors::BRIGHT_BLUE)
}

/// Print ASCII art banners for different operations
pub fn print_banner(operation: &str) {
    let banner = match operation {
        "build" => {
            "┌─────────────────────────────────┐\n\
             │ 🔨 BUILDING NOIR CIRCUIT       │\n\
             └─────────────────────────────────┘"
        }
        "prove" => {
            "┌─────────────────────────────────┐\n\
             │ 🔐 GENERATING PROOF & VK        │\n\
             └─────────────────────────────────┘"
        }
        "verify" => {
            "┌─────────────────────────────────┐\n\
             │ ✅ VERIFYING PROOF              │\n\
             └─────────────────────────────────┘"
        }
        "solidity" => {
            "┌─────────────────────────────────┐\n\
             │ 📄 GENERATING SOLIDITY VERIFIER │\n\
             └─────────────────────────────────┘"
        }
        "clean" => {
            "┌─────────────────────────────────┐\n\
             │ 🧹 CLEANING BUILD ARTIFACTS    │\n\
             └─────────────────────────────────┘"
        }
        "check" => {
            "┌─────────────────────────────────┐\n\
             │ 🔍 CHECKING CIRCUIT SYNTAX      │\n\
             └─────────────────────────────────┘"
        }
        _ => {
            "┌─────────────────────────────────┐\n\
             │ 🚀 RUNNING BARGO OPERATION      │\n\
             └─────────────────────────────────┘"
        }
    };

    println!("{}", colorize(banner, colors::BRIGHT_BLUE));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success() {
        let result = success("test message");
        assert!(result.contains("✅"));
        assert!(result.contains("test message"));
    }

    #[test]
    fn test_info() {
        let result = info("test info");
        assert!(result.contains("ℹ️"));
        assert!(result.contains("test info"));
    }

    #[test]
    fn test_colorize() {
        let result = colorize("test", colors::GREEN);
        // Should contain the text (color codes may vary based on environment)
        assert!(result.contains("test"));
    }

    #[test]
    fn test_print_banner() {
        // This test just ensures the function doesn't panic
        print_banner("build");
        print_banner("prove");
        print_banner("unknown_operation");
    }
}
