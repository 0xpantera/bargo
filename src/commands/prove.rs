use crate::{backends, cli::Cli, util::{self, OperationSummary, Timer, Flavour, success, format_operation_result, enhance_error_with_suggestions, create_smart_error}};
use color_eyre::Result;
use tracing::info;

pub fn run(cli: &Cli, skip_verify: bool) -> Result<()> {
    let pkg_name = util::get_package_name(cli.pkg.as_ref()).map_err(enhance_error_with_suggestions)?;
    let mut summary = OperationSummary::new();

    let required_files = vec![
        util::get_bytecode_path(&pkg_name, Flavour::Bb),
        util::get_witness_path(&pkg_name, Flavour::Bb),
    ];
    if !cli.dry_run {
        util::validate_files_exist(&required_files).map_err(enhance_error_with_suggestions)?;
    }

    let bytecode_path = util::get_bytecode_path(&pkg_name, Flavour::Bb);
    let witness_path = util::get_witness_path(&pkg_name, Flavour::Bb);
    let bytecode_str = bytecode_path.to_string_lossy();
    let witness_str = witness_path.to_string_lossy();

    let prove_args = vec![
        "prove",
        "-b",
        &bytecode_str,
        "-w",
        &witness_str,
        "-o",
        "./target/bb/",
    ];

    if cli.verbose {
        info!("Running: bb {}", prove_args.join(" "));
    }

    if cli.dry_run {
        println!("Would run: bb {}", prove_args.join(" "));
        if !skip_verify {
            let vk_args = vec!["write_vk", "-b", &bytecode_str, "-o", "./target/bb/"];
            println!("Would run: bb {}", vk_args.join(" "));
            println!("Would run: bb verify -k ./target/bb/vk -p ./target/bb/proof");
        }
        return Ok(());
    }

    std::fs::create_dir_all("./target/bb").map_err(|e| {
        create_smart_error(
            &format!("Failed to create target/bb directory: {}", e),
            &["Check directory permissions", "Ensure you have write access to the current directory"],
        )
    })?;

    let prove_timer = Timer::start();
    backends::bb::run(&prove_args).map_err(enhance_error_with_suggestions)?;

    if !cli.quiet {
        let proof_path = util::get_proof_path(Flavour::Bb);
        println!(
            "{}",
            success(&format_operation_result("Proof generated", &proof_path, &prove_timer))
        );
        summary.add_operation(&format!("Proof generated ({})", util::format_file_size(&proof_path)));
    }

    let vk_args = vec!["write_vk", "-b", &bytecode_str, "-o", "./target/bb/"];

    if cli.verbose {
        info!("Running: bb {}", vk_args.join(" "));
    }

    let vk_timer = Timer::start();
    backends::bb::run(&vk_args).map_err(enhance_error_with_suggestions)?;

    if !cli.quiet {
        let vk_path = util::get_vk_path(Flavour::Bb);
        println!(
            "{}",
            success(&format_operation_result("VK saved", &vk_path, &vk_timer))
        );
        summary.add_operation(&format!(
            "Verification key generated ({})",
            util::format_file_size(&vk_path)
        ));
    }

    if !skip_verify {
        let proof_path = util::get_proof_path(Flavour::Bb);
        let vk_path = util::get_vk_path(Flavour::Bb);
        let public_inputs_path = util::get_public_inputs_path(Flavour::Bb);
        let vk_str = vk_path.to_string_lossy();
        let proof_str = proof_path.to_string_lossy();
        let public_inputs_str = public_inputs_path.to_string_lossy();
        let verify_args = vec![
            "verify",
            "-k",
            &vk_str,
            "-p",
            &proof_str,
            "-i",
            &public_inputs_str,
        ];

        if cli.verbose {
            info!("Running: bb {}", verify_args.join(" "));
        }

        let verify_timer = Timer::start();
        backends::bb::run(&verify_args).map_err(enhance_error_with_suggestions)?;

        if !cli.quiet {
            println!(
                "{}",
                success(&format!("Proof verified successfully ({})", verify_timer.elapsed()))
            );
            summary.add_operation("Proof verification completed");
        }
    }

    Ok(())
}
