use clap::{Parser, Subcommand, ValueEnum};
use color_eyre::Result;
use tracing::{info, warn};

mod backends;
mod util;
mod commands;

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

    /// EVM/Foundry operations
    #[command(about = "Generate Solidity verifiers and interact with EVM networks")]
    Evm {
        #[command(subcommand)]
        command: EvmCommands,
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
    Declare {
        /// Network to declare on (sepolia or mainnet)
        #[arg(long, default_value = "sepolia")]
        network: String,
    },

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
        address: Option<String>,
    },
}

#[derive(Subcommand)]
enum EvmCommands {
    /// Generate Solidity verifier contract and Foundry project
    #[command(about = "Generate Solidity verifier contract with complete Foundry project setup")]
    Gen,

    /// Deploy verifier contract to EVM network
    #[command(about = "Deploy verifier contract using Foundry")]
    Deploy {
        /// Network to deploy to (mainnet or sepolia)
        #[arg(long, default_value = "sepolia")]
        network: String,
    },

    /// Generate calldata for proof verification
    #[command(about = "Generate calldata for proof verification using cast")]
    Calldata,

    /// Verify proof on-chain
    #[command(about = "Verify proof on EVM network using deployed verifier")]
    VerifyOnchain,
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

    // Load .env file if present (for EVM environment variables)
    dotenv::dotenv().ok(); // .ok() means don't fail if .env doesn't exist

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
            handle_clean(&cli, (*backend).unwrap_or(Backend::All))?;
        }
        Commands::Rebuild { ref backend } => {
            if !cli.quiet {
                print_banner("rebuild");
            }
            handle_rebuild(&cli, (*backend).unwrap_or(Backend::All))?;
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
            CairoCommands::Declare { network } => {
                if !cli.quiet {
                    print_banner("cairo declare");
                }
                handle_cairo_declare(&cli, network)?;
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
                handle_cairo_verify_onchain(&cli, address.as_deref())?;
            }
        },
        Commands::Evm { ref command } => match command {
            EvmCommands::Gen => {
                if !cli.quiet {
                    print_banner("evm gen");
                }
                handle_evm_gen(&cli)?;
            }
            EvmCommands::Deploy { network } => {
                if !cli.quiet {
                    print_banner("evm deploy");
                }
                handle_evm_deploy(&cli, network)?;
            }
            EvmCommands::Calldata => {
                if !cli.quiet {
                    print_banner("evm calldata");
                }
                handle_evm_calldata(&cli)?;
            }
            EvmCommands::VerifyOnchain => {
                if !cli.quiet {
                    print_banner("evm verify-onchain");
                }
                handle_evm_verify_onchain(&cli)?;
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
    commands::build::run(cli).map_err(enhance_error_with_suggestions)
}

fn handle_prove(cli: &Cli, skip_verify: bool) -> Result<()> {
    commands::prove::run(cli, skip_verify).map_err(enhance_error_with_suggestions)
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
    commands::cairo::run_gen(cli).map_err(enhance_error_with_suggestions)
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
    let (stdout, _stderr) =
        backends::garaga::run_with_output(&garaga_args).map_err(enhance_error_with_suggestions)?;

    // Save calldata to target/starknet/calldata.json
    let calldata_path = std::path::PathBuf::from("./target/starknet/calldata.json");
    std::fs::write(&calldata_path, stdout.trim()).map_err(|e| {
        create_smart_error(
            &format!("Failed to write calldata file: {}", e),
            &[
                "Check directory permissions",
                "Ensure target/starknet directory exists",
            ],
        )
    })?;

    if !cli.quiet {
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

fn handle_cairo_declare(cli: &Cli, network: &str) -> Result<()> {
    let _pkg_name = get_package_name(cli)?;
    let mut summary = OperationSummary::new();

    // Load environment variables from .secrets file
    if std::path::Path::new(".secrets").exists() {
        if let Err(e) = dotenv::from_filename(".secrets") {
            if cli.verbose {
                eprintln!("Warning: Failed to load .secrets file: {}", e);
            }
        }
    }

    // Validate that Cairo project directory exists (created by bargo cairo gen)
    let cairo_project_path = std::path::PathBuf::from("./contracts/cairo");
    let scarb_toml_path = cairo_project_path.join("Scarb.toml");
    if !cli.dry_run {
        util::validate_files_exist(&[scarb_toml_path]).map_err(enhance_error_with_suggestions)?;
    }

    // Set up environment variables for sncast
    let rpc_url = if network == "mainnet" {
        std::env::var("MAINNET_RPC_URL").map_err(|_| {
            create_smart_error(
                "MAINNET_RPC_URL environment variable not found",
                &[
                    "Add to your .secrets file: MAINNET_RPC_URL=https://starknet-mainnet.g.alchemy.com/starknet/version/rpc/v0_8/your_key",
                    "Ensure the .secrets file is loaded in your environment",
                ],
            )
        })?
    } else {
        std::env::var("SEPOLIA_RPC_URL").map_err(|_| {
            create_smart_error(
                "SEPOLIA_RPC_URL environment variable not found",
                &[
                    "Add to your .secrets file: SEPOLIA_RPC_URL=https://starknet-sepolia.g.alchemy.com/starknet/version/rpc/v0_8/your_key",
                    "Ensure the .secrets file is loaded in your environment",
                ],
            )
        })?
    };

    // Get account and keystore paths from environment variables
    let account_path = std::env::var("STARKNET_ACCOUNT").map_err(|_| {
        create_smart_error(
            "STARKNET_ACCOUNT environment variable not found",
            &[
                "Add to your .secrets file: STARKNET_ACCOUNT=path/to/account.json",
                "This should point to your starkli account configuration file",
            ],
        )
    })?;

    let keystore_path = std::env::var("STARKNET_KEYSTORE").map_err(|_| {
        create_smart_error(
            "STARKNET_KEYSTORE environment variable not found",
            &[
                "Add to your .secrets file: STARKNET_KEYSTORE=path/to/keystore.json",
                "This should point to your starkli keystore file",
            ],
        )
    })?;

    // Declare Cairo verifier contract using starkli
    let compiled_contract_path = format!(
        "./contracts/cairo/target/dev/cairo_UltraStarknetZKHonkVerifier.contract_class.json"
    );

    let starkli_args = vec![
        "declare",
        &compiled_contract_path,
        "--rpc",
        &rpc_url,
        "--account",
        &account_path,
        "--keystore",
        &keystore_path,
        "--casm-file",
        "./contracts/cairo/target/dev/cairo_UltraStarknetZKHonkVerifier.compiled_contract_class.json",
    ];

    if cli.verbose {
        info!("Running: starkli {}", starkli_args.join(" "));
    }

    if cli.dry_run {
        println!("Would run: starkli {}", starkli_args.join(" "));
        return Ok(());
    }

    let declare_timer = Timer::start();
    let starkli_output = std::process::Command::new("starkli")
        .args(&starkli_args)
        .output()
        .map_err(|e| {
            create_smart_error(
                &format!("Failed to run starkli declare: {}", e),
                &[
                    "Ensure starkli is installed and available in PATH",
                    "Install with: curl -L https://get.starkli.sh | sh",
                    "Check that the Cairo project was built correctly",
                ],
            )
        })?;

    if !starkli_output.status.success() {
        let stderr = String::from_utf8_lossy(&starkli_output.stderr);
        let stdout = String::from_utf8_lossy(&starkli_output.stdout);
        return Err(create_smart_error(
            &format!("Starkli declare failed: {}{}", stdout, stderr),
            &[
                "Check your account balance and network connectivity",
                "Verify the contract compilation was successful",
                "Ensure your private key and account address are correct",
            ],
        ));
    }

    let stdout = String::from_utf8_lossy(&starkli_output.stdout);

    // Extract class hash from sncast output
    let mut class_hash_output = String::new();

    // Parse class hash from starkli declare output
    // starkli outputs the class hash directly as a single line
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("0x") && trimmed.len() >= 60 {
            class_hash_output = trimmed.to_string();
            break;
        }
    }

    if class_hash_output.is_empty() {
        return Err(create_smart_error(
            "Could not extract class hash from starkli output",
            &[
                "The contract declaration may have failed",
                "Check the starkli output above for details",
                "Try running the command again",
            ],
        ));
    }

    if !cli.quiet {
        println!("{}", success(&format!("Class hash: {}", class_hash_output)));
        // Generate voyager link
        let voyager_link = format!("https://voyager.online/class/{}", class_hash_output);
        println!("🔗 View on Voyager: {}", voyager_link);
    }

    // Save class hash to file for subsequent deploy command
    let class_hash_file = std::path::Path::new("target/starknet/.bargo_class_hash");
    if let Some(parent) = class_hash_file.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            if cli.verbose {
                eprintln!("Warning: Failed to create target directory: {}", e);
            }
        }
    }
    if let Err(e) = std::fs::write(class_hash_file, &class_hash_output) {
        if cli.verbose {
            eprintln!("Warning: Failed to save class hash to file: {}", e);
        }
    }

    if !cli.quiet {
        println!(
            "{}",
            success(&format!(
                "Contract declared successfully ({})",
                declare_timer.elapsed()
            ))
        );
        let mut operation = "Cairo verifier contract declared on Starknet".to_string();
        if !class_hash_output.is_empty() {
            operation = format!(
                "Cairo verifier contract declared on Starknet (class hash: {})",
                class_hash_output
            );
        }
        summary.add_operation(&operation);
        summary.print();
    }

    Ok(())
}

