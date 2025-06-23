use crate::{backends, cli::Cli, util::{self, OperationSummary, Timer, Flavour, success, enhance_error_with_suggestions, create_smart_error}};
use color_eyre::Result;
use tracing::info;

pub fn run(cli: &Cli, address: Option<&str>) -> Result<()> {
    let mut summary = OperationSummary::new();

    let contract_address = match address {
        Some(addr) => addr.to_string(),
        None => match std::fs::read_to_string("target/starknet/.bargo_contract_address") {
            Ok(saved_address) => {
                let addr = saved_address.trim().to_string();
                if !cli.quiet {
                    println!("{}", success(&format!("Using saved contract address: {}", addr)));
                }
                addr
            }
            Err(_) => {
                return Err(create_smart_error(
                    "Contract address is required for verification",
                    &["Provide a contract address with --address <ADDRESS>", "Or run 'bargo cairo deploy' first to save the contract address", "The deploy command will save the contract address for verification"],
                ));
            }
        },
    };

    let required_files = vec![
        util::get_proof_path(Flavour::Starknet),
        util::get_vk_path(Flavour::Starknet),
        util::get_public_inputs_path(Flavour::Starknet),
    ];

    if !cli.dry_run {
        util::validate_files_exist(&required_files).map_err(enhance_error_with_suggestions)?;
    }

    let proof_path = util::get_proof_path(Flavour::Starknet);
    let vk_path = util::get_vk_path(Flavour::Starknet);
    let public_inputs_path = util::get_public_inputs_path(Flavour::Starknet);
    let proof_str = proof_path.to_string_lossy();
    let vk_str = vk_path.to_string_lossy();
    let public_inputs_str = public_inputs_path.to_string_lossy();

    let garaga_args = vec![
        "verify-onchain",
        "--system",
        "ultra_starknet_zk_honk",
        "--contract-address",
        &contract_address,
        "--network",
        "mainnet",
        "--vk",
        &vk_str,
        "--proof",
        &proof_str,
        "--public-inputs",
        &public_inputs_str,
    ];

    if cli.verbose {
        info!("Running: garaga {}", garaga_args.join(" "));
        info!("Verifying proof on-chain at address: {}", contract_address);
    }

    if cli.dry_run {
        println!("Would run: garaga {}", garaga_args.join(" "));
        return Ok(());
    }

    let verify_timer = Timer::start();
    backends::garaga::run(&garaga_args).map_err(enhance_error_with_suggestions)?;

    if !cli.quiet {
        println!(
            "{}",
            success(&format!("Proof verified on-chain successfully ({})", verify_timer.elapsed()))
        );
        summary.add_operation(&format!("On-chain verification completed at address: {}", contract_address));
        summary.print();
    }

    Ok(())
}
