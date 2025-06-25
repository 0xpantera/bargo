use color_eyre::Result;
use tracing::info;

use crate::{config::Config, cli::Backend, util::{info as info_msg, success}};

pub fn run(cfg: &Config, backend: Backend) -> Result<()> {
    if cfg.verbose {
        info!("Cleaning artifacts for backend: {:?}", backend);
    }

    match backend {
        Backend::All => {
            if cfg.dry_run {
                println!("Would run: rm -rf target/");
                return Ok(());
            }

            if std::path::Path::new("target").exists() {
                std::fs::remove_dir_all("target")?;
                if !cfg.quiet {
                    println!("{}", success("Removed target/"));
                }
            } else if !cfg.quiet {
                println!("{}", info_msg("target/ already clean"));
            }
        }
        Backend::Bb => {
            if cfg.dry_run {
                println!("Would run: rm -rf target/bb/");
                return Ok(());
            }

            if std::path::Path::new("target/bb").exists() {
                std::fs::remove_dir_all("target/bb")?;
                if !cfg.quiet {
                    println!("{}", success("Removed target/bb/"));
                }
            } else if !cfg.quiet {
                println!("{}", info_msg("target/bb/ already clean"));
            }
        }
        Backend::Starknet => {
            if cfg.dry_run {
                println!("Would run: rm -rf target/starknet/");
                return Ok(());
            }

            if std::path::Path::new("target/starknet").exists() {
                std::fs::remove_dir_all("target/starknet")?;
                if !cfg.quiet {
                    println!("{}", success("Removed target/starknet/"));
                }
            } else if !cfg.quiet {
                println!("{}", info_msg("target/starknet/ already clean"));
            }
        }
    }

    Ok(())
}
