use color_eyre::Result;
use tracing::info;

use crate::Cli;
use crate::{
    backends,
    util::{self, Flavour},
};

pub fn generate_keccak_proof(pkg: &str) -> Result<()> {
    // Create target/evm directory if it doesn't exist
    std::fs::create_dir_all("./target/evm")?;

    let bytecode = util::get_bytecode_path(pkg, Flavour::Bb);
    let witness = util::get_witness_path(pkg, Flavour::Bb);
    backends::bb::run(&[
        "prove",
        "-b",
        &bytecode.to_string_lossy(),
        "-w",
        &witness.to_string_lossy(),
        "-o",
        "./target/evm/",
        "--oracle_hash",
        "keccak",
        "--output_format",
        "bytes_and_fields",
    ])
}

pub fn generate_keccak_vk(pkg: &str) -> Result<()> {
    let bytecode = util::get_bytecode_path(pkg, Flavour::Bb);
    backends::bb::run(&[
        "write_vk",
        "--oracle_hash",
        "keccak",
        "-b",
        &bytecode.to_string_lossy(),
        "-o",
        "./target/evm/",
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
    if cli.verbose {
        info!("Initializing Foundry project");
    }
    if !cli.dry_run {
        init_foundry_project("contracts/evm")?;
    } else {
        println!("Would run: forge init --force contracts/evm");
    }
    if cli.verbose {
        info!("Generating keccak proof");
    }
    if !cli.dry_run {
        generate_keccak_proof(&pkg_name)?;
    } else {
        let bytecode = util::get_bytecode_path(&pkg_name, Flavour::Bb);
        let witness = util::get_witness_path(&pkg_name, Flavour::Bb);
        println!(
            "Would run: bb prove -b {} -w {} -o ./target/evm/ --oracle_hash keccak --output_format bytes_and_fields",
            bytecode.display(),
            witness.display()
        );
    }
    if cli.verbose {
        info!("Generating keccak VK");
    }
    if !cli.dry_run {
        generate_keccak_vk(&pkg_name)?;
    } else {
        let bytecode = util::get_bytecode_path(&pkg_name, Flavour::Bb);
        println!(
            "Would run: bb write_vk --oracle_hash keccak -b {} -o ./target/evm/",
            bytecode.display()
        );
    }
    // Create contracts directory if it doesn't exist
    if !cli.dry_run {
        std::fs::create_dir_all("./contracts/evm/src")?;
    } else {
        println!("Would run: mkdir -p ./contracts/evm/src");
    }
    if cli.verbose {
        info!("Writing Solidity verifier");
    }
    if !cli.dry_run {
        let vk = util::get_vk_path(Flavour::Evm);
        let vk_str = vk.to_string_lossy().to_string();
        write_verifier_contract(&vk_str, "contracts/evm/src/Verifier.sol")?;
    } else {
        let vk = util::get_vk_path(Flavour::Evm);
        println!(
            "Would run: bb write_solidity_verifier -k {} -o contracts/evm/src/Verifier.sol",
            vk.display()
        );
    }
    Ok(())
}
