use crate::{backends, cli::Cli, util::{self, Flavour, OperationSummary, Timer, success, format_operation_result, enhance_error_with_suggestions, create_smart_error}};
use color_eyre::Result;
use tracing::info;

pub fn run(cli: &Cli) -> Result<()> {
    let pkg_name = util::get_package_name(cli.pkg.as_ref())?;
    let mut summary = OperationSummary::new();

    let required_files = vec![
        util::get_bytecode_path(&pkg_name, Flavour::Bb),
        util::get_witness_path(&pkg_name, Flavour::Bb),
    ];

    if !cli.dry_run {
        util::validate_files_exist(&required_files).map_err(enhance_error_with_suggestions)?;
    }

    std::fs::create_dir_all("./target/starknet").map_err(|e| {
        create_smart_error(
            &format!("Failed to create target/starknet directory: {}", e),
            &["Check directory permissions", "Ensure you have write access to the current directory"],
        )
    })?;

    let bytecode_path = util::get_bytecode_path(&pkg_name, Flavour::Bb);
    let witness_path = util::get_witness_path(&pkg_name, Flavour::Bb);
    let bytecode_str = bytecode_path.to_string_lossy();
    let witness_str = witness_path.to_string_lossy();

    let prove_args = vec![
        "prove",
        "-s",
        "ultra_honk",
        "--oracle_hash",
        "starknet",
        "--zk",
        "-b",
        &bytecode_str,
        "-w",
        &witness_str,
        "-o",
        "./target/starknet/",
    ];

    if cli.verbose {
        info!("Running: bb {}", prove_args.join(" "));
    }

    if cli.dry_run {
        println!("Would run: bb {}", prove_args.join(" "));
        let vk_args = vec![
            "write_vk",
            "-b",
            &bytecode_str,
            "-o",
            "./target/starknet/",
            "--oracle_hash",
            "starknet",
        ];
        println!("Would run: bb {}", vk_args.join(" "));
        println!(
            "Would run: garaga gen --system ultra_starknet_zk_honk --vk target/starknet/vk -o contracts/Verifier.cairo"
        );
        return Ok(());
    }

    let prove_timer = Timer::start();
    backends::bb::run(&prove_args).map_err(enhance_error_with_suggestions)?;

    if !cli.quiet {
        let proof_path = util::get_proof_path(Flavour::Starknet);
        println!(
            "{}",
            success(&format_operation_result("Starknet proof generated", &proof_path, &prove_timer))
        );
        summary.add_operation(&format!(
            "Starknet proof generated ({})",
            util::format_file_size(&proof_path)
        ));
    }

    let vk_args = vec![
        "write_vk",
        "-b",
        &bytecode_str,
        "-o",
        "./target/starknet/",
        "--oracle_hash",
        "starknet",
    ];

    if cli.verbose {
        info!("Running: bb {}", vk_args.join(" "));
    }

    let vk_timer = Timer::start();
    backends::bb::run(&vk_args).map_err(enhance_error_with_suggestions)?;

    if !cli.quiet {
        let vk_path = util::get_vk_path(Flavour::Starknet);
        println!(
            "{}",
            success(&format_operation_result("Starknet VK generated", &vk_path, &vk_timer))
        );
        summary.add_operation(&format!(
            "Starknet verification key ({})",
            util::format_file_size(&vk_path)
        ));
    }

    std::fs::create_dir_all("./contracts").map_err(|e| {
        create_smart_error(
            &format!("Failed to create contracts directory: {}", e),
            &["Check directory permissions", "Ensure you have write access to the current directory"],
        )
    })?;

    let vk_path = util::get_vk_path(Flavour::Starknet);
    let vk_str = vk_path.to_string_lossy();
    let garaga_args = vec![
        "gen",
        "--system",
        "ultra_starknet_zk_honk",
        "--vk",
        &vk_str,
        "--project-name",
        "cairo",
    ];

    if cli.verbose {
        info!("Running: garaga {}", garaga_args.join(" "));
    }

    let garaga_timer = Timer::start();
    backends::garaga::run(&garaga_args).map_err(enhance_error_with_suggestions)?;

    let temp_cairo_dir = std::path::PathBuf::from("./cairo");
    let target_cairo_dir = std::path::PathBuf::from("./contracts/cairo");

    if temp_cairo_dir.exists() {
        if target_cairo_dir.exists() {
            std::fs::remove_dir_all(&target_cairo_dir).map_err(|e| {
                create_smart_error(&format!("Failed to remove existing cairo directory: {}", e), &["Check directory permissions"])
            })?;
        }
        std::fs::rename(&temp_cairo_dir, &target_cairo_dir).map_err(|e| {
            create_smart_error(
                &format!("Failed to move cairo directory: {}", e),
                &["Check directory permissions", "Ensure you have write access to the contracts directory"],
            )
        })?;
    }

    let scarb_build_args = vec!["build"];
    let scarb_timer = Timer::start();

    if cli.verbose {
        info!(
            "Running: scarb {} in contracts/cairo/",
            scarb_build_args.join(" ")
        );
    }

    if !cli.dry_run {
        let scarb_output = std::process::Command::new("scarb")
            .args(&scarb_build_args)
            .current_dir("./contracts/cairo")
            .output()
            .map_err(|e| {
                create_smart_error(
                    &format!("Failed to run scarb build: {}", e),
                    &[
                        "Ensure scarb is installed and available in PATH",
                        "Check that the Cairo project was generated correctly",
                        "Verify you're in the correct directory",
                    ],
                )
            })?;

        if !scarb_output.status.success() {
            let stderr = String::from_utf8_lossy(&scarb_output.stderr);
            return Err(create_smart_error(
                &format!("Scarb build failed: {}", stderr),
                &[
                    "Check the Cairo contract syntax",
                    "Ensure all dependencies are available",
                    "Review the error message above for details",
                ],
            ));
        }
    }

    if !cli.quiet {
        let cairo_verifier_path = std::path::PathBuf::from("./contracts/cairo/");
        println!(
            "{}",
            success(&format_operation_result("Cairo verifier generated", &cairo_verifier_path, &garaga_timer))
        );
        println!(
            "{}",
            success(&format!("Cairo project built successfully ({})", scarb_timer.elapsed()))
        );
        summary.add_operation("Cairo verifier contract generated");
        summary.add_operation("Cairo project compiled successfully");
        summary.print();
    }

    Ok(())
}
