use color_eyre::Result;
use tracing::info;

use crate::{
    backends,
    util::{self, Flavour, Timer, format_operation_result, success},
    Cli,
};

pub fn run_bb_prove(bytecode: &str, witness: &str) -> Result<()> {
    backends::bb::run(&["prove", "-b", bytecode, "-w", witness, "-o", "./target/bb/"])
}

pub fn generate_verification_key(bytecode: &str) -> Result<()> {
    backends::bb::run(&["write_vk", "-b", bytecode, "-o", "./target/bb/"])
}

pub fn verify_proof(vk: &str, proof: &str, public_inputs: &str) -> Result<()> {
    backends::bb::run(&["verify", "-k", vk, "-p", proof, "-i", public_inputs])
}

pub fn run(cli: &Cli, skip_verify: bool) -> Result<()> {
    let pkg_name = util::get_package_name(cli.pkg.as_ref())?;
    let bytecode_path = util::get_bytecode_path(&pkg_name, Flavour::Bb);
    let witness_path = util::get_witness_path(&pkg_name, Flavour::Bb);
    if !cli.dry_run {
        util::validate_files_exist(&[bytecode_path.clone(), witness_path.clone()])?;
    }

    let bytecode_str = bytecode_path.to_string_lossy();
    let witness_str = witness_path.to_string_lossy();

    if cli.verbose { info!("Running bb prove"); }
    if cli.dry_run {
        println!("Would run: bb prove -b {} -w {} -o ./target/bb/", bytecode_str, witness_str);
    } else {
        let timer = Timer::start();
        run_bb_prove(&bytecode_str, &witness_str)?;
        util::organize_bb_artifacts(Flavour::Bb)?;
        if !cli.quiet {
            let proof_path = util::get_proof_path(Flavour::Bb);
            println!(
                "{}",
                success(&format_operation_result("Proof generated", &proof_path, &timer))
            );
        }
    }

    if cli.verbose { info!("Generating verification key"); }
    if cli.dry_run {
        println!("Would run: bb write_vk -b {} -o ./target/bb/", bytecode_str);
    } else {
        let vk_timer = Timer::start();
        generate_verification_key(&bytecode_str)?;
        if !cli.quiet {
            let vk_path = util::get_vk_path(Flavour::Bb);
            println!(
                "{}",
                success(&format_operation_result("VK saved", &vk_path, &vk_timer))
            );
        }
    }

    if !skip_verify {
        let proof = util::get_proof_path(Flavour::Bb).to_string_lossy().to_string();
        let vk = util::get_vk_path(Flavour::Bb).to_string_lossy().to_string();
        let inputs = util::get_public_inputs_path(Flavour::Bb).to_string_lossy().to_string();
        if cli.verbose { info!("Verifying proof"); }
        if cli.dry_run {
            println!("Would run: bb verify -k {} -p {} -i {}", vk, proof, inputs);
        } else {
            let verify_timer = Timer::start();
            verify_proof(&vk, &proof, &inputs)?;
            if !cli.quiet {
                println!(
                    "{}",
                    success(&format!("Proof verified successfully ({})", verify_timer.elapsed()))
                );
            }
        }
    }

    Ok(())
}