fn handle_cairo_deploy(cli: &Cli, class_hash: Option<&str>) -> Result<()> {
    let mut summary = OperationSummary::new();

    // Get class hash from parameter or from saved file
    let hash = match class_hash {
        Some(hash) => hash.to_string(),
        None => {
            // Try to read class hash from file saved by declare command
            match std::fs::read_to_string("target/starknet/.bargo_class_hash") {
                Ok(saved_hash) => {
                    let hash = saved_hash.trim().to_string();
                    if !cli.quiet {
                        println!("{}", success(&format!("Using saved class hash: {}", hash)));
                    }
                    hash
                }
                Err(_) => {
                    return Err(create_smart_error(
                        "Class hash is required for deployment",
                        &[
                            "Provide a class hash with --class-hash <HASH>",
                            "Or run 'bargo cairo declare' first to save the class hash",
                            "The declare command will save the class hash for deployment",
                        ],
                    ));
                }
            }
        }
    };

    // Load environment variables from .secrets file
    if std::path::Path::new(".secrets").exists() {
        if let Err(e) = dotenv::from_filename(".secrets") {
            if cli.verbose {
                eprintln!("Warning: Failed to load .secrets file: {}", e);
            }
        }
    }

    // Get RPC URL (default to mainnet)
    let rpc_url = std::env::var("MAINNET_RPC_URL").map_err(|_| {
        create_smart_error(
            "MAINNET_RPC_URL environment variable not found",
            &[
                "Add to your .secrets file: MAINNET_RPC_URL=https://starknet-mainnet.g.alchemy.com/starknet/version/rpc/v0_8/your_key",
                "Ensure the .secrets file is loaded in your environment",
            ],
        )
    })?;

    // Get account and keystore paths from environment variables
    let account_path = std::env::var("STARKNET_ACCOUNT").map_err(|_| {
        create_smart_error(
            "STARKNET_ACCOUNT environment variable not found",
            &[
                "Add to your .secrets file: STARKNET_ACCOUNT=path/to/account.json",
                "This should point to your starkli account configuration file",
            ],
        )
    })?;

    let keystore_path = std::env::var("STARKNET_KEYSTORE").map_err(|_| {
        create_smart_error(
            "STARKNET_KEYSTORE environment variable not found",
            &[
                "Add to your .secrets file: STARKNET_KEYSTORE=path/to/keystore.json",
                "This should point to your starkli keystore file",
            ],
        )
    })?;

    // Deploy Cairo verifier contract using starkli
    let starkli_args = vec![
        "deploy",
        &hash,
        "--rpc",
        &rpc_url,
        "--account",
        &account_path,
        "--keystore",
        &keystore_path,
    ];

    if cli.verbose {
        info!("Running: starkli {}", starkli_args.join(" "));
    }

    if cli.dry_run {
        println!("Would run: starkli {}", starkli_args.join(" "));
        return Ok(());
    }

    let deploy_timer = Timer::start();
    let starkli_output = std::process::Command::new("starkli")
        .args(&starkli_args)
        .output()
        .map_err(|e| {
            create_smart_error(
                &format!("Failed to run starkli deploy: {}", e),
                &["Ensure starkli is installed and available in PATH"],
            )
        })?;

    if !starkli_output.status.success() {
        let stderr = String::from_utf8_lossy(&starkli_output.stderr);
        let stdout = String::from_utf8_lossy(&starkli_output.stdout);
        return Err(create_smart_error(
            &format!("Starkli deploy failed: {}{}", stdout, stderr),
            &["Check your account balance and network connectivity"],
        ));
    }

    let output = String::from_utf8_lossy(&starkli_output.stdout);

    // Parse contract address from starkli output
    let mut contract_address = String::new();
    for line in output.lines() {
        if let Some(start) = line.find("0x") {
            let addr = &line[start..];
            if addr.len() >= 66 {
                contract_address = addr.split_whitespace().next().unwrap_or("").to_string();
                break;
            }
        }
    }

    // Save contract address to file for subsequent verify-onchain command
    if !contract_address.is_empty() {
        let address_file = std::path::Path::new("target/starknet/.bargo_contract_address");
        if let Some(parent) = address_file.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                if cli.verbose {
                    eprintln!("Warning: Failed to create target directory: {}", e);
                }
            }
        }
        if let Err(e) = std::fs::write(address_file, &contract_address) {
            if cli.verbose {
                eprintln!("Warning: Failed to save contract address to file: {}", e);
            }
        } else if !cli.quiet {
            println!(
                "{}",
                success(&format!("Contract address: {}", contract_address))
            );
        }
    }

    if !cli.quiet {
        println!(
            "{}",
            success(&format!(
                "Contract deployed successfully ({})",
                deploy_timer.elapsed()
            ))
        );
        let mut operation = format!("Cairo verifier deployed with class hash: {}", hash);
        if !contract_address.is_empty() {
            operation = format!("Cairo verifier deployed at address: {}", contract_address);
        }
        summary.add_operation(&operation);
        summary.print();
    }

    Ok(())
}

