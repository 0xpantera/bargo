//! Foundry operations for EVM contract management
//!
//! This module provides focused functions for interacting with Foundry tools
//! for EVM contract compilation, deployment, and verification.

use color_eyre::Result;

use crate::backends;

/// Initialize a new Foundry project
///
/// This function creates a new Foundry project structure with the necessary
/// configuration files and directories.
///
/// # Arguments
/// * `project_path` - Path where the Foundry project should be created
///
/// # Returns
/// * `Result<()>` - Success or error from Foundry initialization
pub fn init_foundry_project(project_path: &str) -> Result<()> {
    backends::foundry::run_forge(&["init", "--force", project_path])
}

/// Initialize Foundry project at the default EVM contracts location
///
/// Convenience function that initializes a Foundry project at the standard
/// location used by the Bargo workflow.
///
/// # Returns
/// * `Result<()>` - Success or error from initialization
pub fn init_default_foundry_project() -> Result<()> {
    init_foundry_project("contracts/evm")
}

/// Deploy a contract using Foundry
///
/// This function deploys a contract to an EVM network using forge create.
///
/// # Arguments
/// * `contract_path` - Path to the contract source file
/// * `contract_name` - Name of the contract to deploy
/// * `rpc_url` - RPC URL for the target network
/// * `private_key` - Private key for deployment (should be env var)
/// * `constructor_args` - Optional constructor arguments
///
/// # Returns
/// * `Result<String>` - Contract address or error
pub fn deploy_contract(
    contract_path: &str,
    _contract_name: &str,
    rpc_url: &str,
    private_key: &str,
    constructor_args: Option<&[&str]>,
) -> Result<String> {
    let mut args = vec![
        "create",
        contract_path,
        "--rpc-url",
        rpc_url,
        "--private-key",
        private_key,
    ];

    if let Some(constructor_args) = constructor_args {
        args.push("--constructor-args");
        args.extend(constructor_args);
    }

    let (stdout, _stderr) = backends::foundry::run_forge_with_output(&args)?;

    // Parse contract address from forge output
    // forge create outputs: "Deployed to: 0x..."
    for line in stdout.lines() {
        if line.contains("Deployed to:") {
            if let Some(address) = line.split_whitespace().last() {
                return Ok(address.to_string());
            }
        }
    }

    Err(color_eyre::eyre::eyre!(
        "Could not parse contract address from forge output"
    ))
}

/// Deploy the default Verifier contract
///
/// Convenience function that deploys the Verifier.sol contract from the
/// standard EVM contracts directory.
///
/// # Arguments
/// * `rpc_url` - RPC URL for the target network
/// * `private_key` - Private key for deployment
///
/// # Returns
/// * `Result<String>` - Contract address or error
pub fn deploy_verifier_contract(rpc_url: &str, private_key: &str) -> Result<String> {
    deploy_contract(
        "contracts/evm/src/Verifier.sol:Verifier",
        "Verifier",
        rpc_url,
        private_key,
        None,
    )
}

/// Validate that Foundry tools are available
///
/// This function checks that forge and cast are installed and accessible.
///
/// # Returns
/// * `Result<()>` - Success if tools are available, error otherwise
pub fn validate_foundry_installation() -> Result<()> {
    backends::foundry::ensure_available()
}
