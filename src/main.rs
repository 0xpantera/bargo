use clap::{Parser, Subcommand, ValueEnum};
use color_eyre::Result;
use tracing::{info, warn};

mod backends;
mod util;

use util::{
    Flavour, OperationSummary, Timer, create_smart_error, enhance_error_with_suggestions,
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

    /// Generate verifier contract
    #[command(about = "Generate Solidity verifier contract optimized for Ethereum deployment")]
    Verifier,

    /// Clean build artifacts
    #[command(about = "Remove target directory and all build artifacts")]
    Clean {
        /// Backend to clean (defaults to all)
        #[arg(long, value_enum)]
        backend: Option<Backend>,
    },

    /// Clean and rebuild (equivalent to clean + build)
    #[command(about = "Remove target directory and rebuild from scratch")]
    Rebuild {
        /// Backend to clean (defaults to all)
        #[arg(long, value_enum)]
        backend: Option<Backend>,
    },

    /// Cairo/Starknet operations
    #[command(about = "Generate Cairo verifiers and interact with Starknet")]
    Cairo {
        #[command(subcommand)]
        command: CairoCommands,
    },

    /// Check system dependencies
    #[command(about = "Verify that all required tools are installed and available")]
    Doctor,
}

#[derive(Subcommand)]
enum CairoCommands {
    /// Generate Cairo verifier contract
    #[command(about = "Generate Cairo verifier contract for Starknet deployment")]
    Gen,

    /// Generate calldata for proof verification
    #[command(about = "Generate calldata JSON for latest proof")]
    Data,

    /// Declare verifier contract on Starknet
    #[command(about = "Declare verifier contract on Starknet")]
    Declare,

    /// Deploy declared verifier contract
    #[command(about = "Deploy declared verifier contract")]
    Deploy {
        /// Class hash of the declared contract
        #[arg(long)]
        class_hash: Option<String>,
    },

