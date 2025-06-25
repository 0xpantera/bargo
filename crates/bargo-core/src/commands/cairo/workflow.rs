//! Workflow orchestration for Cairo commands
//!
//! This module provides clean orchestration functions that coordinate between
//! the different Cairo modules to implement complete workflows for each command.

use color_eyre::Result;
use tracing::info;

use crate::{
    config::Config,
    util::{
        self, Flavour, OperationSummary, Timer, create_smart_error, enhance_error_with_suggestions,
        format_operation_result, success,
    },
};

use super::{bb_operations, directories, garaga, load_env_vars};

/// Run the Cairo gen workflow
///
/// This function orchestrates the complete Cairo verifier generation workflow:
/// 1. Generate Starknet proof and VK
/// 2. Generate Cairo verifier contract
/// 3. Set up project structure
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
        info!("Starting Cairo verifier generation workflow");
    }

    // Validate required files exist
    let required_files = vec![
        util::get_bytecode_path(&pkg_name, Flavour::Bb),
        util::get_witness_path(&pkg_name, Flavour::Bb),
    ];

    if !cfg.dry_run {
        util::validate_files_exist(&required_files).map_err(enhance_error_with_suggestions)?;
        directories::validate_cairo_directory_structure()
            .map_err(enhance_error_with_suggestions)?;
    }

    let mut summary = OperationSummary::new();

    // Step 1: Generate Starknet proof
    if cfg.verbose {
        info!("Generating Starknet proof");
    }
    let proof_timer = Timer::start();
    bb_operations::generate_starknet_proof(cfg, &pkg_name)
        .map_err(enhance_error_with_suggestions)?;

    if !cfg.quiet {
        let proof_path = util::get_proof_path(Flavour::Starknet);
        println!(
            "{}",
            success(&format_operation_result(
                "Starknet proof generated",
                &proof_path,
                &proof_timer
            ))
        );
        summary.add_operation(&format!(
            "Starknet proof ({})",
            util::format_file_size(&proof_path)
        ));
    }

    // Step 2: Generate Starknet VK
    if cfg.verbose {
        info!("Generating Starknet verification key");
    }
    let vk_timer = Timer::start();
    bb_operations::generate_starknet_vk(cfg, &pkg_name).map_err(enhance_error_with_suggestions)?;

    if !cfg.quiet {
        let vk_path = util::get_vk_path(Flavour::Starknet);
        println!(
            "{}",
            success(&format_operation_result(
                "Starknet VK generated",
                &vk_path,
                &vk_timer
            ))
        );
        summary.add_operation(&format!(
            "Verification key ({})",
            util::format_file_size(&vk_path)
        ));
    }

    // Step 3: Generate Cairo verifier contract
    if cfg.verbose {
        info!("Generating Cairo verifier contract");
    }
    let contract_timer = Timer::start();

    // TODO: Migrate garaga to runner abstraction in next checkpoint
    if cfg.dry_run {
        println!(
            "Would run: garaga gen --system ultra_starknet_zk_honk --vk ./target/starknet/vk --output ./contracts/cairo/"
        );
    } else {
        garaga::generate_cairo_contract_from_starknet_vk()
            .map_err(enhance_error_with_suggestions)?;
    }

    if !cfg.quiet {
        let cairo_dir = directories::get_cairo_contracts_dir();
        println!(
            "{}",
            success(&format_operation_result(
                "Cairo verifier contract generated",
                &cairo_dir,
                &contract_timer
            ))
        );
        summary.add_operation("Cairo verifier contract");
        summary.print();
        println!();
        println!("🎯 Next steps:");
        println!("  • Generate calldata: bargo cairo calldata");
        println!("  • Deploy contract: bargo cairo deploy [--auto-declare]");
    }

    Ok(())
}

