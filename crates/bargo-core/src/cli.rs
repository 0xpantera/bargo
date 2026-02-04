use clap::{Parser, Subcommand, ValueEnum};

/// A developer-friendly CLI wrapper for Noir ZK development
#[derive(Parser)]
#[command(
    name = "bargo",
    about = "A developer-friendly CLI wrapper for Noir ZK development",
    long_about = "bargo consolidates nargo and bb workflows into a single, opinionated tool that 'just works' in a standard Noir workspace.",
    version
)]
pub struct Cli {
    /// Enable verbose logging (shows underlying commands)
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Print commands without executing them
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Override package name (auto-detected from Nargo.toml)
    #[arg(long, global = true)]
    pub pkg: Option<String>,

    /// Minimize output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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
    #[cfg(feature = "cairo")]
    #[command(about = "Generate Cairo verifiers and interact with Starknet")]
    Cairo {
        #[command(subcommand)]
        command: CairoCommands,
    },

    /// EVM operations
    #[command(about = "Generate Solidity verifiers and interact with EVM networks")]
    Evm {
        #[command(subcommand)]
        command: EvmCommands,
    },

    /// Check system dependencies
    #[command(about = "Verify that all required tools are installed and available")]
    Doctor,
}

#[cfg(feature = "cairo")]
#[derive(Subcommand)]
pub enum CairoCommands {
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
    Calldata,

    /// Deploy declared verifier contract
    #[command(about = "Deploy declared verifier contract")]
    Deploy {
        /// Class hash of the declared contract
        #[arg(long)]
        class_hash: Option<String>,
        /// Automatically declare contract if not already declared (default: true)
        #[arg(long, default_value = "true")]
        auto_declare: bool,
        /// Skip automatic declaration (fails if contract not declared)
        #[arg(long, conflicts_with = "auto_declare")]
        no_declare: bool,
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
pub enum EvmCommands {
    /// Generate Solidity verifier contract
    #[command(about = "Generate Solidity verifier contract (Foundry project setup when enabled)")]
    Gen,

    /// Generate Keccak oracle proof
    #[command(about = "Generate proof using bb with Keccak oracle hash")]
    Prove,

    /// Verify Keccak oracle proof
    #[command(about = "Verify proof generated with Keccak oracle hash")]
    Verify,

    /// Deploy verifier contract to EVM network
    #[cfg(feature = "evm-foundry")]
    #[command(about = "Deploy verifier contract using Foundry")]
    Deploy {
        /// Network to deploy to (mainnet or sepolia)
        #[arg(long, default_value = "sepolia")]
        network: String,
    },

    /// Generate calldata for proof verification
    #[command(about = "Generate calldata for proof verification")]
    Calldata,

    /// Verify proof on-chain
    #[cfg(feature = "evm-foundry")]
    #[command(about = "Verify proof on EVM network using deployed verifier")]
    VerifyOnchain,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    /// Barretenberg backend (EVM/Solidity)
    Bb,
    /// Starknet backend (Cairo)
    #[cfg(feature = "cairo")]
    Starknet,
    /// All backends
    All,
}
