//! Cairo backend implementation for Starknet proof systems
//!
//! This module provides a Cairo backend that implements the BackendTrait,
//! wrapping the existing Cairo workflow functions to provide a unified interface.

use color_eyre::Result;

use crate::{backend::Backend, config::Config};

use super::workflow;

/// Cairo backend implementation for Starknet-based proof systems
#[derive(Debug)]
pub struct CairoBackend;

impl CairoBackend {
    /// Create a new Cairo backend instance
    pub fn new() -> Self {
        Self
    }
}

impl Backend for CairoBackend {
    /// Generate Cairo verifier contract and setup project structure
    fn generate(&self, cfg: &Config) -> Result<()> {
        workflow::run_gen(cfg)
    }

    /// Generate proof using Cairo/Starknet proof system
    fn prove(&self, cfg: &Config) -> Result<()> {
        workflow::run_prove(cfg)
    }

    /// Verify a generated Cairo proof
    fn verify(&self, cfg: &Config) -> Result<()> {
        workflow::run_verify(cfg)
    }

    /// Generate calldata for Cairo proof verification
    fn calldata(&self, cfg: &Config) -> Result<()> {
        workflow::run_calldata(cfg)
    }

    /// Deploy Cairo verifier contract to Starknet network
    ///
    /// Cairo deployment is a two-step process:
    /// 1. Declare the contract on the network to get a class_hash
    /// 2. Deploy an instance of the contract using the class_hash
    fn deploy(&self, cfg: &Config, network: Option<&str>) -> Result<()> {
        // Use provided network or default to "sepolia"
        let network_str = network.unwrap_or("sepolia");

        // Step 1: Declare the contract to get class_hash
        workflow::run_declare(cfg, network_str)?;

        // Step 2: Deploy the contract using the class_hash from declare
        workflow::run_deploy(cfg, None)
    }

    /// Verify proof on-chain using deployed Cairo verifier on Starknet
    fn verify_onchain(&self, cfg: &Config, address: Option<&str>) -> Result<()> {
        workflow::run_verify_onchain(cfg, address)
    }
}

impl Default for CairoBackend {
    fn default() -> Self {
        Self::new()
    }
}
