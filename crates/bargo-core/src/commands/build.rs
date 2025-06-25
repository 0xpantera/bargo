use color_eyre::Result;

use crate::{
    commands::common::run_nargo_command,
    config::Config,
    util::{self, Flavour, Timer, format_operation_result, success},
};

/// Determine whether a rebuild is needed based on source timestamps
pub fn should_rebuild(pkg: &str, cfg: &Config) -> Result<bool> {
    if cfg.dry_run {
        return Ok(true);
    }
    util::needs_rebuild(pkg)
}

/// Execute the build workflow
pub fn run(cfg: &Config) -> Result<()> {
    if cfg.dry_run {
        return run_nargo_command(cfg, &["execute"]);
    }

    let pkg_name = util::get_package_name(cfg.pkg.as_ref())?;

    if !should_rebuild(&pkg_name, cfg)? {
        if !cfg.quiet {
            println!("{}", success("Build is up to date"));
        }
        return Ok(());
    }

    let timer = Timer::start();
    run_nargo_command(cfg, &["execute"])?;

    util::organize_build_artifacts(&pkg_name, Flavour::Bb)?;
    if !cfg.quiet {
        let bytecode_path = util::get_bytecode_path(&pkg_name, Flavour::Bb);
        let witness_path = util::get_witness_path(&pkg_name, Flavour::Bb);
        println!(
            "{}",
            success(&format_operation_result(
                "Bytecode generated",
                &bytecode_path,
                &timer
            ))
        );
        let witness_timer = Timer::start();
        println!(
            "{}",
            success(&format_operation_result(
                "Witness generated",
                &witness_path,
                &witness_timer
            ))
        );
    }
    Ok(())
}
