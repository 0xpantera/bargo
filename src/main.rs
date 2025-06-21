use clap::{Parser, Subcommand};
use color_eyre::Result;
use tracing::{info, warn};

mod backends;
mod util;

use util::{
    OperationSummary, Timer, create_smart_error, enhance_error_with_suggestions,
    format_operation_result, info, path, print_banner, success,
};

/// A developer-friendly CLI wrapper for Noir ZK development
#[derive(Parser)]
#[command(
    name = "bargo",
    about = "A developer-friendly CLI wrapper for Noir ZK development",
    long_about = "bargo consolidates nargo and bb workflows into a single, opinionated tool that 'just works' in a standard Noir workspace.",
    version
)]
struct Cli {
    /// Enable verbose logging (shows underlying commands)
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Print commands without executing them
    #[arg(long, global = true)]
    dry_run: bool,

    /// Override package name (auto-detected from Nargo.toml)
    #[arg(long, global = true)]
    pkg: Option<String>,

    /// Minimize output
    #[arg(short, long, global = true)]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check circuit syntax and dependencies
    #[command(about = "Run nargo check to validate circuit syntax and dependencies")]
    Check,

    /// Build circuit (compile + execute to generate bytecode and witness)
    #[command(about = "Run nargo execute to generate bytecode and witness files")]
    Build,

    /// Generate proof and verify it
    #[command(about = "Generate proof using bb, write verification key, and verify proof")]
    Prove {
        /// Skip verification step after proving
        #[arg(long)]
        skip_verify: bool,
    },

    /// Verify an existing proof
    #[command(about = "Verify an existing proof using bb verify")]
    Verify,

    /// Generate Solidity verifier contract
    #[command(about = "Generate Solidity verifier contract optimized for Ethereum deployment")]
    Solidity,

    /// Clean build artifacts
    #[command(about = "Remove target directory and all build artifacts")]
    Clean,

    /// Clean and rebuild (equivalent to clean + build)
    #[command(about = "Remove target directory and rebuild from scratch")]
    Rebuild,
}

fn main() -> Result<()> {
    // Install color-eyre for pretty error reporting
    color_eyre::install()?;

    let cli = Cli::parse();

    // Initialize logging based on verbosity
    setup_logging(cli.verbose, cli.quiet)?;

    if cli.verbose {
        info!("🚀 Starting bargo");
        if cli.dry_run {
            warn!("🔍 Dry run mode - commands will be printed but not executed");
        }
    }

    // Route to appropriate command handler
    match cli.command {
        Commands::Check => {
            if !cli.quiet {
                print_banner("check");
            }
            handle_check(&cli)?;
        }
        Commands::Build => {
            if !cli.quiet {
                print_banner("build");
            }
            handle_build(&cli)?;
        }
        Commands::Prove { skip_verify } => {
            if !cli.quiet {
                print_banner("prove");
                if skip_verify {
                    println!("⚡ Skipping verification step\n");
                }
            }
            handle_prove(&cli, skip_verify)?;
        }
        Commands::Verify => {
            if !cli.quiet {
                print_banner("verify");
            }
            handle_verify(&cli)?;
        }
        Commands::Solidity => {
            if !cli.quiet {
                print_banner("solidity");
            }
            handle_solidity(&cli)?;
        }
        Commands::Clean => {
            if !cli.quiet {
                print_banner("clean");
            }
            handle_clean(&cli)?;
        }
        Commands::Rebuild => {
            if !cli.quiet {
                print_banner("rebuild");
            }
            handle_rebuild(&cli)?;
        }
    }

    if cli.verbose {
        info!("✨ bargo completed successfully");
    }

    Ok(())
}

fn setup_logging(verbose: bool, quiet: bool) -> Result<()> {
    use tracing_subscriber::{EnvFilter, fmt};

    if quiet {
        // Only show errors
        let subscriber = fmt()
            .with_max_level(tracing::Level::ERROR)
            .with_target(false)
            .with_level(true)
            .finish();
        tracing::subscriber::set_global_default(subscriber)?;
    } else if verbose {
        // Show info and above, plus set RUST_LOG environment
        unsafe {
            std::env::set_var("RUST_LOG", "info");
        }
        let subscriber = fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .with_target(false)
            .with_level(true)
            .finish();
        tracing::subscriber::set_global_default(subscriber)?;
    } else {
        // Default: only show warnings and errors
        let subscriber = fmt()
            .with_max_level(tracing::Level::WARN)
            .with_target(false)
            .with_level(false)
            .finish();
        tracing::subscriber::set_global_default(subscriber)?;
    }

    Ok(())
}

