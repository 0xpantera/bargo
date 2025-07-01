//! Garaga operations have been migrated to the runner abstraction
//!
//! All garaga command executions now use the runner system through:
//! - `commands::common::run_tool(cfg, "garaga", args)` for basic execution
//! - `commands::common::run_tool_capture(cfg, "garaga", args)` for stdout capture
//!
//! This provides:
//! - Unified dry-run handling
//! - Consistent logging
//! - Testable command execution through DryRunRunner history
//!
//! The actual garaga functionality is implemented in:
//! - `commands::cairo::garaga` for high-level operations
//! - `commands::cairo::workflow` for integrated workflows

// This module is kept for module structure compatibility
// All garaga functionality has moved to the runner-based approach

#[cfg(test)]
mod tests {
    // Tests for garaga functionality should use the runner-based commands
    // See tests/cli_smoke.rs for examples of testing garaga commands through the CLI
}
