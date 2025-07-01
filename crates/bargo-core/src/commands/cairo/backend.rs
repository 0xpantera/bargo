//! Cairo backend implementation for Starknet proof systems
//!
//! This module provides a Cairo backend that implements the BackendTrait,
//! wrapping the existing Cairo workflow functions to provide a unified interface.

use color_eyre::Result;
use color_eyre::eyre::WrapErr;

use crate::{
    backend::{Backend, BackendConfig},
    config::{CairoDeployConfig, Config},
};

use super::workflow;

/// Cairo backend implementation for Starknet-based proof systems
#[derive(Debug)]
pub struct CairoBackend {
    deploy_config: Option<CairoDeployConfig>,
}

impl CairoBackend {
    /// Create a new Cairo backend instance
    pub fn new() -> Self {
        Self {
            deploy_config: None,
        }
    }
}

impl Backend for CairoBackend {
    /// Generate Cairo verifier contract and setup project structure
    fn generate(&mut self, cfg: &Config) -> Result<()> {
        workflow::run_gen(cfg)
    }

    /// Generate proof using Cairo/Starknet proof system
    fn prove(&mut self, cfg: &Config) -> Result<()> {
        workflow::run_prove(cfg)
    }

    /// Verify a generated Cairo proof
    fn verify(&mut self, cfg: &Config) -> Result<()> {
        workflow::run_verify(cfg)
    }

    /// Generate calldata for Cairo proof verification
    fn calldata(&mut self, cfg: &Config) -> Result<()> {
        workflow::run_calldata(cfg)
    }

    /// Deploy Cairo verifier contract to Starknet network
    ///
    /// Cairo deployment is a two-step process:
    /// 1. Declare the contract on the network to get a class_hash (if auto-declare is enabled)
    /// 2. Deploy an instance of the contract using the class_hash
    fn deploy(&mut self, cfg: &Config, network: Option<&str>) -> Result<()> {
        // Use provided network or default to "sepolia"
        let network_str = network.unwrap_or("sepolia");

        // Get deploy configuration or use defaults
        let default_config = CairoDeployConfig::new(
            None, true,  // Default to auto-declare enabled
            false, // Default to not forcing no-declare
        );
        let deploy_cfg = self.deploy_config.as_ref().unwrap_or(&default_config);

        // In dry-run mode, skip all validations and just call the workflow functions
        if cfg.dry_run {
            if deploy_cfg.should_auto_declare() {
                workflow::internal_declare(cfg, network_str)?;
            }
            return workflow::run_deploy(cfg, deploy_cfg.class_hash.as_deref());
        }

        // Check if we should auto-declare
        if deploy_cfg.should_auto_declare() {
            // Check if contract is already declared by trying to read saved class hash
            let class_hash_file = std::path::PathBuf::from("target/starknet/.bargo_class_hash");
            let class_hash_exists = class_hash_file.exists()
                && std::fs::read_to_string(&class_hash_file)
                    .wrap_err_with(|| {
                        format!(
                            "reading saved class hash from {}",
                            class_hash_file.display()
                        )
                    })
                    .map(|s| !s.trim().is_empty())
                    .unwrap_or(false);

            if !class_hash_exists {
                // Step 1: Declare the contract to get class_hash
                workflow::internal_declare(cfg, network_str)?;
            }
        } else if deploy_cfg.class_hash.is_none() {
            // No auto-declare and no class hash provided - check if saved class hash exists
            let class_hash_file = std::path::PathBuf::from("target/starknet/.bargo_class_hash");
            if !class_hash_file.exists() {
                return Err(color_eyre::eyre::eyre!(
                    "No class hash provided and auto-declare is disabled. Either provide --class-hash or enable auto-declare"
                ));
            }
        }

        // Step 2: Deploy the contract using the class_hash
        workflow::run_deploy(cfg, deploy_cfg.class_hash.as_deref())
    }

    /// Verify proof on-chain using deployed Cairo verifier on Starknet
    fn verify_onchain(&mut self, cfg: &Config, address: Option<&str>) -> Result<()> {
        workflow::run_verify_onchain(cfg, address)
    }

    /// Configure backend with backend-specific settings
    fn configure(&mut self, config: BackendConfig) -> Result<()> {
        match config {
            BackendConfig::CairoDeploy(deploy_config) => {
                self.deploy_config = Some(deploy_config);
                Ok(())
            }
        }
    }
}

impl Default for CairoBackend {
    fn default() -> Self {
        Self::new()
    }
}
