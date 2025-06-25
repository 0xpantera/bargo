//! Backend trait and factory for polymorphic proof system backends
//!
//! This module provides a unified interface for different proof system backends
//! (Cairo/Starknet and EVM/Ethereum), allowing them to be used interchangeably
//! through trait objects or concrete types.

use color_eyre::Result;

use crate::config::{CairoDeployConfig, Config};

/// Trait for polymorphic backend implementations (Cairo, EVM, etc.)
///
/// This trait provides a unified interface for different proof system backends,
/// allowing them to be used interchangeably through dynamic dispatch.
pub trait Backend {
    /// Generate verifier contract and setup project structure
    fn generate(&mut self, cfg: &Config) -> Result<()>;

    /// Generate proof using the backend's proof system
    fn prove(&mut self, cfg: &Config) -> Result<()>;

    /// Verify a generated proof
    fn verify(&mut self, cfg: &Config) -> Result<()>;

    /// Generate calldata for proof verification
    fn calldata(&mut self, cfg: &Config) -> Result<()>;

    /// Deploy verifier contract to specified network
    ///
    /// Note: Implementation varies by backend:
    /// - Cairo: Two-step process (declare contract to get class_hash, then deploy instance)
    /// - EVM: Single-step process (deploy contract directly to network)
    fn deploy(&mut self, cfg: &Config, network: Option<&str>) -> Result<()>;

    /// Verify proof on-chain using deployed verifier
    fn verify_onchain(&mut self, cfg: &Config, address: Option<&str>) -> Result<()>;

    /// Configure backend with backend-specific settings
    fn configure(&mut self, config: BackendConfig) -> Result<()>;
}

/// Backend configuration for backend-specific settings
#[derive(Debug, Clone)]
pub enum BackendConfig {
    /// Cairo/Starknet backend configuration
    CairoDeploy(CairoDeployConfig),
}

/// Backend type identifier for factory function
#[derive(Debug, Clone, Copy)]
pub enum BackendKind {
    /// Cairo/Starknet backend
    Cairo,
    /// EVM/Ethereum backend
    Evm,
}

/// Factory function to create appropriate backend implementation
///
/// This function creates concrete backend implementations based on the backend kind,
/// returning a boxed trait object that can be used polymorphically.
///
/// # Arguments
/// * `backend_kind` - The backend kind (Cairo or EVM)
///
/// # Returns
/// * `Box<dyn Backend>` - Boxed backend implementation
///
/// # Example
/// ```ignore
/// let backend = backend_for(BackendKind::Cairo);
/// backend.generate(&config)?;
/// ```
pub fn backend_for(backend_kind: BackendKind) -> Box<dyn Backend> {
    use crate::commands::{cairo, evm};

    match backend_kind {
        BackendKind::Cairo => Box::new(cairo::backend::CairoBackend::new()),
        BackendKind::Evm => Box::new(evm::backend::EvmBackend::new()),
    }
}
