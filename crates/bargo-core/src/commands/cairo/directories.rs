//! Directory management for Cairo workflow
//!
//! This module provides focused functions for managing directories and file organization
//! specific to the Cairo/Starknet workflow, including artifact organization and project movement.

use color_eyre::Result;
use std::path::{Path, PathBuf};

use crate::util::{self, Flavour};

/// Ensure the Starknet target directory exists
///
/// This function creates the target/starknet directory if it doesn't exist,
/// which is where Starknet-specific artifacts (proofs, VKs, public inputs) are stored.
///
/// # Returns
/// * `Result<()>` - Success or error from directory creation
pub fn ensure_starknet_target_dir() -> Result<()> {
    util::ensure_target_dir(Flavour::Starknet)
}

/// Ensure the Cairo contracts directory exists
///
/// This function creates the contracts/cairo directory if it doesn't exist,
/// which is where generated Cairo verifier contracts are stored.
///
/// # Returns
/// * `Result<()>` - Success or error from directory creation
pub fn ensure_cairo_contracts_dir() -> Result<()> {
    let cairo_dir = Path::new("./contracts/cairo");
    if !cairo_dir.exists() {
        std::fs::create_dir_all(cairo_dir)?;
    }
    Ok(())
}

/// Validate Cairo workflow directory structure
///
/// This function checks that all necessary directories exist for the Cairo workflow
/// and creates them if they don't exist.
///
/// # Returns
/// * `Result<()>` - Success or error from validation/creation
pub fn validate_cairo_directory_structure() -> Result<()> {
    ensure_starknet_target_dir()?;
    util::ensure_contracts_dir()?;
    ensure_cairo_contracts_dir()?;
    Ok(())
}

/// Get the path to the Cairo contracts directory
///
/// # Returns
/// * `PathBuf` - Path to the Cairo contracts directory
pub fn get_cairo_contracts_dir() -> PathBuf {
    PathBuf::from("./contracts/cairo")
}
