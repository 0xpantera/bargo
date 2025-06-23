use crate::{backends, cli::Cli, util::{self, Flavour, OperationSummary, Timer, success, format_operation_result, enhance_error_with_suggestions, create_smart_error}};
use color_eyre::Result;
use tracing::info;

pub fn run(cli: &Cli) -> Result<()> {
    let mut summary = OperationSummary::new();

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

    if cli.verbose {
        info!("Running: garaga {}", garaga_args.join(" "));
    }

    if cli.dry_run {
        println!("Would run: garaga {}", garaga_args.join(" "));
        return Ok(());
    }

    let calldata_timer = Timer::start();
    let (stdout, _stderr) = backends::garaga::run_with_output(&garaga_args).map_err(enhance_error_with_suggestions)?;

    let calldata_path = std::path::PathBuf::from("./target/starknet/calldata.json");
    std::fs::write(&calldata_path, stdout.trim()).map_err(|e| {
        create_smart_error(
            &format!("Failed to write calldata file: {}", e),
            &["Check directory permissions", "Ensure target/starknet directory exists"],
        )
    })?;

    if !cli.quiet {
        println!(
            "{}",
            success(&format_operation_result("Calldata generated", &calldata_path, &calldata_timer))
        );
        summary.add_operation(&format!("Calldata for proof verification ({})", util::format_file_size(&calldata_path)));
        summary.print();
        println!();
        println!("🎯 Next step:");
        println!("  • Verify on-chain: bargo cairo verify-onchain");
    }

    Ok(())
}
