//! Build command implementation

use color_eyre::Result;
use std::path::Path;

use crate::{
    commands::common::run_nargo_command_in_directory,
    config::Config,
    util::{self, Flavour, Timer, format_operation_result, success},
};

/// Determine whether a rebuild is needed based on source timestamps
fn should_rebuild(pkg_name: &str, cfg: &Config) -> Result<bool> {
    if cfg.dry_run {
        return Ok(true);
    }
    util::needs_rebuild(pkg_name)
}

/// Execute the build workflow
pub fn run(cfg: &Config) -> Result<()> {
    run_in_directory(cfg, None)
}

/// Execute the build workflow in a specific directory
pub fn run_in_directory(cfg: &Config, working_dir: Option<&Path>) -> Result<()> {
    if cfg.dry_run {
        return run_nargo_command_in_directory(cfg, &["execute"], working_dir);
    }

    let pkg_name = match working_dir {
        Some(dir) => util::get_package_name_in_directory(cfg.pkg.as_ref(), dir)?,
        None => util::get_package_name(cfg.pkg.as_ref())?,
    };

    if !should_rebuild(&pkg_name, cfg)? {
        if !cfg.quiet {
            println!("{}", success("Build is up to date"));
        }
        return Ok(());
    }

    let timer = Timer::start();
    run_nargo_command_in_directory(cfg, &["execute"], working_dir)?;

    match working_dir {
        Some(dir) => util::organize_build_artifacts_in_directory(&pkg_name, Flavour::Bb, dir)?,
        None => util::organize_build_artifacts(&pkg_name, Flavour::Bb)?,
    }

    if !cfg.quiet {
        let current_dir;
        let base_dir = match working_dir {
            Some(dir) => dir,
            None => {
                current_dir = std::env::current_dir()?;
                &current_dir
            }
        };
        let bytecode_path = base_dir.join(util::get_bytecode_path(&pkg_name, Flavour::Bb));
        println!(
            "{}",
            success(&format_operation_result(
                "Build completed",
                &bytecode_path,
                &timer
            ))
        );
    }

    Ok(())
}
