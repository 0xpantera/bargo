use crate::{backends, cli::Cli, util::{self, Flavour, OperationSummary, Timer, success, format_operation_result, enhance_error_with_suggestions, create_smart_error}};
use color_eyre::Result;
use tracing::info;

pub fn run(cli: &Cli) -> Result<()> {
    let mut summary = OperationSummary::new();

    backends::foundry::ensure_available().map_err(enhance_error_with_suggestions)?;

    let pkg_name = util::get_package_name(cli.pkg.as_ref())?;

    let proof_path = util::get_proof_path(Flavour::Bb);
    let witness_path = util::get_witness_path(&pkg_name, Flavour::Bb);
    let bytecode_path = util::get_bytecode_path(&pkg_name, Flavour::Bb);

    if !proof_path.exists() || !util::get_vk_path(Flavour::Bb).exists() {
        if !cli.quiet {
            println!("🔄 Generating keccak-optimized proof and verification key...");
        }

        if !bytecode_path.exists() || !witness_path.exists() {
            return Err(create_smart_error(
                "Missing bytecode or witness files",
                &["Run 'bargo build' first to generate bytecode and witness", "Ensure your Noir circuit compiles successfully"],
            ));
        }

        let bytecode_str = bytecode_path.to_string_lossy();
        let witness_str = witness_path.to_string_lossy();
        let prove_args = vec![
            "prove",
            "-b",
            &bytecode_str,
            "-w",
            &witness_str,
            "-o",
            "./target/bb",
            "--oracle_hash",
            "keccak",
            "--output_format",
            "fields",
        ];

        if cli.verbose {
            info!("Running: bb {}", prove_args.join(" "));
        }

        if !cli.dry_run {
            std::fs::create_dir_all("./target/bb").map_err(|e| {
                create_smart_error(&format!("Failed to create target/bb directory: {}", e), &["Check directory permissions", "Ensure you have write access"])
            })?;

            let prove_timer = Timer::start();
            backends::bb::run(&prove_args).map_err(enhance_error_with_suggestions)?;

            if !cli.quiet {
                println!(
                    "{}",
                    success(&format_operation_result("Keccak proof", &proof_path, &prove_timer))
                );
            }
        }

        let vk_args = vec![
            "write_vk",
            "-b",
            &bytecode_str,
            "-o",
            "./target/bb",
            "--oracle_hash",
            "keccak",
        ];

        if cli.verbose {
            info!("Running: bb {}", vk_args.join(" "));
        }

        if !cli.dry_run {
            let vk_timer = Timer::start();
            backends::bb::run(&vk_args).map_err(enhance_error_with_suggestions)?;

            if !cli.quiet {
                let vk_path = util::get_vk_path(Flavour::Bb);
                println!(
                    "{}",
                    success(&format_operation_result("Keccak VK", &vk_path, &vk_timer))
                );
                summary.add_operation(&format!(
                    "Verification key with keccak optimization ({})",
                    util::format_file_size(&vk_path)
                ));
            }
        }
    }

    let foundry_root = std::path::PathBuf::from("./contracts/evm");

    if !foundry_root.exists() {
        if !cli.quiet {
            println!("🔨 Initializing Foundry project...");
        }

        if cli.verbose {
            info!("Running: forge init --force contracts/evm");
        }

        if !cli.dry_run {
            let init_timer = Timer::start();
            backends::foundry::run_forge(&["init", "--force", "contracts/evm"]).map_err(enhance_error_with_suggestions)?;

            if !cli.quiet {
                println!(
                    "{}",
                    success(&format!("Foundry project initialized ({})", init_timer.elapsed()))
                );
                summary.add_operation("Foundry project structure created");
            }
        }
    }

    if !cli.quiet {
        println!("📝 Generating Solidity verifier contract...");
    }

    let verifier_path = foundry_root.join("src/Verifier.sol");
    let vk_path = util::get_vk_path(Flavour::Bb);
    let vk_str = vk_path.to_string_lossy();
    let verifier_str = verifier_path.to_string_lossy();

    let solidity_args = vec!["write_solidity_verifier", "-k", &vk_str, "-o", &verifier_str];

    if cli.verbose {
        info!("Running: bb {}", solidity_args.join(" "));
    }

    if !cli.dry_run {
        std::fs::create_dir_all(foundry_root.join("src")).map_err(|e| {
            create_smart_error(&format!("Failed to create src directory: {}", e), &["Check directory permissions"])
        })?;

        let solidity_timer = Timer::start();
        backends::bb::run(&solidity_args).map_err(enhance_error_with_suggestions)?;

        if !cli.quiet {
            println!(
                "{}",
                success(&format_operation_result("Solidity verifier", &verifier_path, &solidity_timer))
            );
            summary.add_operation(&format!(
                "Solidity verifier contract ({})",
                util::format_file_size(&verifier_path)
            ));
        }
    }

    let foundry_toml = foundry_root.join("foundry.toml");
    if !foundry_toml.exists() {
        if !cli.quiet && cli.verbose {
            println!("📄 Creating foundry.toml configuration...");
        }

        if !cli.dry_run {
            let toml_content = r#"[profile.default]
solc_version = "0.8.25"
optimizer = true
optimizer_runs = 200

[fmt]
line_length = 100
tab_width = 4
bracket_spacing = true
"#;
            std::fs::write(&foundry_toml, toml_content).map_err(|e| {
                create_smart_error(&format!("Failed to create foundry.toml: {}", e), &["Check directory permissions"])
            })?;

            if !cli.quiet && cli.verbose {
                println!(
                    "{}",
                    success(&format!("foundry.toml configuration created ({})", util::format_file_size(&foundry_toml)))
                );
            }
            summary.add_operation("Foundry configuration file created");
        }
    }

    if !cli.quiet {
        println!("🔨 Building Foundry project...");
    }

    if cli.verbose {
        info!("Running: forge build --root contracts/evm");
    }

    if !cli.dry_run {
        let build_timer = Timer::start();
        backends::foundry::run_forge(&["build", "--root", "contracts/evm"]).map_err(enhance_error_with_suggestions)?;

        if !cli.quiet {
            println!(
                "{}",
                success(&format!("Foundry project compiled successfully ({})", build_timer.elapsed()))
            );
            summary.add_operation("Foundry project compiled successfully");
        }
    }

    if !cli.quiet {
        summary.print();
        println!();
        println!("🎯 Next steps:");
        println!("  1. Set up your .env file with RPC_URL and PRIVATE_KEY");
        println!("  2. Deploy contract: bargo evm deploy --network sepolia");
        println!("  3. Generate calldata: bargo evm calldata");
        println!("  4. Verify on-chain: bargo evm verify-onchain");
    }

    Ok(())
}
