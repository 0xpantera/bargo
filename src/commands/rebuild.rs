use crate::{backends, cli::{Cli, Backend}, commands::build_nargo_args, util::{self, Flavour, OperationSummary, Timer, success, format_operation_result, path, enhance_error_with_suggestions}};
use color_eyre::Result;
use tracing::info;

pub fn run(cli: &Cli, backend: Backend) -> Result<()> {
    let mut summary = OperationSummary::new();

    if cli.verbose {
        info!("Step 1/2: Cleaning artifacts for backend: {:?}", backend);
    }

    if !cli.quiet {
        println!("🧹 Cleaning build artifacts...");
    }

    super::clean::run(cli, backend)?;
    if backend != Backend::Starknet {
        summary.add_operation("Build artifacts cleaned");
    }

    if cli.verbose {
        info!("Step 2/2: Building from scratch");
    }

    if !cli.quiet {
        println!("\n🔨 Building circuit...");
    }

    let pkg_name = util::get_package_name(cli.pkg.as_ref())?;
    let args = build_nargo_args(cli, &[])?;

    if cli.verbose {
        info!("Running: nargo execute {}", args.join(" "));
    }

    if cli.dry_run {
        println!("Would run: nargo execute {}", args.join(" "));
        return Ok(());
    }

    let timer = Timer::start();
    let result = backends::nargo::run(&["execute"]);

    match result {
        Ok(()) => {
            util::organize_build_artifacts(&pkg_name, Flavour::Bb)?;

            if !cli.quiet {
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
        Err(e) => Err(enhance_error_with_suggestions(e)),
    }
}