fn handle_cairo_verify_onchain(cli: &Cli, address: Option<&str>) -> Result<()> {
    let mut summary = OperationSummary::new();

    // Get contract address from parameter or from saved file
    let contract_address = match address {
        Some(addr) => addr.to_string(),
        None => {
            // Try to read contract address from file saved by deploy command
            match std::fs::read_to_string("target/starknet/.bargo_contract_address") {
                Ok(saved_address) => {
                    let addr = saved_address.trim().to_string();
                    if !cli.quiet {
                        println!(
                            "{}",
                            success(&format!("Using saved contract address: {}", addr))
                        );
                    }
                    addr
                }
                Err(_) => {
                    return Err(create_smart_error(
                        "Contract address is required for verification",
                        &[
                            "Provide a contract address with --address <ADDRESS>",
                            "Or run 'bargo cairo deploy' first to save the contract address",
                            "The deploy command will save the contract address for verification",
                        ],
                    ));
                }
            }
        }
    };

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
            success(&format!(
                "Proof verified on-chain successfully ({})",
                verify_timer.elapsed()
            ))
        );
        summary.add_operation(&format!(
            "On-chain verification completed at address: {}",
            contract_address
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

    // Check forge (optional for EVM features)
    match which::which("forge") {
        Ok(path) => {
            if !cli.quiet {
                println!("✅ forge: {}", path.display());
            }
        }
        Err(_) => {
            if !cli.quiet {
                println!("⚠️  forge: not found (optional - needed for EVM features)");
                println!("   Install with: curl -L https://foundry.paradigm.xyz | bash");
                println!("   Then run: foundryup");
            }
        }
    }

    // Check cast (optional for EVM features)
    match which::which("cast") {
        Ok(path) => {
            if !cli.quiet {
                println!("✅ cast: {}", path.display());
            }
        }
        Err(_) => {
            if !cli.quiet {
                println!("⚠️  cast: not found (optional - needed for EVM features)");
                println!("   Install with: curl -L https://foundry.paradigm.xyz | bash");
                println!("   Then run: foundryup");
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
            println!("   Core features require: nargo + bb");
            println!("   EVM deployment features also require: forge + cast");
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

/// Handle `evm gen` command - Generate Solidity verifier contract and Foundry project
fn handle_evm_gen(cli: &Cli) -> Result<()> {
    commands::evm::r#gen::run(cli).map_err(enhance_error_with_suggestions)
}
/// Handle `evm deploy` command - Deploy verifier contract to EVM network
fn handle_evm_deploy(cli: &Cli, network: &str) -> Result<()> {
    let mut summary = OperationSummary::new();

    // Ensure Foundry is available
    backends::foundry::ensure_available().map_err(enhance_error_with_suggestions)?;

    // Check environment variables
    let rpc_url = std::env::var("RPC_URL").map_err(|_| {
        create_smart_error(
            "RPC_URL environment variable not found",
            &[
                "Create a .env file in your project root",
                "Add: RPC_URL=https://your-rpc-endpoint.com",
                "For Sepolia: RPC_URL=https://eth-sepolia.g.alchemy.com/v2/your-key",
                "For Mainnet: RPC_URL=https://eth-mainnet.g.alchemy.com/v2/your-key",
            ],
        )
    })?;

    let private_key = std::env::var("PRIVATE_KEY").map_err(|_| {
        create_smart_error(
            "PRIVATE_KEY environment variable not found",
            &[
                "Add to your .env file: PRIVATE_KEY=0x...",
                "Make sure your private key starts with 0x",
                "Never commit your .env file to version control",
            ],
        )
    })?;

    // Check that Verifier.sol exists
    let verifier_path = std::path::PathBuf::from("./contracts/evm/src/Verifier.sol");
    if !verifier_path.exists() {
        return Err(create_smart_error(
            "Verifier.sol not found",
            &[
                "Run 'bargo evm gen' first to generate the Solidity verifier",
                "Ensure the Foundry project was created successfully",
            ],
        ));
    }

    if !cli.quiet {
        println!("🚀 Deploying Verifier contract to {}...", network);
    }

    // Deploy using forge create
    let deploy_args = vec![
        "create",
        "--rpc-url",
        &rpc_url,
        "--private-key",
        &private_key,
        "src/Verifier.sol:HonkVerifier",
        "--root",
        "contracts/evm",
    ];

    if cli.verbose {
        info!("Running: forge {}", deploy_args.join(" "));
    }

    if cli.dry_run {
        println!("Would run: forge {}", deploy_args.join(" "));
        return Ok(());
    }

    let deploy_timer = Timer::start();
    let (stdout, _stderr) = backends::foundry::run_forge_with_output(&deploy_args)
        .map_err(enhance_error_with_suggestions)?;

    // Parse deployment output to extract contract address
    let contract_address = stdout
        .lines()
        .find(|line| line.contains("Deployed to:"))
        .and_then(|line| line.split("Deployed to:").nth(1))
        .map(|addr| addr.trim())
        .ok_or_else(|| {
            create_smart_error(
                "Failed to parse contract address from forge output",
                &[
                    "Check that the deployment was successful",
                    "Verify your RPC_URL and PRIVATE_KEY are correct",
                    "Ensure you have sufficient funds for deployment",
                ],
            )
        })?;

    if !cli.quiet {
        println!(
            "{}",
            success(&format!(
                "Contract deployed successfully ({})",
                deploy_timer.elapsed()
            ))
        );
        println!("📍 Contract address: {}", contract_address);
        summary.add_operation(&format!("Contract deployed to {}", contract_address));
    }

    // Save contract address for future commands
    std::fs::create_dir_all("./target/bb").map_err(|e| {
        create_smart_error(
            &format!("Failed to create target/bb directory: {}", e),
            &["Check directory permissions"],
        )
    })?;

    std::fs::write("./target/bb/.bargo_contract_address", contract_address).map_err(|e| {
        create_smart_error(
            &format!("Failed to save contract address: {}", e),
            &["Check directory permissions"],
        )
    })?;

    if !cli.quiet {
        summary.print();
        println!();
        println!("🎯 Next steps:");
        println!("  1. Generate calldata: bargo evm calldata");
        println!("  2. Verify proof on-chain: bargo evm verify-onchain");
    }

    Ok(())
}

/// Handle `evm calldata` command - Generate calldata for proof verification
fn handle_evm_calldata(cli: &Cli) -> Result<()> {
    let mut summary = OperationSummary::new();

    // Ensure Foundry is available
    backends::foundry::ensure_available().map_err(enhance_error_with_suggestions)?;

    // Check that proof fields JSON exists
    let proof_fields_path = std::path::PathBuf::from("./target/bb/proof_fields.json");
    if !proof_fields_path.exists() {
        return Err(create_smart_error(
            "Proof fields file not found",
            &[
                "Run 'bargo evm gen' first to generate field-formatted proof",
                "This command requires the proof in JSON field format",
            ],
        ));
    }

    // Check that public inputs fields JSON exists
    let public_inputs_fields_path =
        std::path::PathBuf::from("./target/bb/public_inputs_fields.json");
    if !public_inputs_fields_path.exists() {
        return Err(create_smart_error(
            "Public inputs fields file not found",
            &[
                "Public inputs fields should be generated during proving",
                "Run 'bargo evm gen' to regenerate with field format",
            ],
        ));
    }

    if !cli.quiet {
        println!("📝 Generating calldata for proof verification...");
    }

    // Read proof fields JSON
    let proof_fields_content = std::fs::read_to_string(&proof_fields_path).map_err(|e| {
        create_smart_error(
            &format!("Failed to read proof fields file: {}", e),
            &["Check file permissions and try regenerating the proof"],
        )
    })?;

    // Read public inputs fields JSON
    let public_inputs_fields_content = std::fs::read_to_string(&public_inputs_fields_path)
        .map_err(|e| {
            create_smart_error(
                &format!("Failed to read public inputs fields file: {}", e),
                &["Check file permissions and try regenerating the proof"],
            )
        })?;

    // Parse JSON arrays
    let proof_fields: Vec<String> = serde_json::from_str(&proof_fields_content).map_err(|e| {
        create_smart_error(
            &format!("Failed to parse proof fields JSON: {}", e),
            &["Check that the proof fields file is valid JSON"],
        )
    })?;

    let public_inputs_fields: Vec<String> = serde_json::from_str(&public_inputs_fields_content)
        .map_err(|e| {
            create_smart_error(
                &format!("Failed to parse public inputs fields JSON: {}", e),
                &["Check that the public inputs fields file is valid JSON"],
            )
        })?;

    // Convert proof fields array to bytes format for the verifier
    // The UltraVerifier expects exactly 440 field elements, each 32 bytes
    const EXPECTED_PROOF_SIZE: usize = 440;
    let proof_fields_trimmed = if proof_fields.len() > EXPECTED_PROOF_SIZE {
        &proof_fields[0..EXPECTED_PROOF_SIZE]
    } else {
        &proof_fields
    };

    // Each field element should be exactly 32 bytes (64 hex chars without 0x)
    let proof_bytes: String = proof_fields_trimmed
        .iter()
        .map(|field| {
            let hex_without_prefix = field.trim_start_matches("0x");
            // Pad to 64 chars (32 bytes) if needed
            format!("{:0>64}", hex_without_prefix)
        })
        .collect();
    let proof_hex = format!("0x{}", proof_bytes);

    if cli.verbose {
        println!(
            "Proof fields count: {} (using first {})",
            proof_fields.len(),
            proof_fields_trimmed.len()
        );
        println!("Public inputs count: {}", public_inputs_fields.len());
        println!("Proof byte length: {}", proof_hex.len() - 2); // -2 for 0x prefix
    }

    // Due to command line length limits with very long proofs, we'll manually construct
    // the ABI-encoded calldata instead of using cast abi-encode
    if cli.verbose {
        println!("Manually constructing ABI-encoded calldata...");
    }

    if cli.dry_run {
        println!("Would manually construct ABI-encoded calldata for verify(bytes,bytes32[])");
        println!("Proof hex length: {} chars", proof_hex.len());
        println!("Public inputs array: [{}]", public_inputs_fields.join(","));
        return Ok(());
    }

    let calldata_timer = Timer::start();

    // Manually construct ABI-encoded calldata for verify(bytes,bytes32[])
    // Function selector for verify(bytes,bytes32[])
    let function_selector = "a8e16a41"; // First 4 bytes of keccak256("verify(bytes,bytes32[])")

    // Convert proof hex to bytes (remove 0x prefix)
    let proof_bytes_hex = proof_hex.trim_start_matches("0x");
    let proof_length = proof_bytes_hex.len() / 2; // Length in bytes

    // Calculate offsets
    let bytes_offset = 64; // 0x40 - offset to bytes data (after two 32-byte offset fields)
    let array_offset = bytes_offset + 32 + ((proof_length + 31) / 32) * 32; // Aligned to 32-byte boundary

    // Construct the calldata
    let mut calldata = String::new();
    calldata.push_str("0x");
    calldata.push_str(function_selector);

    // Offset to bytes parameter (32 bytes)
    calldata.push_str(&format!("{:0>64x}", bytes_offset));

    // Offset to bytes32[] parameter (32 bytes)
    calldata.push_str(&format!("{:0>64x}", array_offset));

    // Length of bytes data (32 bytes)
    calldata.push_str(&format!("{:0>64x}", proof_length));

    // The proof bytes data (padded to 32-byte boundary)
    calldata.push_str(proof_bytes_hex);
    let padding_needed = (32 - (proof_length % 32)) % 32;
    calldata.push_str(&"0".repeat(padding_needed * 2));

    // Length of bytes32[] array (32 bytes)
    calldata.push_str(&format!("{:0>64x}", public_inputs_fields.len()));

    // The bytes32[] elements
    for input in &public_inputs_fields {
        let input_hex = input.trim_start_matches("0x");
        calldata.push_str(&format!("{:0>64}", input_hex));
    }

    // Save calldata to file
    std::fs::create_dir_all("./target/bb").map_err(|e| {
        create_smart_error(
            &format!("Failed to create target/bb directory: {}", e),
            &["Check directory permissions"],
        )
    })?;

    let calldata_path = std::path::PathBuf::from("./target/bb/calldata");
    std::fs::write(&calldata_path, calldata).map_err(|e| {
        create_smart_error(
            &format!("Failed to write calldata file: {}", e),
            &["Check directory permissions"],
        )
    })?;

    if !cli.quiet {
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
        println!("  • Verify on-chain: bargo evm verify-onchain");
    }

    Ok(())
}

/// Handle `evm verify-onchain` command - Verify proof on EVM network
fn handle_evm_verify_onchain(cli: &Cli) -> Result<()> {
    let mut summary = OperationSummary::new();

    // Ensure Foundry is available
    backends::foundry::ensure_available().map_err(enhance_error_with_suggestions)?;

    // Check environment variables
    let rpc_url = std::env::var("RPC_URL").map_err(|_| {
        create_smart_error(
            "RPC_URL environment variable not found",
            &[
                "Create a .env file in your project root",
                "Add: RPC_URL=https://your-rpc-endpoint.com",
                "For Sepolia: RPC_URL=https://eth-sepolia.g.alchemy.com/v2/your-key",
                "For Mainnet: RPC_URL=https://eth-mainnet.g.alchemy.com/v2/your-key",
            ],
        )
    })?;

    let private_key = std::env::var("PRIVATE_KEY").map_err(|_| {
        create_smart_error(
            "PRIVATE_KEY environment variable not found",
            &[
                "Add to your .env file: PRIVATE_KEY=0x...",
                "Make sure your private key starts with 0x",
                "Never commit your .env file to version control",
            ],
        )
    })?;

    // Load saved contract address
    let contract_address_path = std::path::PathBuf::from("./target/bb/.bargo_contract_address");
    if !contract_address_path.exists() {
        return Err(create_smart_error(
            "Contract address not found",
            &[
                "Run 'bargo evm deploy' first to deploy the verifier contract",
                "Ensure the deployment was successful",
            ],
        ));
    }

    let contract_address = std::fs::read_to_string(&contract_address_path)
        .map_err(|e| {
            create_smart_error(
                &format!("Failed to read contract address: {}", e),
                &["Check file permissions and try redeploying"],
            )
        })?
        .trim()
        .to_string();

    // Load generated calldata
    let calldata_path = std::path::PathBuf::from("./target/bb/calldata");
    if !calldata_path.exists() {
        return Err(create_smart_error(
            "Calldata not found",
            &[
                "Run 'bargo evm calldata' first to generate calldata",
                "Ensure the proof and public inputs exist",
            ],
        ));
    }

    let calldata = std::fs::read_to_string(&calldata_path)
        .map_err(|e| {
            create_smart_error(
                &format!("Failed to read calldata: {}", e),
                &["Check file permissions and try regenerating calldata"],
            )
        })?
        .trim()
        .to_string();

    if !cli.quiet {
        println!("🚀 Verifying proof on-chain...");
        println!("📍 Contract: {}", contract_address);
    }

    // Send verification transaction using cast
    let verify_args = vec![
        "send",
        &contract_address,
        "verify(bytes,bytes32[])",
        &calldata,
        "--rpc-url",
        &rpc_url,
        "--private-key",
        &private_key,
    ];

    if cli.verbose {
        info!("Running: cast {}", verify_args.join(" "));
    }

    if cli.dry_run {
        println!("Would run: cast {}", verify_args.join(" "));
        return Ok(());
    }

    let verify_timer = Timer::start();
    let (stdout, _stderr) = backends::foundry::run_cast_with_output(&verify_args)
        .map_err(enhance_error_with_suggestions)?;

    // Parse transaction hash from output
    let tx_hash = stdout
        .lines()
        .find(|line| line.contains("transactionHash"))
        .and_then(|line| line.split('"').nth(3))
        .or_else(|| {
            // Alternative parsing for different output formats
            stdout
                .lines()
                .find(|line| line.starts_with("0x") && line.len() == 66)
                .map(|line| line.trim())
        })
        .ok_or_else(|| {
            create_smart_error(
                "Failed to parse transaction hash from cast output",
                &[
                    "Check that the transaction was successful",
                    "Verify your RPC_URL and PRIVATE_KEY are correct",
                    "Ensure you have sufficient funds for gas",
                    "Check that the contract address is correct",
                ],
            )
        })?;

    if !cli.quiet {
        println!(
            "{}",
            success(&format!(
                "Proof verified on-chain successfully ({})",
                verify_timer.elapsed()
            ))
        );
        println!("📋 Transaction hash: {}", tx_hash);

        // Try to provide explorer link if we can detect the network
        if rpc_url.contains("sepolia") {
            println!(
                "🔗 View on Etherscan: https://sepolia.etherscan.io/tx/{}",
                tx_hash
            );
        } else if rpc_url.contains("mainnet") || !rpc_url.contains("sepolia") {
            println!("🔗 View on Etherscan: https://etherscan.io/tx/{}", tx_hash);
        }

        summary.add_operation(&format!("Proof verified on-chain (tx: {})", tx_hash));
        summary.print();
    }

    Ok(())
}
