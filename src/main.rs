use color_eyre::Result;
use tracing::{info, warn};

mod backends;
mod util;
mod cli;
mod commands;

use clap::Parser;
use cli::{Cli, Commands, CairoCommands, EvmCommands, Backend};
use util::{print_banner};

fn main() -> Result<()> {
    color_eyre::install()?;
    dotenv::dotenv().ok();

    let cli = Cli::parse();

    setup_logging(cli.verbose, cli.quiet)?;

    if cli.verbose {
        info!("🚀 Starting bargo");
        if cli.dry_run {
            warn!("🔍 Dry run mode - commands will be printed but not executed");
        }
    }

    match &cli.command {
        Commands::Check => {
            if !cli.quiet { print_banner("check"); }
            commands::check::run(&cli)?;
        }
        Commands::Build => {
            if !cli.quiet { print_banner("build"); }
            commands::build::run(&cli)?;
        }
        Commands::Prove { skip_verify } => {
            if !cli.quiet { print_banner("prove"); if *skip_verify { println!("⚡ Skipping verification step\n"); } }
            commands::prove::run(&cli, *skip_verify)?;
        }
        Commands::Verify => {
            if !cli.quiet { print_banner("verify"); }
            commands::verify::run(&cli)?;
        }
        Commands::Verifier => {
            if !cli.quiet { print_banner("verifier"); }
            commands::verifier::run(&cli)?;
        }
        Commands::Clean { backend } => {
            if !cli.quiet { print_banner("clean"); }
            commands::clean::run(&cli, backend.unwrap_or(Backend::All))?;
        }
        Commands::Rebuild { backend } => {
            if !cli.quiet { print_banner("rebuild"); }
            commands::rebuild::run(&cli, backend.unwrap_or(Backend::All))?;
        }
        Commands::Cairo { command } => match command {
            CairoCommands::Gen => { if !cli.quiet { print_banner("cairo gen"); } commands::cairo::gen_cmd::run(&cli)?; }
            CairoCommands::Data => { if !cli.quiet { print_banner("cairo data"); } commands::cairo::data::run(&cli)?; }
            CairoCommands::Declare { network } => { if !cli.quiet { print_banner("cairo declare"); } commands::cairo::declare::run(&cli, network)?; }
            CairoCommands::Deploy { class_hash } => { if !cli.quiet { print_banner("cairo deploy"); } commands::cairo::deploy::run(&cli, class_hash.as_deref())?; }
            CairoCommands::VerifyOnchain { address } => { if !cli.quiet { print_banner("cairo verify-onchain"); } commands::cairo::verify_onchain::run(&cli, address.as_deref())?; }
        },
        Commands::Evm { command } => match command {
            EvmCommands::Gen => { if !cli.quiet { print_banner("evm gen"); } commands::evm::gen_cmd::run(&cli)?; }
            EvmCommands::Deploy { network } => { if !cli.quiet { print_banner("evm deploy"); } commands::evm::deploy::run(&cli, network)?; }
            EvmCommands::Calldata => { if !cli.quiet { print_banner("evm calldata"); } commands::evm::calldata::run(&cli)?; }
            EvmCommands::VerifyOnchain => { if !cli.quiet { print_banner("evm verify-onchain"); } commands::evm::verify_onchain::run(&cli)?; }
        },
        Commands::Doctor => {
            if !cli.quiet { print_banner("doctor"); }
            commands::doctor::run(&cli)?;
        }
    }

    if cli.verbose { info!("✨ bargo completed successfully"); }

    Ok(())
}

fn setup_logging(verbose: bool, quiet: bool) -> Result<()> {
    use tracing_subscriber::{EnvFilter, fmt};

    if quiet {
        let subscriber = fmt()
            .with_max_level(tracing::Level::ERROR)
            .with_target(false)
            .with_level(true)
            .finish();
        tracing::subscriber::set_global_default(subscriber)?;
    } else if verbose {
        unsafe { std::env::set_var("RUST_LOG", "info"); }
        let subscriber = fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .with_target(false)
            .with_level(true)
            .finish();
        tracing::subscriber::set_global_default(subscriber)?;
    } else {
        let subscriber = fmt()
            .with_max_level(tracing::Level::WARN)
            .with_target(false)
            .with_level(false)
            .finish();
        tracing::subscriber::set_global_default(subscriber)?;
    }
    Ok(())
}
