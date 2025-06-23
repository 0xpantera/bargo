use color_eyre::Result;
use tracing::info;

use crate::{backends, util::{self, Flavour}};
use crate::Cli;

pub fn load_env_vars() {
    dotenv::dotenv().ok();
    if std::path::Path::new(".secrets").exists() {
        let _ = dotenv::from_filename(".secrets");
    }
}

pub fn generate_starknet_proof(pkg: &str) -> Result<()> {
    let bytecode = util::get_bytecode_path(pkg, Flavour::Bb);
    let witness = util::get_witness_path(pkg, Flavour::Bb);
    backends::bb::run(&[
        "prove", "-s", "ultra_honk", "--oracle_hash", "starknet", "--zk", "-b",
        &bytecode.to_string_lossy(), "-w", &witness.to_string_lossy(), "-o",
        "./target/starknet/",
    ])
}

pub fn run_scarb_build(project: &str) -> Result<()> {
    backends::garaga::run(&["build", project])
}

pub fn run_gen(cli: &Cli) -> Result<()> {
    let pkg_name = util::get_package_name(cli.pkg.as_ref())?;
    load_env_vars();
    if cli.verbose { info!("Generating Starknet proof"); }
    if !cli.dry_run {
        generate_starknet_proof(&pkg_name)?;
    }
    Ok(())
}
