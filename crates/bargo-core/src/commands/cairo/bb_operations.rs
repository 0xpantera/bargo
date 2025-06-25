//! BB operations for Cairo/Starknet backend
//!
//! This module provides focused functions for interacting with the BB backend
//! specifically for Starknet oracle hash operations.

use color_eyre::Result;

use crate::{
    commands::common,
    config::Config,
    util::{self, Flavour},
};

/// Generate a Starknet-compatible proof using BB with ultra_honk scheme
///
/// This function generates a proof with the following BB flags:
/// - `--scheme ultra_honk`
/// - `--oracle_hash starknet`
/// - `--zk`
///
/// # Arguments
/// * `cfg` - Configuration containing runner and flags
/// * `pkg` - Package name for locating bytecode and witness files
///
/// # Returns
/// * `Result<()>` - Success or error from BB execution
pub fn generate_starknet_proof(cfg: &Config, pkg: &str) -> Result<()> {
    let bytecode = util::get_bytecode_path(pkg, Flavour::Bb);
    let witness = util::get_witness_path(pkg, Flavour::Bb);

    common::run_bb_command(
        cfg,
        &[
            "prove",
            "--scheme",
            "ultra_honk",
            "--oracle_hash",
            "starknet",
            "--zk",
            "-b",
            &bytecode.to_string_lossy(),
            "-w",
            &witness.to_string_lossy(),
            "-o",
            "./target/starknet/",
        ],
    )
}

/// Generate a Starknet-compatible verification key using BB
///
/// This function generates a VK with the following BB flags:
/// - `--oracle_hash starknet`
///
/// # Arguments
/// * `cfg` - Configuration containing runner and flags
/// * `pkg` - Package name for locating bytecode file
///
/// # Returns
/// * `Result<()>` - Success or error from BB execution
pub fn generate_starknet_vk(cfg: &Config, pkg: &str) -> Result<()> {
    let bytecode = util::get_bytecode_path(pkg, Flavour::Bb);

    common::run_bb_command(
        cfg,
        &[
            "write_vk",
            "--oracle_hash",
            "starknet",
            "-b",
            &bytecode.to_string_lossy(),
            "-o",
            "./target/starknet/",
        ],
    )
}

/// Verify a Starknet proof using BB
///
/// This function verifies a proof using the verification key and public inputs
/// stored in the target/starknet/ directory.
///
/// # Arguments
/// * `cfg` - Configuration containing runner and flags
/// * `_pkg` - Package name (currently unused but kept for consistency)
///
/// # Returns
/// * `Result<()>` - Success or error from BB execution
pub fn verify_starknet_proof(cfg: &Config, _pkg: &str) -> Result<()> {
    let proof_path = util::get_proof_path(Flavour::Starknet);
    let vk_path = util::get_vk_path(Flavour::Starknet);
    let public_inputs_path = util::get_public_inputs_path(Flavour::Starknet);

    common::run_bb_command(
        cfg,
        &[
            "verify",
            "--scheme",
            "ultra_honk",
            "--zk",
            "-p",
            &proof_path.to_string_lossy(),
            "-k",
            &vk_path.to_string_lossy(),
            "-i",
            &public_inputs_path.to_string_lossy(),
            "--oracle_hash",
            "starknet",
        ],
    )
}

/// Generate both Starknet proof and verification key in a single operation
///
/// This is a convenience function that calls both generate_starknet_proof
/// and generate_starknet_vk sequentially.
///
/// # Arguments
/// * `cfg` - Configuration containing runner and flags
/// * `pkg` - Package name for locating bytecode and witness files
///
/// # Returns
/// * `Result<()>` - Success or error from either operation
pub fn generate_starknet_proof_and_vk(cfg: &Config, pkg: &str) -> Result<()> {
    generate_starknet_proof(cfg, pkg)?;
    generate_starknet_vk(cfg, pkg)?;
    Ok(())
}