    /// Verify proof on-chain
    #[command(about = "Verify proof on Starknet using deployed verifier")]
    VerifyOnchain {
        /// Address of deployed verifier contract
        #[arg(short = 'a', long)]
        address: String,
    },
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum Backend {
    /// Barretenberg backend (EVM/Solidity)
    Bb,
    /// Starknet backend (Cairo)
    Starknet,
    /// All backends
    All,
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
        Commands::Verifier => {
            if !cli.quiet {
                print_banner("verifier");
            }
            handle_verifier(&cli)?;
        }
        Commands::Clean { ref backend } => {
            if !cli.quiet {
                print_banner("clean");
            }
            handle_clean(&cli, backend.unwrap_or(Backend::All))?;
        }
        Commands::Rebuild { ref backend } => {
            if !cli.quiet {
                print_banner("rebuild");
            }
            handle_rebuild(&cli, backend.unwrap_or(Backend::All))?;
        }
        Commands::Cairo { ref command } => match command {
            CairoCommands::Gen => {
                if !cli.quiet {
                    print_banner("cairo gen");
                }
                handle_cairo_gen(&cli)?;
            }
            CairoCommands::Data => {
                if !cli.quiet {
                    print_banner("cairo data");
                }
                handle_cairo_data(&cli)?;
            }
            CairoCommands::Declare => {
                if !cli.quiet {
                    print_banner("cairo declare");
                }
                handle_cairo_declare(&cli)?;
            }
            CairoCommands::Deploy { class_hash } => {
                if !cli.quiet {
                    print_banner("cairo deploy");
                }
                handle_cairo_deploy(&cli, class_hash.as_deref())?;
            }
            CairoCommands::VerifyOnchain { address } => {
                if !cli.quiet {
                    print_banner("cairo verify-onchain");
                }
                handle_cairo_verify_onchain(&cli, &address)?;
            }
        },
        Commands::Doctor => {
            if !cli.quiet {
                print_banner("doctor");
            }
            handle_doctor(&cli)?;
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
            // Organize build artifacts into flavour-specific directories
            util::organize_build_artifacts(&pkg_name, Flavour::Bb)?;

            if !cli.quiet {
                let bytecode_path = util::get_bytecode_path(&pkg_name, Flavour::Bb);
                let witness_path = util::get_witness_path(&pkg_name, Flavour::Bb);

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
        util::get_bytecode_path(&pkg_name, Flavour::Bb),
        util::get_witness_path(&pkg_name, Flavour::Bb),
    ];

    if !cli.dry_run {
        util::validate_files_exist(&required_files).map_err(enhance_error_with_suggestions)?;
    }

    // Build bb prove arguments
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

    // Create target/bb directory if it doesn't exist
    std::fs::create_dir_all("./target/bb").map_err(|e| {
        create_smart_error(
            &format!("Failed to create target/bb directory: {}", e),
            &[
                "Check directory permissions",
                "Ensure you have write access to the current directory",
            ],
        )
    })?;

    // Run bb prove
    let prove_timer = Timer::start();
    backends::bb::run(&prove_args).map_err(enhance_error_with_suggestions)?;

    if !cli.quiet {
        let proof_path = util::get_proof_path(Flavour::Bb);
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

    // Verify proof unless skipped
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
    let required_files = vec![
        util::get_proof_path(Flavour::Bb),
        util::get_vk_path(Flavour::Bb),
        util::get_public_inputs_path(Flavour::Bb),
    ];

    if !cli.dry_run {
        util::validate_files_exist(&required_files).map_err(enhance_error_with_suggestions)?;
    }

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

fn handle_verifier(cli: &Cli) -> Result<()> {
    let pkg_name = get_package_name(cli).map_err(enhance_error_with_suggestions)?;
    let mut summary = OperationSummary::new();

    // Validate that required build files exist
    let required_files = vec![util::get_bytecode_path(&pkg_name, Flavour::Bb)];

    if !cli.dry_run {
        util::validate_files_exist(&required_files).map_err(enhance_error_with_suggestions)?;
    }

    // Generate VK with keccak oracle hash for Solidity optimization
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
        println!(
            "Would run: bb write_solidity_verifier -k ./target/bb/vk -o ./contracts/Verifier.sol"
        );
        return Ok(());
    }

    // Create target/bb directory if it doesn't exist
    std::fs::create_dir_all("./target/bb").map_err(|e| {
        create_smart_error(
            &format!("Failed to create target/bb directory: {}", e),
            &[
                "Check directory permissions",
                "Ensure you have write access to the current directory",
            ],
        )
    })?;

    let vk_timer = Timer::start();
    backends::bb::run(&vk_args).map_err(enhance_error_with_suggestions)?;

    if !cli.quiet {
        let vk_path = util::get_vk_path(Flavour::Bb);
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

fn handle_clean(cli: &Cli, backend: Backend) -> Result<()> {
    if cli.verbose {
        info!("Cleaning artifacts for backend: {:?}", backend);
    }

    match backend {
        Backend::All => {
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
        }
        Backend::Bb => {
            if cli.dry_run {
                println!("Would run: rm -rf target/bb/");
                return Ok(());
            }

            if std::path::Path::new("target/bb").exists() {
                std::fs::remove_dir_all("target/bb")?;
                if !cli.quiet {
                    println!("{}", success("Removed target/bb/"));
                }
            } else {
                if !cli.quiet {
                    println!("{}", info("target/bb/ already clean"));
                }
            }
        }
        Backend::Starknet => {
            if cli.dry_run {
                println!("Would run: rm -rf target/starknet/");
                return Ok(());
            }

            if std::path::Path::new("target/starknet").exists() {
                std::fs::remove_dir_all("target/starknet")?;
                if !cli.quiet {
                    println!("{}", success("Removed target/starknet/"));
                }
            } else {
                if !cli.quiet {
                    println!("{}", info("target/starknet/ already clean"));
                }
            }
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

fn handle_rebuild(cli: &Cli, backend: Backend) -> Result<()> {
    let mut summary = OperationSummary::new();

    // Step 1: Clean
    if cli.verbose {
        info!("Step 1/2: Cleaning artifacts for backend: {:?}", backend);
    }

    if !cli.quiet {
        println!("🧹 Cleaning build artifacts...");
    }

    handle_clean(cli, backend)?;
    if backend != Backend::Starknet {
        summary.add_operation("Build artifacts cleaned");
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
            // Organize build artifacts into flavour-specific directories
            util::organize_build_artifacts(&pkg_name, Flavour::Bb)?;

            if !cli.quiet {
                let bytecode_path = util::get_bytecode_path(&pkg_name, Flavour::Bb);
                let witness_path = util::get_witness_path(&pkg_name, Flavour::Bb);

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

fn handle_cairo_gen(cli: &Cli) -> Result<()> {
    let pkg_name = get_package_name(cli)?;
    let mut summary = OperationSummary::new();

    // Validate that required build files exist (from regular build)
    let required_files = vec![
        util::get_bytecode_path(&pkg_name, Flavour::Bb),
        util::get_witness_path(&pkg_name, Flavour::Bb),
    ];

    if !cli.dry_run {
        util::validate_files_exist(&required_files).map_err(enhance_error_with_suggestions)?;
    }

    // Create target/starknet directory if it doesn't exist
    std::fs::create_dir_all("./target/starknet").map_err(|e| {
        create_smart_error(
            &format!("Failed to create target/starknet directory: {}", e),
            &[
                "Check directory permissions",
                "Ensure you have write access to the current directory",
            ],
        )
    })?;

    // Generate keccak-flavoured proof in target/starknet/
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
        "./target/starknet/",
        "--oracle_hash",
        "keccak",
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
            "keccak",
        ];
        println!("Would run: bb {}", vk_args.join(" "));
        println!(
            "Would run: garaga gen --system ultra_keccak_honk --vk target/starknet/vk -o contracts/Verifier.cairo"
        );
        return Ok(());
    }

    // Run bb prove with keccak oracle
    let prove_timer = Timer::start();
    backends::bb::run(&prove_args).map_err(enhance_error_with_suggestions)?;

    if !cli.quiet {
        let proof_path = util::get_proof_path(Flavour::Starknet);
        println!(
            "{}",
            success(&format_operation_result(
                "Keccak proof generated",
                &proof_path,
                &prove_timer
            ))
        );
        summary.add_operation(&format!(
            "Keccak proof generated ({})",
            util::format_file_size(&proof_path)
        ));
    }

    // Generate keccak-flavoured VK in target/starknet/
    let vk_args = vec![
        "write_vk",
        "-b",
        &bytecode_str,
        "-o",
        "./target/starknet/",
        "--oracle_hash",
        "keccak",
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
            success(&format_operation_result(
                "Keccak VK generated",
                &vk_path,
                &vk_timer
            ))
        );
        summary.add_operation(&format!(
            "Keccak verification key ({})",
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

    // Generate Cairo verifier using garaga
    let vk_path = util::get_vk_path(Flavour::Starknet);
    let vk_str = vk_path.to_string_lossy();
    let garaga_args = vec![
        "gen",
        "--system",
        "ultra_keccak_honk",
        "--vk",
        &vk_str,
        "--project-name",
        &pkg_name,
    ];

    if cli.verbose {
        info!("Running: garaga {}", garaga_args.join(" "));
    }

    let garaga_timer = Timer::start();
    backends::garaga::run(&garaga_args).map_err(enhance_error_with_suggestions)?;

    if !cli.quiet {
        let cairo_verifier_path = std::path::PathBuf::from("./contracts/Verifier.cairo");
        println!(
            "{}",
            success(&format_operation_result(
                "Cairo verifier generated",
                &cairo_verifier_path,
                &garaga_timer
            ))
        );
        summary.add_operation(&format!(
            "Cairo verifier contract ({})",
            util::format_file_size(&cairo_verifier_path)
        ));
        summary.print();
    }

    Ok(())
}

fn handle_cairo_data(cli: &Cli) -> Result<()> {
    let mut summary = OperationSummary::new();

    // Validate that required Starknet artifacts exist
    let required_files = vec![
        util::get_proof_path(Flavour::Starknet),
        util::get_vk_path(Flavour::Starknet),
        util::get_public_inputs_path(Flavour::Starknet),
    ];

    if !cli.dry_run {
        util::validate_files_exist(&required_files).map_err(enhance_error_with_suggestions)?;
    }

    // Generate calldata using garaga
    let proof_path = util::get_proof_path(Flavour::Starknet);
    let vk_path = util::get_vk_path(Flavour::Starknet);
    let public_inputs_path = util::get_public_inputs_path(Flavour::Starknet);
    let proof_str = proof_path.to_string_lossy();
    let vk_str = vk_path.to_string_lossy();
    let public_inputs_str = public_inputs_path.to_string_lossy();

    let garaga_args = vec![
        "calldata",
        "--system",
        "ultra_keccak_honk",
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
    backends::garaga::run(&garaga_args).map_err(enhance_error_with_suggestions)?;

    if !cli.quiet {
        println!(
            "{}",
            success(&format!(
                "Calldata generated successfully ({})",
                calldata_timer.elapsed()
            ))
        );
        summary.add_operation("Calldata JSON generated for proof verification");
        summary.print();
    }

    Ok(())
}

fn handle_cairo_declare(cli: &Cli) -> Result<()> {
    let mut summary = OperationSummary::new();

    // Validate that Cairo verifier contract exists
    let verifier_contract_path = std::path::PathBuf::from("./contracts/Verifier.cairo");
    if !cli.dry_run {
        util::validate_files_exist(&[verifier_contract_path.clone()])
            .map_err(enhance_error_with_suggestions)?;
    }

    // Declare Cairo verifier contract using garaga
    let contract_str = verifier_contract_path.to_string_lossy();
    let garaga_args = vec!["declare", "--contract", &contract_str];

    if cli.verbose {
        info!("Running: garaga {}", garaga_args.join(" "));
    }

    if cli.dry_run {
        println!("Would run: garaga {}", garaga_args.join(" "));
        return Ok(());
    }

    let declare_timer = Timer::start();
    backends::garaga::run(&garaga_args).map_err(enhance_error_with_suggestions)?;

    if !cli.quiet {
        println!(
            "{}",
            success(&format!(
                "Contract declared successfully ({})",
                declare_timer.elapsed()
            ))
        );
        summary.add_operation("Cairo verifier contract declared on Starknet");
        summary.print();
    }

    Ok(())
}

fn handle_cairo_deploy(cli: &Cli, class_hash: Option<&str>) -> Result<()> {
    let mut summary = OperationSummary::new();

    // Validate that class hash is provided
    let hash = class_hash.ok_or_else(|| {
        create_smart_error(
            "Class hash is required for deployment",
            &[
                "Provide a class hash with --class-hash <HASH>",
                "Get the class hash from the declare command output",
                "Use 'bargo cairo declare' to declare the contract first",
            ],
        )
    })?;

    // Deploy Cairo verifier contract using garaga
    let garaga_args = vec!["deploy", "--class-hash", hash];

    if cli.verbose {
        info!("Running: garaga {}", garaga_args.join(" "));
        info!("Deploying contract with class hash: {}", hash);
    }

    if cli.dry_run {
        println!("Would run: garaga {}", garaga_args.join(" "));
        return Ok(());
    }

    let deploy_timer = Timer::start();
    backends::garaga::run(&garaga_args).map_err(enhance_error_with_suggestions)?;

    if !cli.quiet {
        println!(
            "{}",
            success(&format!(
                "Contract deployed successfully ({})",
                deploy_timer.elapsed()
            ))
        );
        summary.add_operation(&format!(
            "Cairo verifier deployed with class hash: {}",
            hash
        ));
        summary.print();
    }

    Ok(())
}

fn handle_cairo_verify_onchain(cli: &Cli, address: &str) -> Result<()> {
    let mut summary = OperationSummary::new();

    // Validate that required Starknet artifacts exist
    let required_files = vec![
        util::get_proof_path(Flavour::Starknet),
        util::get_vk_path(Flavour::Starknet),
        util::get_public_inputs_path(Flavour::Starknet),
    ];

    if !cli.dry_run {
        util::validate_files_exist(&required_files).map_err(enhance_error_with_suggestions)?;
    }

    // Verify proof on-chain using garaga
    let proof_path = util::get_proof_path(Flavour::Starknet);
    let vk_path = util::get_vk_path(Flavour::Starknet);
    let public_inputs_path = util::get_public_inputs_path(Flavour::Starknet);
    let proof_str = proof_path.to_string_lossy();
    let vk_str = vk_path.to_string_lossy();
    let public_inputs_str = public_inputs_path.to_string_lossy();

    let garaga_args = vec![
        "verify-onchain",
        "--address",
        address,
        "--vk",
        &vk_str,
        "--proof",
        &proof_str,
        "--public-inputs",
        &public_inputs_str,
    ];

    if cli.verbose {
        info!("Running: garaga {}", garaga_args.join(" "));
        info!("Verifying proof on-chain at address: {}", address);
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
            success(&format!(
                "Proof verified on-chain successfully ({})",
                verify_timer.elapsed()
            ))
        );
        summary.add_operation(&format!(
            "On-chain verification completed at address: {}",
            address
        ));
        summary.print();
    }

    Ok(())
}

fn handle_doctor(cli: &Cli) -> Result<()> {
    if !cli.quiet {
        println!("🔍 Checking system dependencies...\n");
    }

    let mut all_good = true;

    // Check nargo
    match which::which("nargo") {
        Ok(path) => {
            if !cli.quiet {
                println!("✅ nargo: {}", path.display());
            }
        }
        Err(_) => {
            if !cli.quiet {
                println!("❌ nargo: not found");
                println!(
                    "   Install from: https://noir-lang.org/docs/getting_started/installation/"
                );
            }
            all_good = false;
        }
    }

    // Check bb
    match which::which("bb") {
        Ok(path) => {
            if !cli.quiet {
                println!("✅ bb: {}", path.display());
            }
        }
        Err(_) => {
            if !cli.quiet {
                println!("❌ bb: not found");
                println!("   Install from: https://github.com/AztecProtocol/aztec-packages");
            }
            all_good = false;
        }
    }

    // Check garaga (optional for Cairo features)
    match which::which("garaga") {
        Ok(path) => {
            if !cli.quiet {
                println!("✅ garaga: {}", path.display());
            }
        }
        Err(_) => {
            if !cli.quiet {
                println!("⚠️  garaga: not found (optional - needed for Cairo features)");
                println!("   Install with: pipx install garaga");
                println!("   Requires Python 3.10+");
            }
        }
    }

    if !cli.quiet {
        println!();
        if all_good {
            println!("🎉 All required dependencies are available!");
            println!("   You can use all bargo features.");
        } else {
            println!("🚨 Some required dependencies are missing.");
            println!("   EVM features require: nargo + bb");
            println!("   Cairo features also require: garaga");
        }
    }

    if !all_good {
        std::process::exit(1);
    }

    Ok(())
}

fn get_package_name(cli: &Cli) -> Result<String> {
    util::get_package_name(cli.pkg.as_ref()).map_err(enhance_error_with_suggestions)
}
