//! Backend modules for external tool integrations
//!
//! All command executions have been migrated to use the runner abstraction
//! through `commands::common::run_*_command()` functions which provide:
//! - Unified dry-run handling
//! - Consistent logging
//! - Testable command execution
//!
//! Legacy direct command execution functions have been removed.

pub mod bb;
pub mod foundry;
pub mod garaga;
pub mod nargo;

#[cfg(test)]
mod tests {
    // Tests for backend functionality should use the runner-based commands
    // See tests/cli_smoke.rs for examples of testing commands through the CLI
}
