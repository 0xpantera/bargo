//! EVM backend specific errors
//!
//! This module defines error types specific to EVM operations including
//! Foundry interactions, contract deployment, and verification.
//!
//! # Error Context
//!
//! All errors in this module are designed to provide rich context information
//! to help users understand what went wrong and how to fix it. The errors
//! implement the `thiserror::Error` trait for automatic error chain support.
//!
//! # Examples
//!
//! ```ignore
//! use bargo_core::commands::evm::{EvmError, Result};
//!
//! fn deploy_contract() -> Result<String> {
//!     // This will return a structured error with context
//!     Err(EvmError::deploy_failed("Missing verifier contract"))
//! }
//!
//! // Error chaining with context
//! let result = deploy_contract()
//!     .wrap_err("Failed to deploy EVM contract");
//! ```
//!
//! # Error Categories
//!
//! - **Deployment**: Contract deployment and verification failures
//! - **File Operations**: I/O errors when reading/writing artifacts
//! - **Tool Integration**: Errors from external tools (Foundry, bb, etc.)
//! - **Network**: Ethereum network and blockchain interaction errors

use thiserror::Error;

/// EVM backend specific errors
///
/// This enum represents all possible errors that can occur during EVM
/// backend operations. Each variant includes a descriptive message and
/// provides context about what operation failed.
///
/// # Design
///
/// The error variants are organized by the type of operation that failed:
/// - Contract lifecycle (deploy, compile, verify)
/// - Tool integration (Foundry, bb)
/// - File operations (reading artifacts, writing results)
/// - Network and blockchain interactions
///
/// # Error Messages
///
/// All error messages are designed to be user-friendly and include
/// actionable information when possible.
#[derive(Error, Debug)]
pub enum EvmError {
    /// Error during contract deployment
    ///
    /// This error occurs when contract deployment to Ethereum fails,
    /// including compilation, deployment, and initialization phases.
    #[error("Deploy failed: {message}")]
    DeployFailed { message: String },

    /// Contract address related errors
    ///
    /// This error occurs when there are issues with contract addresses,
    /// such as missing address files or invalid address formats.
    #[error("Contract address error: {message}")]
    ContractAddressError { message: String },

    /// Foundry-specific errors (forge, anvil, cast)
    ///
    /// This error occurs when Foundry tool operations fail, such as
    /// contract compilation, deployment, or blockchain interactions.
    #[error("Foundry operation failed: {message}")]
    FoundryError { message: String },

    /// Network configuration or interaction errors
    ///
    /// This error occurs when there are issues with Ethereum network
    /// configuration or blockchain interactions.
    #[error("Network error: {message}")]
    NetworkError { message: String },

    /// EVM file I/O errors
    ///
    /// This error occurs when file operations fail, such as reading
    /// artifacts, writing results, or accessing configuration files.
    #[error("EVM file operation failed: {message}")]
    FileError { message: String },

    /// Proof generation errors
    ///
    /// This error occurs when proof generation fails, typically due to
    /// issues with the bb tool or invalid circuit inputs.
    #[error("Proof generation failed: {message}")]
    ProofError { message: String },

    /// Verification errors
    ///
    /// This error occurs when proof verification fails, either locally
    /// or during on-chain verification.
    #[error("Verification failed: {message}")]
    VerificationError { message: String },

    /// Calldata generation errors
    ///
    /// This error occurs when calldata generation fails, typically due to
    /// missing proof artifacts or formatting issues.
    #[error("Calldata generation failed: {message}")]
    CalldataError { message: String },

    /// Contract compilation errors
    ///
    /// This error occurs when Solidity contract compilation fails,
    /// typically due to syntax errors or dependency issues.
    #[error("Contract compilation failed: {message}")]
    CompilationError { message: String },

    /// Configuration parsing errors
    ///
    /// This error occurs when configuration files cannot be parsed
    /// or contain invalid values.
    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    /// Generic EVM backend error
    ///
    /// This error is used for EVM backend errors that don't fit
    /// into more specific categories.
    #[error("EVM backend error: {message}")]
    Other { message: String },
}

impl EvmError {
    /// Create a deploy failed error
    pub fn deploy_failed<S: Into<String>>(message: S) -> Self {
        Self::DeployFailed {
            message: message.into(),
        }
    }

    /// Create a contract address error
    pub fn contract_address_error<S: Into<String>>(message: S) -> Self {
        Self::ContractAddressError {
            message: message.into(),
        }
    }

    /// Create a Foundry error
    pub fn foundry_error<S: Into<String>>(message: S) -> Self {
        Self::FoundryError {
            message: message.into(),
        }
    }

    /// Create a network error
    pub fn network_error<S: Into<String>>(message: S) -> Self {
        Self::NetworkError {
            message: message.into(),
        }
    }

    /// Create a file error
    pub fn file_error<S: Into<String>>(message: S) -> Self {
        Self::FileError {
            message: message.into(),
        }
    }

    /// Create a proof error
    pub fn proof_error<S: Into<String>>(message: S) -> Self {
        Self::ProofError {
            message: message.into(),
        }
    }

    /// Create a verification error
    pub fn verification_error<S: Into<String>>(message: S) -> Self {
        Self::VerificationError {
            message: message.into(),
        }
    }

    /// Create a calldata error
    pub fn calldata_error<S: Into<String>>(message: S) -> Self {
        Self::CalldataError {
            message: message.into(),
        }
    }

    /// Create a compilation error
    pub fn compilation_error<S: Into<String>>(message: S) -> Self {
        Self::CompilationError {
            message: message.into(),
        }
    }

    /// Create a configuration error
    pub fn config_error<S: Into<String>>(message: S) -> Self {
        Self::ConfigError {
            message: message.into(),
        }
    }

    /// Create a generic error
    pub fn other<S: Into<String>>(message: S) -> Self {
        Self::Other {
            message: message.into(),
        }
    }
}

/// Result type alias for EVM operations
///
/// This is a convenience alias for `std::result::Result<T, EvmError>`.
/// It should be used for all EVM backend functions that can fail.
///
/// # Examples
///
/// ```ignore
/// use bargo_core::commands::evm::Result;
///
/// fn evm_operation() -> Result<String> {
///     Ok("success".to_string())
/// }
/// ```
pub type Result<T> = std::result::Result<T, EvmError>;

/// Convert from eyre::Error to EvmError
///
/// This implementation allows automatic conversion from generic eyre errors
/// to EVM-specific errors, preserving the error chain and context.
impl From<color_eyre::eyre::Error> for EvmError {
    fn from(err: color_eyre::eyre::Error) -> Self {
        Self::Other {
            message: err.to_string(),
        }
    }
}

/// Convert from std::io::Error to EvmError
///
/// This implementation allows automatic conversion from I/O errors
/// to EVM file errors, making error handling more ergonomic.
impl From<std::io::Error> for EvmError {
    fn from(err: std::io::Error) -> Self {
        Self::FileError {
            message: err.to_string(),
        }
    }
}