/// Run the Cairo prove workflow
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
        directories::ensure_starknet_target_dir().map_err(enhance_error_with_suggestions)?;
    }

    let timer = Timer::start();
    bb_operations::generate_starknet_proof_and_vk(cfg, &pkg_name)
        .map_err(enhance_error_with_suggestions)?;

    if !cfg.quiet {
        let proof_path = util::get_proof_path(Flavour::Starknet);
        let vk_path = util::get_vk_path(Flavour::Starknet);
        println!(
            "{}",
            success(&format_operation_result(
                "Starknet proof and VK generated",
                &proof_path,
                &timer
            ))
        );
        println!("  • Proof: {}", proof_path.display());
        println!("  • VK: {}", vk_path.display());
    }

    Ok(())
}

/// Run the Cairo verify workflow
///
/// # Arguments
/// * `cli` - CLI configuration
///
/// # Returns
/// * `Result<()>` - Success or error from workflow
pub fn run_verify(cfg: &Config) -> Result<()> {
    let pkg_name =
        util::get_package_name(cfg.pkg.as_ref()).map_err(enhance_error_with_suggestions)?;

    // Validate that required Starknet artifacts exist
    let required_files = vec![
        util::get_proof_path(Flavour::Starknet),
        util::get_vk_path(Flavour::Starknet),
        util::get_public_inputs_path(Flavour::Starknet),
    ];

    if !cfg.dry_run {
        util::validate_files_exist(&required_files).map_err(enhance_error_with_suggestions)?;
    }

    let timer = Timer::start();
    bb_operations::verify_starknet_proof(cfg, &pkg_name).map_err(enhance_error_with_suggestions)?;

    if !cfg.quiet {
        println!(
            "{}",
            success(&format!(
                "Starknet proof verified successfully ({})",
                timer.elapsed()
            ))
        );
    }

    Ok(())
}

/// Run the Cairo calldata workflow (generate calldata)
///
/// # Arguments
/// * `cli` - CLI configuration
///
/// # Returns
/// * `Result<()>` - Success or error
pub fn run_calldata(cfg: &Config) -> Result<()> {
    let mut summary = OperationSummary::new();

    if !cfg.dry_run {
        garaga::validate_starknet_artifacts().map_err(enhance_error_with_suggestions)?;
    }

    if cfg.dry_run {
        let proof_path = util::get_proof_path(Flavour::Starknet);
        let vk_path = util::get_vk_path(Flavour::Starknet);
        let public_inputs_path = util::get_public_inputs_path(Flavour::Starknet);
        println!(
            "Would run: garaga calldata --system ultra_starknet_zk_honk --proof {} --vk {} --public-inputs {}",
            proof_path.display(),
            vk_path.display(),
            public_inputs_path.display()
        );
        return Ok(());
    }

    if cfg.verbose {
        info!("Generating calldata for Starknet proof verification");
    }

    let calldata_timer = Timer::start();
    let calldata_path = garaga::generate_calldata_from_starknet_artifacts()
        .map_err(enhance_error_with_suggestions)?;

    if !cfg.quiet {
        println!(
            "{}",
            success(&format_operation_result(
                "Calldata generated",
                &calldata_path,
                &calldata_timer
            ))
        );
        summary.add_operation(&format!(
            "Calldata for proof verification ({})",
            util::format_file_size(&calldata_path)
        ));
        summary.print();
        println!();
        println!("🎯 Next step:");
        println!("  • Verify on-chain: bargo cairo verify-onchain");
    }

    Ok(())
}

/// Internal function to declare a Cairo contract (used by auto-declare functionality)
///
/// This is an internal function used by the CairoBackend for auto-declare functionality.
/// It should not be called directly by users - use `cairo deploy --auto-declare` instead.
///
/// # Arguments
/// * `cfg` - CLI configuration
/// * `network` - Starknet network to declare on
///
/// # Returns
/// * `Result<()>` - Success or error from workflow
pub(crate) fn internal_declare(cfg: &Config, network: &str) -> Result<()> {
    load_env_vars();

    if cfg.dry_run {
        println!("Would declare contract on network: {}", network);
        return Ok(());
    }

    let cairo_dir = directories::get_cairo_contracts_dir();
    if !cairo_dir.exists() {
        return Err(create_smart_error(
            "Cairo contract directory not found",
            &[
                "Run 'bargo cairo gen' first to generate the verifier contract",
                "Ensure the contracts/cairo directory exists",
            ],
        ));
    }

    if cfg.verbose {
        info!("Declaring Cairo verifier contract on {}", network);
    }

    // Implementation would depend on Starknet CLI integration
    // This is a placeholder for the actual declare logic
    println!("🚧 Contract declaration functionality coming soon");
    println!("Network: {}", network);
    println!("Contract directory: {}", cairo_dir.display());

    Ok(())
}

