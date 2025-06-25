use clap::Parser;
use color_eyre::Result;
use tracing::{info, warn};

mod backends;
mod util;

pub mod cli;
pub mod commands;
pub mod config;

pub use cli::Cli;
pub use config::Config;

pub fn run() -> Result<()> {
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

    let cfg = Config::from(&cli);
    dispatch(&cli, &cfg)?;

    if cli.verbose {
        info!("✨ bargo completed successfully");
    }

    Ok(())
}

fn dispatch(cli: &Cli, cfg: &Config) -> Result<()> {
    use cli::{Backend, CairoCommands, Commands, EvmCommands};
    use util::print_banner;

    match &cli.command {
        Commands::Check => {
            if !cfg.quiet {
                print_banner("check");
            }
            commands::check::run(cfg)
        }
        Commands::Build => {
            if !cfg.quiet {
                print_banner("build");
            }
            commands::build::run(cfg)
        }
        Commands::Clean { backend } => {
            if !cfg.quiet {
                print_banner("clean");
            }
            commands::clean::run(cfg, backend.unwrap_or(Backend::All))
        }
        Commands::Rebuild { backend } => {
            if !cfg.quiet {
                print_banner("rebuild");
            }
            commands::rebuild::run(cfg, backend.unwrap_or(Backend::All))
        }
        Commands::Cairo { command } => match command {
            CairoCommands::Gen => {
                if !cfg.quiet {
                    print_banner("cairo gen");
                }
                commands::cairo::run_gen(cfg)
            }
            CairoCommands::Prove => {
                if !cfg.quiet {
                    print_banner("cairo prove");
                }
                commands::cairo::run_prove(cfg)
            }
            CairoCommands::Verify => {
                if !cfg.quiet {
                    print_banner("cairo verify");
                }
                commands::cairo::run_verify(cfg)
            }
            CairoCommands::Calldata => {
                if !cfg.quiet {
                    print_banner("cairo calldata");
                }
                commands::cairo::run_calldata(cfg)
            }
            CairoCommands::Declare { network } => {
                if !cfg.quiet {
                    print_banner("cairo declare");
                }
                commands::cairo::run_declare(cfg, network)
            }
            CairoCommands::Deploy { class_hash } => {
                if !cfg.quiet {
                    print_banner("cairo deploy");
                }
                commands::cairo::run_deploy(cfg, class_hash.as_deref())
            }
            CairoCommands::VerifyOnchain { address } => {
                if !cfg.quiet {
                    print_banner("cairo verify-onchain");
                }
                commands::cairo::run_verify_onchain(cfg, address.as_deref())
            }
        },
        Commands::Evm { command } => match command {
            EvmCommands::Gen => {
                if !cfg.quiet {
                    print_banner("evm gen");
                }
                commands::evm::run_gen(cfg)
            }
            EvmCommands::Prove => {
                if !cfg.quiet {
                    print_banner("evm prove");
                }
                commands::evm::run_prove(cfg)
            }
            EvmCommands::Verify => {
                if !cfg.quiet {
                    print_banner("evm verify");
                }
                commands::evm::run_verify(cfg)
            }
            EvmCommands::Deploy { network } => {
                if !cfg.quiet {
                    print_banner("evm deploy");
                }
                commands::evm::run_deploy(cfg, network)
            }
            EvmCommands::Calldata => {
                if !cfg.quiet {
                    print_banner("evm calldata");
                }
                commands::evm::run_calldata(cfg)
            }
            EvmCommands::VerifyOnchain => {
                if !cfg.quiet {
                    print_banner("evm verify-onchain");
                }
                commands::evm::run_verify_onchain(cfg)
            }
        },
        Commands::Doctor => {
            if !cfg.quiet {
                print_banner("doctor");
            }
            commands::doctor::run(cfg)
        }
    }
}

fn setup_logging(verbose: bool, quiet: bool) -> Result<()> {
    use tracing_subscriber::{fmt, EnvFilter};

    if quiet {
        let subscriber = fmt()
            .with_max_level(tracing::Level::ERROR)
            .with_target(false)
            .with_level(true)
            .finish();
        tracing::subscriber::set_global_default(subscriber)?;
    } else if verbose {
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
        let subscriber = fmt()
            .with_env_filter(filter)
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
