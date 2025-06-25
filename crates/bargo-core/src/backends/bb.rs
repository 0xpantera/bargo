//! BB operations have been migrated to the runner abstraction
//!
//! All bb command executions now use the runner system through
//! `commands::common::run_bb_command()` which provides:
//! - Unified dry-run handling
//! - Consistent logging
//! - Testable command execution
//!
//! The legacy `run()` function has been removed. Use the runner-based
//! approach in bb_operations modules instead.

// This module is kept for module structure compatibility
// All bb functionality has moved to:
// - commands::common::run_bb_command() for execution
// - commands::cairo::bb_operations for Cairo-specific operations
// - commands::evm::bb_operations for EVM-specific operations
