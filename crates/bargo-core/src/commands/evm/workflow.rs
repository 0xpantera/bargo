//! Workflow orchestration for EVM commands
//!
//! This module provides clean orchestration functions that coordinate between
//! the different EVM modules to implement complete workflows for each command.

use color_eyre::Result;
use tracing::info;

use crate::{
    config::Config,
    util::{
        self, Flavour, OperationSummary, Timer, create_smart_error, enhance_error_with_suggestions,
        format_operation_result, success,
    },
};

use super::{bb_operations, directories, foundry, load_env_vars};

/// Run the EVM gen workflow
///
/// This function orchestrates the complete EVM verifier generation workflow:
/// 1. Initialize Foundry project
/// 2. Generate EVM proof and VK with keccak oracle
/// 3. Generate Solidity verifier contract
/// 4. Set up project structure
///
/// # Arguments
/// * `cli` - CLI configuration
///
/// # Returns
/// * `Result<()>` - Success or error from workflow
pub fn run_gen(cfg: &Config) -> Result<()> {
    let pkg_name = util::get_package_name(cfg.pkg.as_ref())?;
    load_env_vars();

    if cfg.verbose {
        info!("Starting EVM verifier generation workflow");
    }

    // Validate required files exist
    let required_files = vec![
        util::get_bytecode_path(&pkg_name, Flavour::Bb),
        util::get_witness_path(&pkg_name, Flavour::Bb),
    ];

    if !cfg.dry_run {
        util::validate_files_exist(&required_files).map_err(enhance_error_with_suggestions)?;
        directories::validate_evm_directory_structure().map_err(enhance_error_with_suggestions)?;
    }

    if cfg.dry_run {
        print_dry_run_commands(&pkg_name)?;
        return Ok(());
    }

    let mut summary = OperationSummary::new();

    // Step 1: Initialize Foundry project
    if cfg.verbose {
        info!("Initializing Foundry project");
    }
    let foundry_timer = Timer::start();
    foundry::init_default_foundry_project().map_err(enhance_error_with_suggestions)?;

    if !cfg.quiet {
        let foundry_dir = directories::get_evm_contracts_dir();
        println!(
            "{}",
            success(&format_operation_result(
                "Foundry project initialized",
                &foundry_dir,
                &foundry_timer
            ))
        );
        summary.add_operation("Foundry project structure");
    }

    // Step 2: Generate EVM proof
    if cfg.verbose {
        info!("Generating EVM proof with keccak oracle");
    }
    let proof_timer = Timer::start();
    bb_operations::generate_evm_proof(&pkg_name).map_err(enhance_error_with_suggestions)?;

    if !cfg.quiet {
        let proof_path = util::get_proof_path(Flavour::Evm);
        println!(
            "{}",
            success(&format_operation_result(
                "EVM proof generated",
                &proof_path,
                &proof_timer
            ))
        );
        summary.add_operation(&format!(
            "EVM proof ({})",
            util::format_file_size(&proof_path)
        ));
    }

    // Step 3: Generate EVM VK
    if cfg.verbose {
        info!("Generating EVM verification key");
    }
    let vk_timer = Timer::start();
    bb_operations::generate_evm_vk(&pkg_name).map_err(enhance_error_with_suggestions)?;

    if !cfg.quiet {
        let vk_path = util::get_vk_path(Flavour::Evm);
        println!(
            "{}",
            success(&format_operation_result(
                "EVM VK generated",
                &vk_path,
                &vk_timer
            ))
        );
        summary.add_operation(&format!(
            "Verification key ({})",
            util::format_file_size(&vk_path)
        ));
    }

    // Step 4: Generate Solidity verifier contract
    if cfg.verbose {
        info!("Generating Solidity verifier contract");
    }
    let contract_timer = Timer::start();
    let verifier_path = directories::get_verifier_contract_path();
    bb_operations::write_solidity_verifier_from_evm_vk(&verifier_path.to_string_lossy())
        .map_err(enhance_error_with_suggestions)?;

    if !cfg.quiet {
        println!(
            "{}",
            success(&format_operation_result(
                "Solidity verifier contract generated",
                &verifier_path,
                &contract_timer
            ))
        );
        summary.add_operation(&format!(
            "Solidity verifier contract ({})",
            util::format_file_size(&verifier_path)
        ));
        summary.print();
        println!();
        println!("🎯 Next steps:");
        println!("  • Generate calldata: bargo evm calldata");
        println!("  • Deploy contract: bargo evm deploy --network <network>");
    }

    Ok(())
}

