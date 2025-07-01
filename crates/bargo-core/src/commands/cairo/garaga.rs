//! Garaga operations for Cairo contract generation and calldata
//!
//! This module provides focused functions for interacting with the Garaga tool
//! for Cairo verifier contract generation and proof calldata creation.

use color_eyre::Result;
use color_eyre::eyre::WrapErr;
use std::path::{Path, PathBuf};

use crate::{
    commands::common,
    config::Config,
    util::{self, Flavour, move_generated_project},
};

/// Generate calldata JSON for Starknet proof verification
///
/// This function uses Garaga to generate properly formatted calldata
/// that can be used for on-chain proof verification on Starknet.
///
/// # Arguments
/// * `cfg` - Configuration containing runner and flags
/// * `proof_path` - Path to the proof file
/// * `vk_path` - Path to the verification key file
/// * `public_inputs_path` - Path to the public inputs file
/// * `output_path` - Optional output path for calldata (defaults to target/starknet/calldata.json)
///
/// # Returns
/// * `Result<PathBuf>` - Path to generated calldata file or error
pub fn generate_calldata(
    cfg: &Config,
    proof_path: &Path,
    vk_path: &Path,
    public_inputs_path: &Path,
    output_path: Option<&Path>,
) -> Result<PathBuf> {
    let proof_str = proof_path.to_string_lossy();
    let vk_str = vk_path.to_string_lossy();
    let public_inputs_str = public_inputs_path.to_string_lossy();

    let garaga_args = vec![
        "calldata",
        "--system",
        "ultra_starknet_zk_honk",
        "--proof",
        &proof_str,
        "--vk",
        &vk_str,
        "--public-inputs",
        &public_inputs_str,
    ];

    // Use runner to capture stdout for calldata generation
    let stdout = common::run_tool_capture(cfg, "garaga", &garaga_args)?;

    // Determine output path
    let calldata_path = output_path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("./target/starknet/calldata.json"));

    // Save calldata to file
    std::fs::write(&calldata_path, stdout.trim())
        .wrap_err_with(|| format!("writing calldata to {}", calldata_path.display()))?;

    Ok(calldata_path)
}

/// Generate calldata using default Starknet artifact paths
///
/// Convenience function that uses the standard Starknet artifact locations
/// to generate calldata JSON.
///
/// # Arguments
/// * `cfg` - Configuration containing runner and flags
///
/// # Returns
/// * `Result<PathBuf>` - Path to generated calldata file or error
pub fn generate_calldata_from_starknet_artifacts(cfg: &Config) -> Result<PathBuf> {
    let proof_path = util::get_proof_path(Flavour::Starknet);
    let vk_path = util::get_vk_path(Flavour::Starknet);
    let public_inputs_path = util::get_public_inputs_path(Flavour::Starknet);

    generate_calldata(cfg, &proof_path, &vk_path, &public_inputs_path, None)
}

/// Generate Cairo verifier contract using Garaga
///
/// This function generates a Cairo smart contract that can verify proofs
/// on Starknet using the provided verification key.
///
/// # Arguments
/// * `cfg` - Configuration containing runner and flags
/// * `vk_path` - Path to the verification key file
/// * `output_dir` - Optional output directory (defaults to ./contracts/cairo/)
///
/// # Returns
/// * `Result<()>` - Success or error from Garaga execution
pub fn generate_cairo_contract(
    cfg: &Config,
    vk_path: &Path,
    output_dir: Option<&str>,
) -> Result<()> {
    let output = output_dir.unwrap_or("./contracts/cairo/");

    let vk_str = vk_path.to_string_lossy();

    let garaga_args = vec![
        "gen",
        "--system",
        "ultra_starknet_zk_honk",
        "--vk",
        &vk_str,
        "--project-name",
        "cairo_verifier",
    ];

    common::run_tool(cfg, "garaga", &garaga_args)?;

    // Move the generated project to the correct location (skip in dry-run mode)
    if cfg.dry_run {
        Ok(())
    } else {
        move_generated_project("cairo_verifier", output)
    }
}

/// Generate Cairo verifier contract using default Starknet VK path
///
/// Convenience function that uses the standard Starknet VK location
/// to generate a Cairo verifier contract.
///
/// # Arguments
/// * `cfg` - Configuration containing runner and flags
///
/// # Returns
/// * `Result<()>` - Success or error from Garaga execution
pub fn generate_cairo_contract_from_starknet_vk(cfg: &Config) -> Result<()> {
    let vk_path = util::get_vk_path(Flavour::Starknet);
    generate_cairo_contract(cfg, &vk_path, None)
}

/// Validate that required Starknet artifacts exist for Garaga operations
///
/// This function checks that all necessary files exist before attempting
/// to generate calldata or contracts.
///
/// # Returns
/// * `Result<()>` - Success if all files exist, error otherwise
pub fn validate_starknet_artifacts() -> Result<()> {
    let required_files = vec![
        util::get_proof_path(Flavour::Starknet),
        util::get_vk_path(Flavour::Starknet),
        util::get_public_inputs_path(Flavour::Starknet),
    ];

    util::validate_files_exist(&required_files)
}
