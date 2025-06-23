use crate::{cli::{Cli, Backend}, util::{success, info}};
use color_eyre::Result;
use tracing::info as tinfo;

pub fn run(cli: &Cli, backend: Backend) -> Result<()> {
    if cli.verbose {
        tinfo!("Cleaning artifacts for backend: {:?}", backend);
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
            } else if !cli.quiet {
                println!("{}", info("target/ already clean"));
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
            } else if !cli.quiet {
                println!("{}", info("target/bb/ already clean"));
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
            } else if !cli.quiet {
                println!("{}", info("target/starknet/ already clean"));
            }
        }
    }

    Ok(())
}
