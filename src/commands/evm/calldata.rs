use crate::{backends, cli::Cli, util::{self, OperationSummary, Timer, success, format_operation_result, enhance_error_with_suggestions, create_smart_error}};
use color_eyre::Result;

pub fn run(cli: &Cli) -> Result<()> {
    let mut summary = OperationSummary::new();

    backends::foundry::ensure_available().map_err(enhance_error_with_suggestions)?;

    let proof_fields_path = std::path::PathBuf::from("./target/bb/proof_fields.json");
    if !proof_fields_path.exists() {
        return Err(create_smart_error(
            "Proof fields file not found",
            &["Run 'bargo evm gen' first to generate field-formatted proof", "This command requires the proof in JSON field format"],
        ));
    }

    let public_inputs_fields_path = std::path::PathBuf::from("./target/bb/public_inputs_fields.json");
    if !public_inputs_fields_path.exists() {
        return Err(create_smart_error(
            "Public inputs fields file not found",
            &["Public inputs fields should be generated during proving", "Run 'bargo evm gen' to regenerate with field format"],
        ));
    }

    if !cli.quiet {
        println!("📝 Generating calldata for proof verification...");
    }

    let proof_fields_content = std::fs::read_to_string(&proof_fields_path).map_err(|e| {
        create_smart_error(&format!("Failed to read proof fields file: {}", e), &["Check file permissions and try regenerating the proof"])
    })?;

    let public_inputs_fields_content = std::fs::read_to_string(&public_inputs_fields_path).map_err(|e| {
        create_smart_error(&format!("Failed to read public inputs fields file: {}", e), &["Check file permissions and try regenerating the proof"])
    })?;

    let calldata_timer = Timer::start();
    let calldata = backends::foundry::run_cast_with_output(&[
        "calldata",
        "--format-json",
        &proof_fields_content,
        &public_inputs_fields_content,
    ])
    .map_err(enhance_error_with_suggestions)?
    .0;

    std::fs::create_dir_all("./target/bb").map_err(|e| {
        create_smart_error(&format!("Failed to create target/bb directory: {}", e), &["Check directory permissions"])
    })?;
    let calldata_path = std::path::PathBuf::from("./target/bb/calldata");
    std::fs::write(&calldata_path, calldata).map_err(|e| {
        create_smart_error(&format!("Failed to write calldata file: {}", e), &["Check directory permissions"])
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
        println!("  • Verify on-chain: bargo evm verify-onchain");
    }

    Ok(())
}
