//! EVM workflow module for Ethereum backend operations
//!
//! This module provides a clean, modular interface for EVM/Ethereum operations
//! including proof generation, verification, contract management, and deployment.

pub mod backend;
pub mod bb_operations;
pub mod directories;
pub mod error;
pub mod workflow;

#[cfg(feature = "evm-foundry")]
pub mod foundry;

// Re-export main workflow functions for use by main.rs
pub use workflow::{run_calldata, run_gen, run_prove, run_verify};

#[cfg(feature = "evm-foundry")]
pub use workflow::{run_deploy, run_verify_onchain};

// Re-export error types for convenience
pub use error::{EvmError, Result};

// Re-export utility functions that may be needed elsewhere
// (Currently none are needed externally, but modules are available for import)

// Helper function to load environment variables (shared across workflow)
pub fn load_env_vars() {
    dotenv::dotenv().ok();
    if std::path::Path::new(".env").exists() {
        let _ = dotenv::from_filename(".env");
    }
}
