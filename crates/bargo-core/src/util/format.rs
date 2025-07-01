//! Text and data formatting utilities for bargo
//!
//! This module provides utilities for formatting various types of data including
//! file sizes, operation results, and path displays used throughout the bargo workflow.
//!
//! ## Key Features
//!
//! - Human-readable file size formatting
//! - Operation result formatting with timing information
//! - Path text formatting with color
//! - Consistent data presentation across commands
//!
//! ## Examples
//!
//! ```ignore
//! use bargo_core::util::format::{format_file_size, format_operation_result, path};
//! use bargo_core::util::timer::Timer;
//! use std::path::Path;
//!
//! // Format file size
//! let size = format_file_size(Path::new("./target/proof"));
//! println!("File size: {}", size);
//!
//! // Format operation result with timing
//! let timer = Timer::start();
//! // ... do work ...
//! let result = format_operation_result("Proving", Path::new("./proof"), &timer);
//! println!("{}", result);
//!
//! // Format path with color
//! println!("Output: {}", path("./target/contract.sol"));
//! ```

use std::path::Path;

// Placeholder functions - these will be moved here from other modules
// in Checkpoint B

/// Format file size in human-readable format
pub fn format_file_size(path: &Path) -> String {
    match std::fs::metadata(path) {
        Ok(metadata) => {
            let size = metadata.len();
            if size < 1024 {
                format!("{size} B")
            } else if size < 1024 * 1024 {
                format!("{:.1} KB", size as f64 / 1024.0)
            } else {
                format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
            }
        }
        Err(_) => "unknown size".to_string(),
    }
}

/// Format operation result with file size and timing
pub fn format_operation_result(
    operation: &str,
    file_path: &Path,
    timer: &crate::util::timer::Timer,
) -> String {
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

/// Create path text with cyan color
pub fn path(text: &str) -> String {
    crate::util::log::colorize(text, crate::util::log::colors::BRIGHT_CYAN)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_format_file_size() {
        // Test with a file that doesn't exist (should return "unknown size")
        let result = format_file_size(Path::new("nonexistent_file.txt"));
        assert_eq!(result, "unknown size");
    }

    #[test]
    fn test_format_operation_result() {
        let timer = crate::util::timer::Timer::start();
        let result = format_operation_result("Test", Path::new("nonexistent.txt"), &timer);

        // Should contain the operation name and file path
        assert!(result.contains("Test"));
        assert!(result.contains("nonexistent.txt"));
        assert!(result.contains("unknown size")); // Since file doesn't exist
    }

    #[test]
    fn test_path_formatting() {
        let result = path("test/path");
        // Should contain the path text (color codes may vary based on environment)
        assert!(result.contains("test/path"));
    }
}