/// Run the EVM prove workflow
///
/// # Arguments
/// * `cli` - CLI configuration
///
/// # Returns
/// * `Result<()>` - Success or error from workflow
pub fn run_prove(cfg: &Config) -> Result<()> {
    let pkg_name =
        util::get_package_name(cfg.pkg.as_ref()).map_err(enhance_error_with_suggestions)?;

    // Validate that required build files exist
    let required_files = vec![
        util::get_bytecode_path(&pkg_name, Flavour::Bb),
        util::get_witness_path(&pkg_name, Flavour::Bb),
    ];

    if !cfg.dry_run {
        util::validate_files_exist(&required_files).map_err(enhance_error_with_suggestions)?;
        directories::ensure_evm_target_dir().map_err(enhance_error_with_suggestions)?;
    }

    if cfg.dry_run {
        println!(
            "Would run: bb prove -b ./target/bb/{}.json -w ./target/bb/{}.gz -o ./target/evm/ --oracle_hash keccak --output_format bytes_and_fields",
            pkg_name, pkg_name
        );
        println!(
            "Would run: bb write_vk --oracle_hash keccak -b ./target/bb/{}.json -o ./target/evm/",
            pkg_name
        );
        return Ok(());
    }

    let timer = Timer::start();
    bb_operations::generate_evm_proof_and_vk(&pkg_name).map_err(enhance_error_with_suggestions)?;

    if !cfg.quiet {
        let proof_path = util::get_proof_path(Flavour::Evm);
        let vk_path = util::get_vk_path(Flavour::Evm);
        println!(
            "{}",
            success(&format_operation_result(
                "EVM proof and VK generated",
                &proof_path,
                &timer
            ))
        );
        println!("  • Proof: {}", proof_path.display());
        println!("  • VK: {}", vk_path.display());
    }

    Ok(())
}

/// Run the EVM verify workflow
///
/// # Arguments
/// * `cli` - CLI configuration
///
/// # Returns
/// * `Result<()>` - Success or error from workflow
pub fn run_verify(cfg: &Config) -> Result<()> {
    let pkg_name =
        util::get_package_name(cfg.pkg.as_ref()).map_err(enhance_error_with_suggestions)?;

    // Validate that required EVM artifacts exist
    if !cfg.dry_run {
        bb_operations::validate_evm_artifacts().map_err(enhance_error_with_suggestions)?;
    }

    if cfg.dry_run {
        println!(
            "Would run: bb verify -p ./target/evm/proof -k ./target/evm/vk -j ./target/evm/public_inputs"
        );
        return Ok(());
    }

    let timer = Timer::start();
    bb_operations::verify_evm_proof(&pkg_name).map_err(enhance_error_with_suggestions)?;

    if !cfg.quiet {
        println!(
            "{}",
            success(&format!(
                "EVM proof verified successfully ({})",
                timer.elapsed()
            ))
        );
    }

    Ok(())
}

