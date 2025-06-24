//! EVM workflow module for Ethereum backend operations
//!
//! This module provides a clean, modular interface for EVM/Ethereum operations
//! including proof generation, verification, contract management, and deployment.

pub mod bb_operations;
pub mod directories;
pub mod foundry;
pub mod workflow;

// Re-export main workflow functions for use by main.rs
pub use workflow::{run_calldata, run_deploy, run_gen, run_prove, run_verify, run_verify_onchain};

// Re-export utility functions that may be needed elsewhere
// (Currently none are needed externally, but modules are available for import)

// Helper function to load environment variables (shared across workflow)
pub fn load_env_vars() {
    dotenv::dotenv().ok();
    if std::path::Path::new(".env").exists() {
        let _ = dotenv::from_filename(".env");
    }
}
