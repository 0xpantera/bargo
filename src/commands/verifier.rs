use crate::{backends, cli::Cli, util::{self, Flavour, OperationSummary, Timer, success, format_operation_result, enhance_error_with_suggestions, create_smart_error}};
use color_eyre::Result;
use tracing::info;

pub fn run(cli: &Cli) -> Result<()> {
    let pkg_name = util::get_package_name(cli.pkg.as_ref()).map_err(enhance_error_with_suggestions)?;
    let mut summary = OperationSummary::new();

    let required_files = vec![util::get_bytecode_path(&pkg_name, Flavour::Bb)];

    if !cli.dry_run {
        util::validate_files_exist(&required_files).map_err(enhance_error_with_suggestions)?;
    }

    let bytecode_path = util::get_bytecode_path(&pkg_name, Flavour::Bb);
    let bytecode_str = bytecode_path.to_string_lossy();
    let vk_args = vec![
        "write_vk",
        "--oracle_hash",
        "keccak",
        "-b",
        &bytecode_str,
        "-o",
        "./target/bb/",
    ];

    if cli.verbose {
        info!("Running: bb {}", vk_args.join(" "));
    }

    if cli.dry_run {
        println!("Would run: bb {}", vk_args.join(" "));
        println!("Would run: bb write_solidity_verifier -k ./target/bb/vk -o ./contracts/Verifier.sol");
        return Ok(());
    }

    std::fs::create_dir_all("./target/bb").map_err(|e| {
        create_smart_error(
            &format!("Failed to create target/bb directory: {}", e),
            &["Check directory permissions", "Ensure you have write access to the current directory"],
        )
    })?;

    let vk_timer = Timer::start();
    backends::bb::run(&vk_args).map_err(enhance_error_with_suggestions)?;

    if !cli.quiet {
        let vk_path = util::get_vk_path(Flavour::Bb);
        println!(
            "{}",
            success(&format_operation_result("VK (keccak optimized)", &vk_path, &vk_timer))
        );
        summary.add_operation(&format!(
            "Verification key with keccak optimization ({})",
            util::format_file_size(&vk_path)
        ));
    }

    std::fs::create_dir_all("./contracts").map_err(|e| {
        create_smart_error(
            &format!("Failed to create contracts directory: {}", e),
            &["Check directory permissions", "Ensure you have write access to the current directory"],
        )
    })?;

    let vk_path = util::get_vk_path(Flavour::Bb);
    let vk_str = vk_path.to_string_lossy();
    let solidity_args = vec![
        "write_solidity_verifier",
        "-k",
        &vk_str,
        "-o",
        "./contracts/Verifier.sol",
    ];

    if cli.verbose {
        info!("Running: bb {}", solidity_args.join(" "));
    }

    let solidity_timer = Timer::start();
    backends::bb::run(&solidity_args).map_err(enhance_error_with_suggestions)?;

    if !cli.quiet {
        let verifier_path = std::path::PathBuf::from("./contracts/Verifier.sol");
        println!(
            "{}",
            success(&format_operation_result("Solidity verifier", &verifier_path, &solidity_timer))
        );
        summary.add_operation(&format!(
            "Solidity verifier contract ({})",
            util::format_file_size(&verifier_path)
        ));
        summary.print();
    }

    Ok(())
}