/// Run the EVM deploy workflow
///
/// # Arguments
/// * `cli` - CLI configuration
/// * `network` - Target network for deployment
///
/// # Returns
/// * `Result<()>` - Success or error from workflow
pub fn run_deploy(cfg: &Config, network: &str) -> Result<()> {
    load_env_vars();

    // Validate Foundry installation
    if !cfg.dry_run {
        foundry::validate_foundry_installation().map_err(enhance_error_with_suggestions)?;
    }

    // Check that verifier contract exists
    if !cfg.dry_run && !directories::verifier_contract_exists() {
        return Err(create_smart_error(
            "Verifier contract not found",
            &[
                "Run 'bargo evm gen' first to generate the verifier contract",
                "Ensure the contracts/evm/src/Verifier.sol file exists",
            ],
        ));
    }

    // Get environment variables
    let rpc_url = std::env::var("RPC_URL").map_err(|_| {
        create_smart_error(
            "RPC_URL environment variable not found",
            &[
                "Add to your .env file: RPC_URL=https://eth-mainnet.g.alchemy.com/v2/your_key",
                "Ensure the .env file is loaded in your environment",
            ],
        )
    })?;

    let private_key = std::env::var("PRIVATE_KEY").map_err(|_| {
        create_smart_error(
            "PRIVATE_KEY environment variable not found",
            &[
                "Add to your .env file: PRIVATE_KEY=your_private_key",
                "Ensure the .env file is loaded in your environment",
                "⚠️  Keep your private key secure and never commit it to version control",
            ],
        )
    })?;

    if cfg.dry_run {
        println!("Would deploy Verifier contract to network: {}", network);
        println!("Would use RPC URL: {}", rpc_url);
        return Ok(());
    }

    if cfg.verbose {
        info!("Deploying Verifier contract to {}", network);
    }

    let deploy_timer = Timer::start();
    let contract_address = foundry::deploy_verifier_contract(&rpc_url, &private_key)
        .map_err(enhance_error_with_suggestions)?;

    // Save contract address for future commands
    let address_file = std::path::Path::new("target/evm/.bargo_contract_address");
    if let Some(parent) = address_file.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    std::fs::write(address_file, &contract_address).ok();

    if !cfg.quiet {
        println!(
            "{}",
            success(&format!(
                "Verifier contract deployed successfully ({})",
                deploy_timer.elapsed()
            ))
        );
        println!("Contract address: {}", contract_address);

        let mut summary = OperationSummary::new();
        summary.add_operation(&format!(
            "Verifier contract deployed at: {}",
            contract_address
        ));
        summary.print();
        println!();
        println!("🎯 Next steps:");
        println!("  • Generate calldata: bargo evm calldata");
        println!("  • Verify on-chain: bargo evm verify-onchain");
    }

    Ok(())
}

/// Run the EVM calldata workflow
///
/// # Arguments
/// * `cli` - CLI configuration
///
/// # Returns
/// * `Result<()>` - Success or error from workflow
pub fn run_calldata(cfg: &Config) -> Result<()> {
    load_env_vars();

    // Validate Foundry installation
    if !cfg.dry_run {
        foundry::validate_foundry_installation().map_err(enhance_error_with_suggestions)?;
    }

    // Check that proof fields JSON exists (BB output for EVM)
    let proof_fields_path = std::path::PathBuf::from("./target/evm/proof_fields.json");
    if !cfg.dry_run && !proof_fields_path.exists() {
        return Err(create_smart_error(
            "Proof fields file not found",
            &[
                "Run 'bargo build' and 'bargo evm prove' first to generate proof files",
                "Ensure the target/evm/proof_fields.json file exists",
            ],
        ));
    }

    if cfg.dry_run {
        println!("Would generate calldata from proof fields JSON");
        println!("Would read: {}", proof_fields_path.display());
        return Ok(());
    }

    if cfg.verbose {
        info!("Generating calldata for EVM proof verification");
    }

    // Read proof fields and format for contract call
    let proof_fields_content = std::fs::read_to_string(&proof_fields_path)
        .map_err(|e| color_eyre::eyre::eyre!("Failed to read proof fields file: {}", e))?;

    // Save formatted calldata
    let calldata_path = std::path::PathBuf::from("./target/evm/calldata.json");
    std::fs::write(&calldata_path, &proof_fields_content)
        .map_err(|e| color_eyre::eyre::eyre!("Failed to write calldata file: {}", e))?;

    if !cfg.quiet {
        let calldata_timer = Timer::start();
        println!(
            "{}",
            success(&format_operation_result(
                "Calldata generated",
                &calldata_path,
                &calldata_timer
            ))
        );

        let mut summary = OperationSummary::new();
        summary.add_operation(&format!(
            "Calldata for proof verification ({})",
            util::format_file_size(&calldata_path)
        ));
        summary.print();
        println!();
        println!("🎯 Next step:");
        println!("  • Verify on-chain: bargo evm verify-onchain");
    }

    Ok(())
}

