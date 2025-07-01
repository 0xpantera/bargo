//! Directory management for EVM workflow
//!
//! This module provides focused functions for managing directories and file organization
//! specific to the EVM workflow, including artifact organization and project movement.

use color_eyre::Result;
use color_eyre::eyre::WrapErr;
use std::path::{Path, PathBuf};

use crate::util::{self, Flavour};

/// Ensure the EVM target directory exists
///
/// This function creates the target/evm directory if it doesn't exist,
/// which is where EVM-specific artifacts (proofs, VKs, public inputs) are stored.
///
/// # Returns
/// * `Result<()>` - Success or error from directory creation
pub fn ensure_evm_target_dir() -> Result<()> {
    util::ensure_target_dir(Flavour::Evm)
}

/// Ensure the EVM contracts directory exists
///
/// This function creates the contracts/evm directory if it doesn't exist,
/// which is where Foundry projects and Solidity verifier contracts are stored.
///
/// # Returns
/// * `Result<()>` - Success or error from directory creation
pub fn ensure_evm_contracts_dir() -> Result<()> {
    let evm_dir = Path::new("./contracts/evm");
    if !evm_dir.exists() {
        std::fs::create_dir_all(evm_dir).wrap_err_with(|| {
            format!("creating EVM contracts directory at {}", evm_dir.display())
        })?;
    }
    Ok(())
}

/// Ensure the EVM contracts source directory exists
///
/// This function creates the contracts/evm/src directory for Solidity contracts.
///
/// # Returns
/// * `Result<()>` - Success or error from directory creation
pub fn ensure_evm_contracts_src_dir() -> Result<()> {
    let src_dir = Path::new("./contracts/evm/src");
    if !src_dir.exists() {
        std::fs::create_dir_all(src_dir).wrap_err_with(|| {
            format!(
                "creating EVM contracts source directory at {}",
                src_dir.display()
            )
        })?;
    }
    Ok(())
}

/// Validate EVM workflow directory structure
///
/// This function checks that all necessary directories exist for the EVM workflow
/// and creates them if they don't exist.
///
/// # Returns
/// * `Result<()>` - Success or error from validation/creation
pub fn validate_evm_directory_structure() -> Result<()> {
    ensure_evm_target_dir()?;
    util::ensure_contracts_dir()?;
    ensure_evm_contracts_dir()?;
    ensure_evm_contracts_src_dir()?;
    Ok(())
}

/// Get the path to the EVM contracts directory
///
/// # Returns
/// * `PathBuf` - Path to the EVM contracts directory
pub fn get_evm_contracts_dir() -> PathBuf {
    PathBuf::from("./contracts/evm")
}

/// Get the path to the EVM contracts source directory
///
/// # Returns
/// * `PathBuf` - Path to the EVM contracts source directory
pub fn get_evm_contracts_src_dir() -> PathBuf {
    PathBuf::from("./contracts/evm/src")
}

/// Check if the standard Verifier.sol contract exists
///
/// This function checks if the generated Verifier.sol contract exists in the
/// expected location.
///
/// # Returns
/// * `bool` - True if Verifier.sol exists
pub fn verifier_contract_exists() -> bool {
    let verifier_path = get_evm_contracts_src_dir().join("Verifier.sol");
    verifier_path.exists()
}

/// Get the path to the Verifier.sol contract
///
/// # Returns
/// * `PathBuf` - Path to the Verifier.sol contract
pub fn get_verifier_contract_path() -> PathBuf {
    get_evm_contracts_src_dir().join("Verifier.sol")
}
