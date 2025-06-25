use color_eyre::Result;
use tracing::info;

use crate::{backends, config::Config};

/// Build argument list for nargo commands based on global config
///
/// This function takes base command arguments and extends them with global flags
/// from the configuration, such as `--package` when a specific package is specified.
///
/// # Arguments
/// * `cfg` - The global configuration containing flags like `pkg`
/// * `base_args` - Base command arguments (e.g., `["check"]` or `["execute"]`)
///
/// # Returns
/// * `Result<Vec<String>>` - Complete argument list ready for nargo execution
///
/// # Example
/// ```ignore
/// let args = build_nargo_args(&config, &["check"])?;
/// // If config.pkg is Some("my_package"), returns: ["check", "--package", "my_package"]
/// ```
pub fn build_nargo_args(cfg: &Config, base_args: &[&str]) -> Result<Vec<String>> {
    let mut args = base_args.iter().map(|s| s.to_string()).collect::<Vec<_>>();

    if let Some(pkg) = &cfg.pkg {
        args.push("--package".to_string());
        args.push(pkg.clone());
    }

    Ok(args)
}

/// Run a nargo command with consolidated argument building, logging, and dry-run handling
///
/// This is the primary helper for executing nargo commands consistently across all
/// command modules. It handles:
/// - Building arguments with global flags via `build_nargo_args`
/// - Verbose logging (when enabled and not quiet)
/// - Dry-run mode (prints command without executing)
/// - Backend execution via `backends::nargo::run`
///
/// # Arguments
/// * `cfg` - The global configuration containing all flags
/// * `base_args` - Base command arguments to pass to nargo
///
/// # Returns
/// * `Result<()>` - Success or error from command execution
///
/// # Example
/// ```ignore
/// // Execute "nargo check --package my_pkg" (with appropriate flags from config)
/// run_nargo_command(&config, &["check"])?;
/// ```
pub fn run_nargo_command(cfg: &Config, base_args: &[&str]) -> Result<()> {
    let args = build_nargo_args(cfg, base_args)?;
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    if cfg.verbose && !cfg.quiet {
        info!("Running: nargo {}", args.join(" "));
    }

    if cfg.dry_run {
        println!("Would run: nargo {}", args.join(" "));
        return Ok(());
    }

    backends::nargo::run(&arg_refs)
}
