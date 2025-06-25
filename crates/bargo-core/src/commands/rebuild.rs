use color_eyre::Result;
use tracing::info;

use crate::{
    backends,
    cli::Backend,
    config::Config,
    commands::build_nargo_args,
    util::{self, Flavour, OperationSummary, Timer, format_operation_result, path, success},
};

use super::clean;

pub fn run(cfg: &Config, backend: Backend) -> Result<()> {
    let mut summary = OperationSummary::new();

    // Step 1: Clean
    if cfg.verbose {
        info!("Step 1/2: Cleaning artifacts for backend: {:?}", backend);
    }

    if !cfg.quiet {
        println!("🧹 Cleaning build artifacts...");
    }

    clean::run(cfg, backend)?;
    if backend != Backend::Starknet {
        summary.add_operation("Build artifacts cleaned");
    }

    // Step 2: Build
    if cfg.verbose {
        info!("Step 2/2: Building from scratch");
    }

    if !cfg.quiet {
        println!("\n🔨 Building circuit...");
    }

    let pkg_name =
        util::get_package_name(cfg.pkg.as_ref()).map_err(util::enhance_error_with_suggestions)?;
    let args = build_nargo_args(cfg, &["execute"])?;
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    if cfg.verbose {
        info!("Running: nargo {}", args.join(" "));
    }

    if cfg.dry_run {
        println!("Would run: nargo {}", args.join(" "));
        return Ok(());
    }

    let timer = Timer::start();
    let result = backends::nargo::run(&arg_refs);

    match result {
        Ok(()) => {
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

                summary.add_operation(&format!("Circuit rebuilt for {}", path(&pkg_name)));
                summary.add_operation(&format!(
                    "Bytecode generated ({})",
                    util::format_file_size(&bytecode_path)
                ));
                summary.add_operation(&format!(
                    "Witness generated ({})",
                    util::format_file_size(&witness_path)
                ));
                summary.print();
            }
            Ok(())
        }
        Err(e) => Err(util::enhance_error_with_suggestions(e)),
    }
}
