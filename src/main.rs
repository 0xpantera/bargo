use clap::{Parser, Subcommand, ValueEnum};
use color_eyre::Result;
use tracing::{info, warn};

mod backends;
mod commands;
mod util;

use util::{
    Flavour, OperationSummary, Timer, enhance_error_with_suggestions, format_operation_result,
    info, path, print_banner, success,
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

    /// Generate Starknet oracle proof
    #[command(about = "Generate proof using bb with Starknet oracle hash")]
    Prove,

    /// Verify Starknet oracle proof
    #[command(about = "Verify proof generated with Starknet oracle hash")]
    Verify,

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

    /// Generate Keccak oracle proof
    #[command(about = "Generate proof using bb with Keccak oracle hash")]
    Prove,

    /// Verify Keccak oracle proof
    #[command(about = "Verify proof generated with Keccak oracle hash")]
    Verify,

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
            CairoCommands::Prove => {
                if !cli.quiet {
                    print_banner("cairo prove");
                }
                handle_cairo_prove(&cli)?;
            }
            CairoCommands::Verify => {
                if !cli.quiet {
                    print_banner("cairo verify");
                }
                handle_cairo_verify(&cli)?;
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
            EvmCommands::Prove => {
                if !cli.quiet {
                    print_banner("evm prove");
                }
                handle_evm_prove(&cli)?;
            }
            EvmCommands::Verify => {
                if !cli.quiet {
                    print_banner("evm verify");
                }
                handle_evm_verify(&cli)?;
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

fn handle_cairo_prove(cli: &Cli) -> Result<()> {
    commands::cairo::run_prove(cli).map_err(enhance_error_with_suggestions)
}

fn handle_cairo_verify(cli: &Cli) -> Result<()> {
    commands::cairo::run_verify(cli).map_err(enhance_error_with_suggestions)
}

fn handle_evm_prove(cli: &Cli) -> Result<()> {
    commands::evm::run_prove(cli).map_err(enhance_error_with_suggestions)
}

fn handle_evm_verify(cli: &Cli) -> Result<()> {
    commands::evm::run_verify(cli).map_err(enhance_error_with_suggestions)
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
    commands::cairo::run_data(cli).map_err(enhance_error_with_suggestions)
}

fn handle_cairo_declare(cli: &Cli, network: &str) -> Result<()> {
    commands::cairo::run_declare(cli, network).map_err(enhance_error_with_suggestions)
}

fn handle_cairo_deploy(cli: &Cli, class_hash: Option<&str>) -> Result<()> {
    commands::cairo::run_deploy(cli, class_hash).map_err(enhance_error_with_suggestions)
}

fn handle_cairo_verify_onchain(cli: &Cli, address: Option<&str>) -> Result<()> {
    commands::cairo::run_verify_onchain(cli, address).map_err(enhance_error_with_suggestions)
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
    commands::evm::run_gen(cli).map_err(enhance_error_with_suggestions)
}
/// Handle `evm deploy` command - Deploy verifier contract to EVM network
fn handle_evm_deploy(cli: &Cli, network: &str) -> Result<()> {
    commands::evm::run_deploy(cli, network).map_err(enhance_error_with_suggestions)
}

/// Handle `evm calldata` command - Generate calldata for proof verification
fn handle_evm_calldata(cli: &Cli) -> Result<()> {
    commands::evm::run_calldata(cli).map_err(enhance_error_with_suggestions)
}

/// Handle `evm verify-onchain` command - Verify proof on EVM network
fn handle_evm_verify_onchain(cli: &Cli) -> Result<()> {
    commands::evm::run_verify_onchain(cli).map_err(enhance_error_with_suggestions)
}
