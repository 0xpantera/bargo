pub mod paths;

pub use paths::{
    Flavour,
    get_package_name,
    get_bytecode_path,
    get_witness_path,
    get_proof_path,
    get_vk_path,
    get_public_inputs_path,
    organize_build_artifacts,
    validate_files_exist,
    needs_rebuild,
};

use std::path::Path;

/// Timer for tracking operation duration
pub struct Timer {
    start: std::time::Instant,
}

impl Timer {
    /// Start a new timer
    pub fn start() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }

    /// Get elapsed time as a formatted string
    pub fn elapsed(&self) -> String {
        let duration = self.start.elapsed();
        if duration.as_secs() > 0 {
            format!("{:.1}s", duration.as_secs_f64())
        } else {
            format!("{}ms", duration.as_millis())
        }
    }
}

/// Format file size in human-readable format
pub fn format_file_size(path: &Path) -> String {
    match std::fs::metadata(path) {
        Ok(metadata) => {
            let size = metadata.len();
            if size < 1024 {
                format!("{} B", size)
            } else if size < 1024 * 1024 {
                format!("{:.1} KB", size as f64 / 1024.0)
            } else {
                format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
            }
        }
        Err(_) => "unknown size".to_string(),
    }
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

/// Format operation result with file size and timing
pub fn format_operation_result(operation: &str, file_path: &Path, timer: &Timer) -> String {
    let size = format_file_size(file_path);
    let elapsed = timer.elapsed();
    format!(
        "{} → {} ({}, {})",
        operation,
        file_path.display(),
        size,
        elapsed
    )
}

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
            "┌─────────────────────────────────┐\n\
             │ 🚀 RUNNING BARGO OPERATION      │\n\
             └─────────────────────────────────┘"
        }
    };

    println!("{}", colorize(banner, colors::BRIGHT_BLUE));
}

/// Print operation summary with colored output
pub struct OperationSummary {
    operations: Vec<String>,
    start_time: std::time::Instant,
}

impl OperationSummary {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
            start_time: std::time::Instant::now(),
        }
    }

    pub fn add_operation(&mut self, operation: &str) {
        self.operations.push(operation.to_string());
    }

    pub fn print(&self) {
        if self.operations.is_empty() {
            return;
        }

        let total_time = self.start_time.elapsed();
        let time_str = if total_time.as_secs() > 0 {
            format!("{:.1}s", total_time.as_secs_f64())
        } else {
            format!("{}ms", total_time.as_millis())
        };

        println!("\n{}", colorize("🎉 Summary:", colors::BOLD));
        for operation in &self.operations {
            println!(
                "   {}",
                colorize(&format!("• {}", operation), colors::GREEN)
            );
        }
        println!(
            "   {}",
            colorize(&format!("Total time: {}", time_str), colors::GRAY)
        );
    }
}
