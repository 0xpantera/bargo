use color_eyre::Result;
use tracing::info;

use crate::{
    backends,
    util::{self, Flavour, Timer, format_operation_result, success},
    Cli,
};

/// Determine whether a rebuild is needed based on source timestamps
pub fn should_rebuild(pkg: &str, cli: &Cli) -> Result<bool> {
    if cli.dry_run { return Ok(true); }
    util::needs_rebuild(pkg)
}

/// Run `nargo execute` with the provided arguments
pub fn run_nargo_execute(args: &[&str]) -> Result<()> {
    backends::nargo::run(args)
}

/// Execute the build workflow
pub fn run(cli: &Cli) -> Result<()> {
    let pkg_name = util::get_package_name(cli.pkg.as_ref())?;

    if !should_rebuild(&pkg_name, cli)? {
        if !cli.quiet {
            println!("{}", success("Build is up to date"));
        }
        return Ok(());
    }

    let args = ["execute"];
    if cli.verbose { info!("Running: nargo execute {:?}", args); }

    if cli.dry_run {
        println!("Would run: nargo execute {}", args.join(" "));
        return Ok(());
    }

    let timer = Timer::start();
    run_nargo_execute(&args)?;

    util::organize_build_artifacts(&pkg_name, Flavour::Bb)?;
    if !cli.quiet {
        let bytecode_path = util::get_bytecode_path(&pkg_name, Flavour::Bb);
        let witness_path = util::get_witness_path(&pkg_name, Flavour::Bb);
        println!(
            "{}",
            success(&format_operation_result("Bytecode generated", &bytecode_path, &timer))
        );
        let witness_timer = Timer::start();
        println!(
            "{}",
            success(&format_operation_result("Witness generated", &witness_path, &witness_timer))
        );
    }
    Ok(())
}
