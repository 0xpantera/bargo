use color_eyre::Result;
use tracing::info;

use crate::{
    backends,
    commands::build_nargo_args,
    config::Config,
    util::{self, format_operation_result, success, Flavour, Timer},
};

/// Determine whether a rebuild is needed based on source timestamps
pub fn should_rebuild(pkg: &str, cfg: &Config) -> Result<bool> {
    if cfg.dry_run { return Ok(true); }
    util::needs_rebuild(pkg)
}

/// Run `nargo execute` with the provided arguments.
///
/// The slice is typically produced by [`build_nargo_args`].
pub fn run_nargo_execute(args: &[&str]) -> Result<()> {
    backends::nargo::run(args)
}

/// Execute the build workflow
pub fn run(cfg: &Config) -> Result<()> {
    let args = build_nargo_args(cfg, &["execute"])?;
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    if cfg.verbose {
        info!("Running: nargo {}", args.join(" "));
    }

    if cfg.dry_run {
        println!("Would run: nargo {}", args.join(" "));
        return Ok(());
    }

    let pkg_name = util::get_package_name(cfg.pkg.as_ref())?;

    if !should_rebuild(&pkg_name, cfg)? {
        if !cfg.quiet {
            println!("{}", success("Build is up to date"));
        }
        return Ok(());
    }

    let timer = Timer::start();
    run_nargo_execute(&arg_refs)?;

    util::organize_build_artifacts(&pkg_name, Flavour::Bb)?;
    if !cfg.quiet {
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