/// Run the EVM verify-onchain workflow
///
/// # Arguments
/// * `cli` - CLI configuration
///
/// # Returns
/// * `Result<()>` - Success or error from workflow
pub fn run_verify_onchain(cfg: &Config) -> Result<()> {
    load_env_vars();

    // Validate Foundry installation
    if !cfg.dry_run {
        foundry::validate_foundry_installation().map_err(enhance_error_with_suggestions)?;
    }

    // Get contract address from saved file or environment
    let contract_address = match std::fs::read_to_string("target/evm/.bargo_contract_address") {
        Ok(saved_address) => saved_address.trim().to_string(),
        Err(_) => std::env::var("CONTRACT_ADDRESS").map_err(|_| {
            create_smart_error(
                "Contract address not found",
                &[
                    "Run 'bargo evm deploy' first to deploy and save contract address",
                    "Or set CONTRACT_ADDRESS environment variable",
                    "The deploy command saves the contract address for verification",
                ],
            )
        })?,
    };

    // Check that calldata exists
    let calldata_path = std::path::PathBuf::from("./target/evm/calldata.json");
    if !cfg.dry_run && !calldata_path.exists() {
        return Err(create_smart_error(
            "Calldata file not found",
            &[
                "Run 'bargo evm calldata' first to generate calldata",
                "Ensure the target/evm/calldata.json file exists",
            ],
        ));
    }

    let rpc_url = std::env::var("RPC_URL").map_err(|_| {
        create_smart_error(
            "RPC_URL environment variable not found",
            &[
                "Add to your .env file: RPC_URL=https://eth-mainnet.g.alchemy.com/v2/your_key",
                "Ensure the .env file is loaded in your environment",
            ],
        )
    })?;

    if cfg.dry_run {
        println!(
            "Would verify proof on-chain at contract: {}",
            contract_address
        );
        println!("Would use calldata from: {}", calldata_path.display());
        return Ok(());
    }

    if cfg.verbose {
        info!("Verifying proof on-chain at contract: {}", contract_address);
    }

    // Read calldata for verification
    let calldata_content = std::fs::read_to_string(&calldata_path)
        .map_err(|e| color_eyre::eyre::eyre!("Failed to read calldata file: {}", e))?;

    if cfg.verbose {
        info!("Using calldata: {}", calldata_content.trim());
    }

    // This is a placeholder for actual on-chain verification
    // The actual implementation would depend on the specific verifier contract interface
    println!("🚧 On-chain verification functionality coming soon");
    println!("Contract address: {}", contract_address);
    println!("RPC URL: {}", rpc_url);
    println!("Calldata: {}", calldata_path.display());

    if !cfg.quiet {
        let mut summary = OperationSummary::new();
        summary.add_operation(&format!(
            "On-chain verification prepared for contract: {}",
            contract_address
        ));
        summary.print();
    }

    Ok(())
}

/// Print dry-run commands for EVM gen workflow
///
/// # Arguments
/// * `pkg` - Package name
///
/// # Returns
/// * `Result<()>` - Success or error
pub fn print_dry_run_commands(pkg: &str) -> Result<()> {
    println!("Would run the following commands:");
    println!();
    println!("# Initialize Foundry project");
    println!("forge init --force contracts/evm");
    println!();
    println!("# Generate EVM proof with keccak oracle");
    println!(
        "bb prove -b ./target/bb/{}.json -w ./target/bb/{}.gz -o ./target/evm/ --oracle_hash keccak --output_format bytes_and_fields",
        pkg, pkg
    );
    println!();
    println!("# Generate EVM verification key");
    println!(
        "bb write_vk --oracle_hash keccak -b ./target/bb/{}.json -o ./target/evm/",
        pkg
    );
    println!();
    println!("# Generate Solidity verifier contract");
    println!("bb write_solidity_verifier -k ./target/evm/vk -o contracts/evm/src/Verifier.sol");

    Ok(())
}
