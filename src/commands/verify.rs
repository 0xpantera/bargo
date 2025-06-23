use crate::{backends, cli::Cli, util::{self, Flavour, Timer, success, enhance_error_with_suggestions}};
use color_eyre::Result;
use tracing::info;

pub fn run(cli: &Cli) -> Result<()> {
    let required_files = vec![
        util::get_proof_path(Flavour::Bb),
        util::get_vk_path(Flavour::Bb),
        util::get_public_inputs_path(Flavour::Bb),
    ];

    if !cli.dry_run {
        util::validate_files_exist(&required_files).map_err(enhance_error_with_suggestions)?;
    }

    let proof_path = util::get_proof_path(Flavour::Bb);
    let vk_path = util::get_vk_path(Flavour::Bb);
    let public_inputs_path = util::get_public_inputs_path(Flavour::Bb);
    let vk_str = vk_path.to_string_lossy();
    let proof_str = proof_path.to_string_lossy();
    let public_inputs_str = public_inputs_path.to_string_lossy();
    let verify_args = vec![
        "verify",
        "-k",
        &vk_str,
        "-p",
        &proof_str,
        "-i",
        &public_inputs_str,
    ];

    if cli.verbose {
        info!("Running: bb {}", verify_args.join(" "));
    }

    if cli.dry_run {
        println!("Would run: bb {}", verify_args.join(" "));
        return Ok(());
    }

    let timer = Timer::start();
    backends::bb::run(&verify_args).map_err(enhance_error_with_suggestions)?;

    if !cli.quiet {
        println!(
            "{}",
            success(&format!("Proof verified successfully ({})", timer.elapsed()))
        );
    }

    Ok(())
}
