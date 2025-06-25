use clap::Parser;
use color_eyre::Result;
use tracing::{info, warn};

mod backends;
mod util;

pub mod backend;
pub mod cli;
pub mod commands;
pub mod config;
pub mod runner;

use backend::{BackendConfig, BackendKind, backend_for};
use config::CairoDeployConfig;

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
                let mut backend = backend_for(BackendKind::Cairo);
                backend.generate(cfg)
            }
            CairoCommands::Prove => {
                if !cfg.quiet {
                    print_banner("cairo prove");
                }
                let mut backend = backend_for(BackendKind::Cairo);
                backend.prove(cfg)
            }
            CairoCommands::Verify => {
                if !cfg.quiet {
                    print_banner("cairo verify");
                }
                let mut backend = backend_for(BackendKind::Cairo);
                backend.verify(cfg)
            }
            CairoCommands::Calldata => {
                if !cfg.quiet {
                    print_banner("cairo calldata");
                }
                let mut backend = backend_for(BackendKind::Cairo);
                backend.calldata(cfg)
            }

            CairoCommands::Deploy {
                class_hash,
                auto_declare,
                no_declare,
            } => {
                if !cfg.quiet {
                    print_banner("cairo deploy");
                }
                let mut backend = backend_for(BackendKind::Cairo);

                // Configure the backend with deploy-specific settings
                let deploy_config =
                    CairoDeployConfig::new(class_hash.clone(), *auto_declare, *no_declare);
                backend.configure(BackendConfig::CairoDeploy(deploy_config))?;

                backend.deploy(cfg, None)
            }
            CairoCommands::VerifyOnchain { address } => {
                if !cfg.quiet {
                    print_banner("cairo verify-onchain");
                }
                let mut backend = backend_for(BackendKind::Cairo);
                backend.verify_onchain(cfg, address.as_deref())
            }
        },
        Commands::Evm { command } => match command {
            EvmCommands::Gen => {
                if !cfg.quiet {
                    print_banner("evm gen");
                }
                let mut backend = backend_for(BackendKind::Evm);
                backend.generate(cfg)
            }
            EvmCommands::Prove => {
                if !cfg.quiet {
                    print_banner("evm prove");
                }
                let mut backend = backend_for(BackendKind::Evm);
                backend.prove(cfg)
            }
            EvmCommands::Verify => {
                if !cfg.quiet {
                    print_banner("evm verify");
                }
                let mut backend = backend_for(BackendKind::Evm);
                backend.verify(cfg)
            }
            EvmCommands::Deploy { network } => {
                if !cfg.quiet {
                    print_banner("evm deploy");
                }
                let mut backend = backend_for(BackendKind::Evm);
                backend.deploy(cfg, Some(network))
            }
            EvmCommands::Calldata => {
                if !cfg.quiet {
                    print_banner("evm calldata");
                }
                let mut backend = backend_for(BackendKind::Evm);
                backend.calldata(cfg)
            }
            EvmCommands::VerifyOnchain => {
                if !cfg.quiet {
                    print_banner("evm verify-onchain");
                }
                let mut backend = backend_for(BackendKind::Evm);
                backend.verify_onchain(cfg, None)
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
    use tracing_subscriber::{EnvFilter, fmt};

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