/// Run the Cairo deploy workflow
///
/// # Arguments
/// * `cli` - CLI configuration
/// * `class_hash` - Optional class hash of the declared contract
///
/// # Returns
/// * `Result<()>` - Success or error from workflow
pub fn run_deploy(cfg: &Config, class_hash: Option<&str>) -> Result<()> {
    load_env_vars();

    if cfg.dry_run {
        let hash = match class_hash {
            Some(hash) => hash.to_string(),
            None => "<class_hash_from_declare>".to_string(), // Placeholder for dry-run
        };
        println!("Would deploy contract with class hash: {}", hash);
        return Ok(());
    }

    let hash = match class_hash {
        Some(hash) => hash.to_string(),
        None => {
            // Try to read class hash from file saved by declare command
            match std::fs::read_to_string("target/starknet/.bargo_class_hash") {
                Ok(saved_hash) => saved_hash.trim().to_string(),
                Err(_) => {
                    return Err(create_smart_error(
                        "No class hash provided and no saved class hash found",
                        &[
                            "Provide class hash with --class-hash option",
                            "Or use 'bargo cairo deploy --auto-declare' to declare and deploy automatically",
                        ],
                    ));
                }
            }
        }
    };

    if cfg.verbose {
        info!("Deploying Cairo verifier contract");
    }

    // Implementation would depend on Starknet CLI integration
    // This is a placeholder for the actual deploy logic
    println!("🚧 Contract deployment functionality coming soon");
    println!("Class hash: {}", hash);

    Ok(())
}

/// Run the Cairo verify-onchain workflow
///
/// # Arguments
/// * `cli` - CLI configuration
/// * `address` - Optional contract address to verify against
///
/// # Returns
/// * `Result<()>` - Success or error from workflow
pub fn run_verify_onchain(cfg: &Config, address: Option<&str>) -> Result<()> {
    load_env_vars();

    let contract_address = match address {
        Some(addr) => addr.to_string(),
        None => {
            // Try to read contract address from file saved by deploy command
            match std::fs::read_to_string("target/starknet/.bargo_contract_address") {
                Ok(saved_address) => saved_address.trim().to_string(),
                Err(_) => {
                    return Err(create_smart_error(
                        "No contract address provided and no saved address found",
                        &[
                            "Provide contract address with --address option",
                            "Or run 'bargo cairo deploy' first to save contract address",
                        ],
                    ));
                }
            }
        }
    };

    // Validate calldata exists
    let calldata_path = std::path::PathBuf::from("./target/starknet/calldata.json");
    if !cfg.dry_run && !calldata_path.exists() {
        return Err(create_smart_error(
            "Calldata file not found",
            &[
                "Run 'bargo cairo calldata' first to generate calldata",
                "Ensure the target/starknet/calldata.json file exists",
            ],
        ));
    }

    if cfg.dry_run {
        println!(
            "Would verify proof on-chain at address: {}",
            contract_address
        );
        return Ok(());
    }

    if cfg.verbose {
        info!("Verifying proof on-chain at address: {}", contract_address);
    }

    // Implementation would depend on Starknet CLI integration
    // This is a placeholder for the actual on-chain verification logic
    println!("🚧 On-chain verification functionality coming soon");
    println!("Contract address: {}", contract_address);
    println!("Calldata: {}", calldata_path.display());

    Ok(())
}