fn handle_check(cli: &Cli) -> Result<()> {
    let args = build_nargo_args(cli, &[])?;

    if cli.verbose {
        info!("Running: nargo check {}", args.join(" "));
    }

    if cli.dry_run {
        println!("Would run: nargo check {}", args.join(" "));
        return Ok(());
    }

    backends::nargo::run(&["check"])
}

fn handle_build(cli: &Cli) -> Result<()> {
    let pkg_name = get_package_name(cli)?;
    let mut summary = OperationSummary::new();

    // Check if rebuild is needed (smart rebuild detection)
    if !cli.dry_run {
        let needs_rebuild = util::needs_rebuild(&pkg_name)?;
        if !needs_rebuild && !cli.quiet {
            println!("{}", success("Build is up to date"));
            return Ok(());
        }
        if needs_rebuild && cli.verbose {
            info!("Source files have changed, rebuilding...");
        }
    }

    let args = build_nargo_args(cli, &[])?;

    if cli.verbose {
        info!("Running: nargo execute {}", args.join(" "));
    }

    if cli.dry_run {
        println!("Would run: nargo execute {}", args.join(" "));
        return Ok(());
    }

    let timer = Timer::start();
    let result = backends::nargo::run(&["execute"]);

    match result {
        Ok(()) => {
            if !cli.quiet {
                let bytecode_path = util::get_bytecode_path(&pkg_name);
                let witness_path = util::get_witness_path(&pkg_name);

                println!(
                    "{}",
                    success(&format_operation_result(
                        "Bytecode generated",
                        &bytecode_path,
                        &timer
                    ))
                );

                // Create a new timer for witness (they're generated together but we show separate timing)
                let witness_timer = Timer::start();
                println!(
                    "{}",
                    success(&format_operation_result(
                        "Witness generated",
                        &witness_path,
                        &witness_timer
                    ))
                );

                summary.add_operation(&format!("Circuit compiled for {}", path(&pkg_name)));
                summary.add_operation(&format!(
                    "Bytecode generated ({})",
                    util::format_file_size(&bytecode_path)
                ));
                summary.add_operation(&format!(
                    "Witness generated ({})",
                    util::format_file_size(&witness_path)
                ));
                summary.print();
            }
            Ok(())
        }
        Err(e) => Err(enhance_error_with_suggestions(e)),
    }
}

