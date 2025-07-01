//! Cairo backend specific errors
//!
//! This module defines error types specific to Cairo operations including
//! deploy failures, class hash management, and Starknet interactions.
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
//! use bargo_core::commands::cairo::{CairoError, Result};
//!
//! fn deploy_contract() -> Result<String> {
//!     // This will return a structured error with context
//!     Err(CairoError::deploy_failed("Missing class hash"))
//! }
//!
//! // Error chaining with context
//! let result = deploy_contract()
//!     .wrap_err("Failed to deploy Cairo contract");
//! ```
//!
//! # Error Categories
//!
//! - **Deployment**: Contract deployment and declaration failures
//! - **File Operations**: I/O errors when reading/writing artifacts
//! - **Tool Integration**: Errors from external tools (Garaga, Scarb, etc.)
//! - **Blockchain**: Starknet network and contract interaction errors

use thiserror::Error;

/// Cairo backend specific errors
///
/// This enum represents all possible errors that can occur during Cairo
/// backend operations. Each variant includes a descriptive message and
/// provides context about what operation failed.
///
/// # Design
///
/// The error variants are organized by the type of operation that failed:
/// - Contract lifecycle (deploy, declare, verify)
/// - Tool integration (Garaga, Scarb, bb)
/// - File operations (reading artifacts, writing results)
/// - Configuration and setup
///
/// # Error Messages
///
/// All error messages are designed to be user-friendly and include
/// actionable information when possible.
#[derive(Error, Debug)]
pub enum CairoError {
    /// Error during contract deployment
    ///
    /// This error occurs when contract deployment to Starknet fails,
    /// including both declaration and deployment phases.
    #[error("Deploy failed: {message}")]
    DeployFailed { message: String },

    /// Class hash related errors
    ///
    /// This error occurs when there are issues with class hash management,
    /// such as missing class hash files or invalid hash values.
    #[error("Class hash error: {message}")]
    ClassHashError { message: String },

    /// Contract address related errors
    ///
    /// This error occurs when there are issues with contract addresses,
    /// such as missing address files or invalid address formats.
    #[error("Contract address error: {message}")]
    ContractAddressError { message: String },

    /// Starknet configuration or interaction errors
    ///
    /// This error occurs when there are issues with Starknet network
    /// configuration or blockchain interactions.
    #[error("Starknet error: {message}")]
    StarknetError { message: String },

    /// Garaga-specific errors
    ///
    /// This error occurs when Garaga tool operations fail, such as
    /// contract generation or calldata creation.
    #[error("Garaga operation failed: {message}")]
    GaragaError { message: String },

    /// Scarb-specific errors
    ///
    /// This error occurs when Scarb tool operations fail, such as
    /// project building or dependency management.
    #[error("Scarb operation failed: {message}")]
    ScarbError { message: String },

    /// Cairo file I/O errors
    ///
    /// This error occurs when file operations fail, such as reading
    /// artifacts, writing results, or accessing configuration files.
    #[error("Cairo file operation failed: {message}")]
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

    /// Configuration parsing errors
    ///
    /// This error occurs when configuration files cannot be parsed
    /// or contain invalid values.
    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    /// Generic Cairo backend error
    ///
    /// This error is used for Cairo backend errors that don't fit
    /// into more specific categories.
    #[error("Cairo backend error: {message}")]
    Other { message: String },
}

impl CairoError {
    /// Create a deploy failed error
    pub fn deploy_failed<S: Into<String>>(message: S) -> Self {
        Self::DeployFailed {
            message: message.into(),
        }
    }

    /// Create a class hash error
    pub fn class_hash_error<S: Into<String>>(message: S) -> Self {
        Self::ClassHashError {
            message: message.into(),
        }
    }

    /// Create a contract address error
    pub fn contract_address_error<S: Into<String>>(message: S) -> Self {
        Self::ContractAddressError {
            message: message.into(),
        }
    }

    /// Create a Starknet error
    pub fn starknet_error<S: Into<String>>(message: S) -> Self {
        Self::StarknetError {
            message: message.into(),
        }
    }

    /// Create a Garaga error
    pub fn garaga_error<S: Into<String>>(message: S) -> Self {
        Self::GaragaError {
            message: message.into(),
        }
    }

    /// Create a Scarb error
    pub fn scarb_error<S: Into<String>>(message: S) -> Self {
        Self::ScarbError {
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

/// Result type alias for Cairo operations
///
/// This is a convenience alias for `std::result::Result<T, CairoError>`.
/// It should be used for all Cairo backend functions that can fail.
///
/// # Examples
///
/// ```ignore
/// use bargo_core::commands::cairo::Result;
///
/// fn cairo_operation() -> Result<String> {
///     Ok("success".to_string())
/// }
/// ```
pub type Result<T> = std::result::Result<T, CairoError>;

/// Convert from eyre::Error to CairoError
///
/// This implementation allows automatic conversion from generic eyre errors
/// to Cairo-specific errors, preserving the error chain and context.
impl From<color_eyre::eyre::Error> for CairoError {
    fn from(err: color_eyre::eyre::Error) -> Self {
        Self::Other {
            message: err.to_string(),
        }
    }
}

/// Convert from std::io::Error to CairoError
///
/// This implementation allows automatic conversion from I/O errors
/// to Cairo file errors, making error handling more ergonomic.
impl From<std::io::Error> for CairoError {
    fn from(err: std::io::Error) -> Self {
        Self::FileError {
            message: err.to_string(),
        }
    }
}
