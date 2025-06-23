// Utility functions for colored CLI output
// Extracted from util.rs

/// ANSI color codes for terminal output
pub mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const GREEN: &str = "\x1b[32m";
    pub const RED: &str = "\x1b[31m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const CYAN: &str = "\x1b[36m";
    pub const GRAY: &str = "\x1b[90m";
    pub const BRIGHT_GREEN: &str = "\x1b[92m";
    pub const BRIGHT_BLUE: &str = "\x1b[94m";
    pub const BRIGHT_CYAN: &str = "\x1b[96m";
}

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

/// Create error message with red color
pub fn error(text: &str) -> String {
    colorize(&format!("❌ {}", text), colors::RED)
}

/// Create warning message with yellow color
pub fn warning(text: &str) -> String {
    colorize(&format!("⚠️ {}", text), colors::YELLOW)
}

/// Create info message with blue color
pub fn info(text: &str) -> String {
    colorize(&format!("ℹ️ {}", text), colors::BRIGHT_BLUE)
}

/// Create path text with cyan color
pub fn path(text: &str) -> String {
    colorize(text, colors::BRIGHT_CYAN)
}

/// ASCII art banners for different operations
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
            "┌────────────────────────────────┐\n\
             │ 🚀 RUNNING BARGO OPERATION      │\n\
             └─────────────────────────────────┘"
        }
    };

    println!("{}", colorize(banner, colors::BRIGHT_BLUE));
}