fn handle_prove(cli: &Cli, skip_verify: bool) -> Result<()> {
    let pkg_name = get_package_name(cli).map_err(enhance_error_with_suggestions)?;
    let mut summary = OperationSummary::new();

    // Validate that required build files exist
    let required_files = vec![
        util::get_bytecode_path(&pkg_name),
        util::get_witness_path(&pkg_name),
    ];

    if !cli.dry_run {
        util::validate_files_exist(&required_files).map_err(enhance_error_with_suggestions)?;
    }

    // Build bb prove arguments
    let bytecode_path = util::get_bytecode_path(&pkg_name);
    let witness_path = util::get_witness_path(&pkg_name);
    let bytecode_str = bytecode_path.to_string_lossy();
    let witness_str = witness_path.to_string_lossy();

    let prove_args = vec![
        "prove",
        "-b",
        &bytecode_str,
        "-w",
        &witness_str,
        "-o",
        "./target/",
    ];

    if cli.verbose {
        info!("Running: bb {}", prove_args.join(" "));
    }

    if cli.dry_run {
        println!("Would run: bb {}", prove_args.join(" "));
        if !skip_verify {
            let vk_args = vec!["write_vk", "-b", &bytecode_str, "-o", "./target/"];
            println!("Would run: bb {}", vk_args.join(" "));
            println!("Would run: bb verify -k ./target/vk -p ./target/proof");
        }
        return Ok(());
    }

    // Run bb prove
    let prove_timer = Timer::start();
    backends::bb::run(&prove_args).map_err(enhance_error_with_suggestions)?;

    if !cli.quiet {
        let proof_path = util::get_proof_path();
        println!(
            "{}",
            success(&format_operation_result(
                "Proof generated",
                &proof_path,
                &prove_timer
            ))
        );
        summary.add_operation(&format!(
            "Proof generated ({})",
            util::format_file_size(&proof_path)
        ));
    }

    // Generate verification key
    let vk_args = vec!["write_vk", "-b", &bytecode_str, "-o", "./target/"];

    if cli.verbose {
        info!("Running: bb {}", vk_args.join(" "));
    }

    let vk_timer = Timer::start();
    backends::bb::run(&vk_args).map_err(enhance_error_with_suggestions)?;

    if !cli.quiet {
        let vk_path = util::get_vk_path();
        println!(
            "{}",
            success(&format_operation_result("VK saved", &vk_path, &vk_timer))
        );
        summary.add_operation(&format!(
            "Verification key generated ({})",
            util::format_file_size(&vk_path)
        ));
    }

    // Verify proof unless skipped
    if !skip_verify {
        let proof_path = util::get_proof_path();
        let vk_path = util::get_vk_path();
        let vk_str = vk_path.to_string_lossy();
        let proof_str = proof_path.to_string_lossy();
        let verify_args = vec!["verify", "-k", &vk_str, "-p", &proof_str];

        if cli.verbose {
            info!("Running: bb {}", verify_args.join(" "));
        }

        let verify_timer = Timer::start();
        backends::bb::run(&verify_args).map_err(enhance_error_with_suggestions)?;

        if !cli.quiet {
            println!(
                "{}",
                success(&format!(
                    "Proof verified successfully ({})",
                    verify_timer.elapsed()
                ))
            );
            summary.add_operation("Proof verification completed");
        }
    }

    Ok(())
}

fn handle_verify(cli: &Cli) -> Result<()> {
    // Validate that required files exist
    let required_files = vec![util::get_proof_path(), util::get_vk_path()];

    if !cli.dry_run {
        util::validate_files_exist(&required_files).map_err(enhance_error_with_suggestions)?;
    }

    let proof_path = util::get_proof_path();
    let vk_path = util::get_vk_path();
    let vk_str = vk_path.to_string_lossy();
    let proof_str = proof_path.to_string_lossy();
    let verify_args = vec!["verify", "-k", &vk_str, "-p", &proof_str];

    if cli.verbose {
        info!("Running: bb {}", verify_args.join(" "));
    }

    if cli.dry_run {
        println!("Would run: bb {}", verify_args.join(" "));
        return Ok(());
    }

    let timer = Timer::start();
    backends::bb::run(&verify_args).map_err(enhance_error_with_suggestions)?;

    if !cli.quiet {
        println!(
            "{}",
            success(&format!(
                "Proof verified successfully ({})",
                timer.elapsed()
            ))
        );
    }

    Ok(())
}

fn handle_solidity(cli: &Cli) -> Result<()> {
    let pkg_name = get_package_name(cli).map_err(enhance_error_with_suggestions)?;
    let mut summary = OperationSummary::new();

    // Validate that required build files exist
    let required_files = vec![util::get_bytecode_path(&pkg_name)];

    if !cli.dry_run {
        util::validate_files_exist(&required_files).map_err(enhance_error_with_suggestions)?;
    }

    // Generate VK with keccak oracle hash for Solidity optimization
    let bytecode_path = util::get_bytecode_path(&pkg_name);
    let bytecode_str = bytecode_path.to_string_lossy();
    let vk_args = vec![
        "write_vk",
        "--oracle_hash",
        "keccak",
        "-b",
        &bytecode_str,
        "-o",
        "./target/",
    ];

    if cli.verbose {
        info!("Running: bb {}", vk_args.join(" "));
    }

    if cli.dry_run {
        println!("Would run: bb {}", vk_args.join(" "));
        println!(
            "Would run: bb write_solidity_verifier -k ./target/vk -o ./contracts/Verifier.sol"
        );
        return Ok(());
    }

    let vk_timer = Timer::start();
    backends::bb::run(&vk_args).map_err(enhance_error_with_suggestions)?;

    if !cli.quiet {
        let vk_path = util::get_vk_path();
        println!(
            "{}",
            success(&format_operation_result(
                "VK (keccak optimized)",
                &vk_path,
                &vk_timer
            ))
        );
        summary.add_operation(&format!(
            "Verification key with keccak optimization ({})",
            util::format_file_size(&vk_path)
        ));
    }

    // Create contracts directory if it doesn't exist
    std::fs::create_dir_all("./contracts").map_err(|e| {
        create_smart_error(
            &format!("Failed to create contracts directory: {}", e),
            &[
                "Check directory permissions",
                "Ensure you have write access to the current directory",
            ],
        )
    })?;

    // Generate Solidity verifier
    let vk_path = util::get_vk_path();
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
            success(&format_operation_result(
                "Solidity verifier",
                &verifier_path,
                &solidity_timer
            ))
        );
        summary.add_operation(&format!(
            "Solidity verifier contract ({})",
            util::format_file_size(&verifier_path)
        ));
        summary.print();
    }

    Ok(())
}

