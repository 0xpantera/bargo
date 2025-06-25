//! Nargo operations have been migrated to the runner abstraction
//!
//! All nargo command executions now use the runner system through
//! `commands::common::run_nargo_command()` which provides:
//! - Unified dry-run handling
//! - Consistent logging
//! - Testable command execution
//! - Global flag propagation (--pkg, --verbose, etc.)
//!
//! The legacy `run()` function has been removed. Use the runner-based
//! approach in common commands instead.

// This module is kept for module structure compatibility
// All nargo functionality has moved to:
// - commands::common::run_nargo_command() for execution
// - commands::common::build_nargo_args() for argument building

#[cfg(test)]
mod tests {
    // Tests for nargo functionality should use the runner-based commands
    // See tests/cli_smoke.rs for examples of testing nargo commands through the CLI
}
