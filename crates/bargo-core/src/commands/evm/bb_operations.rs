//! BB operations for EVM backend
//!
//! This module provides focused functions for interacting with the BB backend
//! specifically for EVM keccak oracle hash operations.

use color_eyre::Result;

use crate::{
    backends,
    util::{self, Flavour},
};

/// Generate an EVM-compatible proof using BB with keccak oracle hash
///
/// This function generates a proof with the following BB flags:
/// - `--oracle_hash keccak`
/// - `--output_format bytes_and_fields`
///
/// # Arguments
/// * `pkg` - Package name for locating bytecode and witness files
///
/// # Returns
/// * `Result<()>` - Success or error from BB execution
pub fn generate_evm_proof(pkg: &str) -> Result<()> {
    let bytecode = util::get_bytecode_path(pkg, Flavour::Bb);
    let witness = util::get_witness_path(pkg, Flavour::Bb);

    backends::bb::run(&[
        "prove",
        "-b",
        &bytecode.to_string_lossy(),
        "-w",
        &witness.to_string_lossy(),
        "-o",
        "./target/evm/",
        "--oracle_hash",
        "keccak",
        "--output_format",
        "bytes_and_fields",
    ])
}

/// Generate an EVM-compatible verification key using BB
///
/// This function generates a VK with the following BB flags:
/// - `--oracle_hash keccak`
///
/// # Arguments
/// * `pkg` - Package name for locating bytecode file
///
/// # Returns
/// * `Result<()>` - Success or error from BB execution
pub fn generate_evm_vk(pkg: &str) -> Result<()> {
    let bytecode = util::get_bytecode_path(pkg, Flavour::Bb);

    backends::bb::run(&[
        "write_vk",
        "--oracle_hash",
        "keccak",
        "-b",
        &bytecode.to_string_lossy(),
        "-o",
        "./target/evm/",
    ])
}

/// Verify an EVM proof using BB
///
/// This function verifies a proof using the verification key and public inputs
/// stored in the target/evm/ directory.
///
/// # Arguments
/// * `pkg` - Package name (currently unused but kept for consistency)
///
/// # Returns
/// * `Result<()>` - Success or error from BB execution
pub fn verify_evm_proof(_pkg: &str) -> Result<()> {
    let proof_path = util::get_proof_path(Flavour::Evm);
    let vk_path = util::get_vk_path(Flavour::Evm);
    let public_inputs_path = util::get_public_inputs_path(Flavour::Evm);

    backends::bb::run(&[
        "verify",
        "-p",
        &proof_path.to_string_lossy(),
        "-k",
        &vk_path.to_string_lossy(),
        "-i",
        &public_inputs_path.to_string_lossy(),
        "--oracle_hash",
        "keccak",
    ])
}

/// Generate both EVM proof and verification key in a single operation
///
/// This is a convenience function that calls both generate_evm_proof
/// and generate_evm_vk sequentially.
///
/// # Arguments
/// * `pkg` - Package name for locating bytecode and witness files
///
/// # Returns
/// * `Result<()>` - Success or error from either operation
pub fn generate_evm_proof_and_vk(pkg: &str) -> Result<()> {
    generate_evm_proof(pkg)?;
    generate_evm_vk(pkg)?;
    Ok(())
}

/// Write Solidity verifier contract using BB
///
/// This function generates a Solidity smart contract that can verify proofs
/// on EVM networks using the provided verification key.
///
/// # Arguments
/// * `vk_path` - Path to the verification key file
/// * `output_path` - Path where the Solidity contract should be written
///
/// # Returns
/// * `Result<()>` - Success or error from BB execution
pub fn write_solidity_verifier(vk_path: &str, output_path: &str) -> Result<()> {
    backends::bb::run(&["write_solidity_verifier", "-k", vk_path, "-o", output_path])
}

/// Write Solidity verifier contract using default EVM VK path
///
/// Convenience function that uses the standard EVM VK location
/// to generate a Solidity verifier contract.
///
/// # Arguments
/// * `output_path` - Path where the Solidity contract should be written
///
/// # Returns
/// * `Result<()>` - Success or error from BB execution
pub fn write_solidity_verifier_from_evm_vk(output_path: &str) -> Result<()> {
    let vk_path = util::get_vk_path(Flavour::Evm);
    write_solidity_verifier(&vk_path.to_string_lossy(), output_path)
}

/// Validate that required EVM artifacts exist for BB operations
///
/// This function checks that all necessary files exist before attempting
/// to verify proofs or generate contracts.
///
/// # Returns
/// * `Result<()>` - Success if all files exist, error otherwise
pub fn validate_evm_artifacts() -> Result<()> {
    let required_files = vec![
        util::get_proof_path(Flavour::Evm),
        util::get_vk_path(Flavour::Evm),
        util::get_public_inputs_path(Flavour::Evm),
    ];

    util::validate_files_exist(&required_files)
}
