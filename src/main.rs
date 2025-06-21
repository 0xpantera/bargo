use clap::{Parser, Subcommand};
use color_eyre::Result;
use tracing::{info, warn};

mod backends;
mod util;

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
                println!("🔍 Checking circuit...");
            }
            handle_check(&cli)?;
        }
        Commands::Build => {
            if !cli.quiet {
                println!("🔨 Building circuit...");
            }
            handle_build(&cli)?;
        }
        Commands::Prove { skip_verify } => {
            if !cli.quiet {
                if skip_verify {
                    println!("⚡ Generating proof (skipping verification)...");
                } else {
                    println!("🔐 Generating and verifying proof...");
                }
            }
            handle_prove(&cli, skip_verify)?;
        }
        Commands::Verify => {
            if !cli.quiet {
                println!("✅ Verifying proof...");
            }
            handle_verify(&cli)?;
        }
        Commands::Solidity => {
            if !cli.quiet {
                println!("📄 Generating Solidity verifier...");
            }
            handle_solidity(&cli)?;
        }
        Commands::Clean => {
            if !cli.quiet {
                println!("🧹 Cleaning build artifacts...");
            }
            handle_clean(&cli)?;
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

    // Check if rebuild is needed (smart rebuild detection)
    if !cli.dry_run {
        let needs_rebuild = util::needs_rebuild(&pkg_name)?;
        if !needs_rebuild && !cli.quiet {
            println!("✅ Build is up to date");
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

    let result = backends::nargo::run(&["execute"]);

    if result.is_ok() && !cli.quiet {
        println!(
            "✅ Bytecode → {}",
            util::get_bytecode_path(&pkg_name).display()
        );
        println!(
            "✅ Witness → {}",
            util::get_witness_path(&pkg_name).display()
        );
    }

    result
}

fn handle_prove(cli: &Cli, skip_verify: bool) -> Result<()> {
    let pkg_name = get_package_name(cli)?;

    // Validate that required build files exist
    let required_files = vec![
        util::get_bytecode_path(&pkg_name),
        util::get_witness_path(&pkg_name),
    ];

    if !cli.dry_run {
        util::validate_files_exist(&required_files)?;
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
    backends::bb::run(&prove_args)?;

    if !cli.quiet {
        println!("✅ Proof generated → {}", util::get_proof_path().display());
    }

    // Generate verification key
    let vk_args = vec!["write_vk", "-b", &bytecode_str, "-o", "./target/"];

    if cli.verbose {
        info!("Running: bb {}", vk_args.join(" "));
    }

    backends::bb::run(&vk_args)?;

    if !cli.quiet {
        println!("✅ VK saved → {}", util::get_vk_path().display());
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

        backends::bb::run(&verify_args)?;

        if !cli.quiet {
            println!("✅ Proof verified successfully");
        }
    }

    Ok(())
}

fn handle_verify(cli: &Cli) -> Result<()> {
    // Validate that required files exist
    let required_files = vec![util::get_proof_path(), util::get_vk_path()];

    if !cli.dry_run {
        util::validate_files_exist(&required_files)?;
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

    backends::bb::run(&verify_args)?;

    if !cli.quiet {
        println!("✅ Proof verified successfully");
    }

    Ok(())
}

fn handle_solidity(cli: &Cli) -> Result<()> {
    let pkg_name = get_package_name(cli)?;

    // Validate that required build files exist
    let required_files = vec![util::get_bytecode_path(&pkg_name)];

    if !cli.dry_run {
        util::validate_files_exist(&required_files)?;
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

    backends::bb::run(&vk_args)?;

    if !cli.quiet {
        println!(
            "✅ VK (keccak optimized) → {}",
            util::get_vk_path().display()
        );
    }

    // Create contracts directory if it doesn't exist
    std::fs::create_dir_all("./contracts")?;

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

    backends::bb::run(&solidity_args)?;

    if !cli.quiet {
        println!("✅ Solidity verifier → contracts/Verifier.sol");
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
            println!("✅ Removed target/");
        }
    } else {
        if !cli.quiet {
            println!("✅ target/ already clean");
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

fn get_package_name(cli: &Cli) -> Result<String> {
    util::get_package_name(cli.pkg.as_ref())
}
