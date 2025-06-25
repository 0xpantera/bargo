//! EVM backend implementation for Ethereum/Solidity proof systems
//!
//! This module provides an EVM backend that implements the BackendTrait,
//! wrapping the existing EVM workflow functions to provide a unified interface.

use color_eyre::Result;

use crate::{
    backend::{Backend, BackendConfig},
    config::Config,
};

use super::workflow;

/// EVM backend implementation for Ethereum-based proof systems
#[derive(Debug)]
pub struct EvmBackend;

impl EvmBackend {
    /// Create a new EVM backend instance
    pub fn new() -> Self {
        Self
    }
}

impl Backend for EvmBackend {
    /// Generate Solidity verifier contract and setup Foundry project structure
    fn generate(&mut self, cfg: &Config) -> Result<()> {
        workflow::run_gen(cfg)
    }

    /// Generate proof using EVM/Keccak proof system
    fn prove(&mut self, cfg: &Config) -> Result<()> {
        workflow::run_prove(cfg)
    }

    /// Verify a generated EVM proof
    fn verify(&mut self, cfg: &Config) -> Result<()> {
        workflow::run_verify(cfg)
    }

    /// Generate calldata for EVM proof verification
    fn calldata(&mut self, cfg: &Config) -> Result<()> {
        workflow::run_calldata(cfg)
    }

    /// Deploy Solidity verifier contract to EVM network
    fn deploy(&mut self, cfg: &Config, network: Option<&str>) -> Result<()> {
        // Use provided network or default to "sepolia"
        let network_str = network.unwrap_or("sepolia");
        workflow::run_deploy(cfg, network_str)
    }

    /// Verify proof on-chain using deployed EVM verifier
    fn verify_onchain(&mut self, cfg: &Config, _address: Option<&str>) -> Result<()> {
        // EVM verify_onchain doesn't take an address parameter in the current implementation
        workflow::run_verify_onchain(cfg)
    }

    /// Configure backend with backend-specific settings
    fn configure(&mut self, _config: BackendConfig) -> Result<()> {
        // EVM backend currently doesn't need any configuration
        // This could be extended in the future for EVM-specific settings
        Ok(())
    }
}

impl Default for EvmBackend {
    fn default() -> Self {
        Self::new()
    }
}