fn handle_clean(cli: &Cli) -> Result<()> {
    if cli.verbose {
        info!("Removing target directory");
    }

    if cli.dry_run {
        println!("Would run: rm -rf target/");
        return Ok(());
    }

    if std::path::Path::new("target").exists() {
        std::fs::remove_dir_all("target")?;
        if !cli.quiet {
            println!("{}", success("Removed target/"));
        }
    } else {
        if !cli.quiet {
            println!("{}", info("target/ already clean"));
        }
    }

    Ok(())
}

fn build_nargo_args(cli: &Cli, base_args: &[&str]) -> Result<Vec<String>> {
    let mut args = base_args.iter().map(|s| s.to_string()).collect::<Vec<_>>();

    // Add package-specific args if needed
    if let Some(pkg) = &cli.pkg {
        args.push("--package".to_string());
        args.push(pkg.clone());
    }

    Ok(args)
}

fn handle_rebuild(cli: &Cli) -> Result<()> {
    let mut summary = OperationSummary::new();

    // Step 1: Clean
    if cli.verbose {
        info!("Step 1/2: Cleaning target directory");
    }

    if !cli.quiet {
        println!("🧹 Cleaning build artifacts...");
    }

    if cli.dry_run {
        println!("Would run: rm -rf target/");
    } else {
        if std::path::Path::new("target").exists() {
            std::fs::remove_dir_all("target")?;
            if !cli.quiet {
                println!("{}", success("Removed target/"));
            }
            summary.add_operation("Target directory cleaned");
        } else {
            if !cli.quiet {
                println!("{}", info("target/ already clean"));
            }
        }
    }

    // Step 2: Build
    if cli.verbose {
        info!("Step 2/2: Building from scratch");
    }

    if !cli.quiet {
        println!("\n🔨 Building circuit...");
    }

    let pkg_name = get_package_name(cli)?;
    let args = build_nargo_args(cli, &[])?;

    if cli.verbose {
        info!("Running: nargo execute {}", args.join(" "));
    }

    if cli.dry_run {
        println!("Would run: nargo execute {}", args.join(" "));
        return Ok(());
    }

    let timer = Timer::start();
    let result = backends::nargo::run(&["execute"]);

    match result {
        Ok(()) => {
            if !cli.quiet {
                let bytecode_path = util::get_bytecode_path(&pkg_name);
                let witness_path = util::get_witness_path(&pkg_name);

                println!(
                    "{}",
                    success(&format_operation_result(
                        "Bytecode generated",
                        &bytecode_path,
                        &timer
                    ))
                );

                // Create a new timer for witness (they're generated together but we show separate timing)
                let witness_timer = Timer::start();
                println!(
                    "{}",
                    success(&format_operation_result(
                        "Witness generated",
                        &witness_path,
                        &witness_timer
                    ))
                );

                summary.add_operation(&format!("Circuit rebuilt for {}", path(&pkg_name)));
                summary.add_operation(&format!(
                    "Bytecode generated ({})",
                    util::format_file_size(&bytecode_path)
                ));
                summary.add_operation(&format!(
                    "Witness generated ({})",
                    util::format_file_size(&witness_path)
                ));
                summary.print();
            }
            Ok(())
        }
        Err(e) => Err(enhance_error_with_suggestions(e)),
    }
}

fn get_package_name(cli: &Cli) -> Result<String> {
    util::get_package_name(cli.pkg.as_ref()).map_err(enhance_error_with_suggestions)
}
