use color_eyre::Result;
use tracing::info;

use crate::{backends, util::{self, Flavour}};
use crate::Cli;

pub fn generate_keccak_proof(pkg: &str) -> Result<()> {
    let bytecode = util::get_bytecode_path(pkg, Flavour::Bb);
    let witness = util::get_witness_path(pkg, Flavour::Bb);
    backends::bb::run(&[
        "prove", "-b", &bytecode.to_string_lossy(), "-w", &witness.to_string_lossy(),
        "-o", "./target/bb", "--oracle_hash", "keccak", "--output_format", "fields",
    ])
}

pub fn write_verifier_contract(vk: &str, out: &str) -> Result<()> {
    backends::bb::run(&["write_solidity_verifier", "-k", vk, "-o", out])
}

pub fn init_foundry_project(path: &str) -> Result<()> {
    backends::foundry::run_forge(&["init", "--force", path])
}

pub fn run(cli: &Cli) -> Result<()> {
    let pkg_name = util::get_package_name(cli.pkg.as_ref())?;
    if cli.verbose { info!("Initializing Foundry project"); }
    if !cli.dry_run {
        init_foundry_project("contracts/evm")?;
    }
    if cli.verbose { info!("Generating keccak proof"); }
    if !cli.dry_run {
        generate_keccak_proof(&pkg_name)?;
    }
    if cli.verbose { info!("Writing Solidity verifier"); }
    if !cli.dry_run {
        let vk = util::get_vk_path(Flavour::Bb);
        let vk_str = vk.to_string_lossy().to_string();
        write_verifier_contract(&vk_str, "contracts/evm/src/Verifier.sol")?;
    }
    Ok(())
}
