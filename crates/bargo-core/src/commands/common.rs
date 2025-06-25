use color_eyre::Result;
use tracing::info;

use crate::{config::Config, runner::CmdSpec};

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
/// - Command execution via the configured runner
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

    if cfg.verbose && !cfg.quiet {
        info!("Running: nargo {}", args.join(" "));
    }

    // Create command specification for nargo
    let spec = CmdSpec::new("nargo".to_string(), args);

    // Use the runner to execute the command (handles dry-run automatically)
    cfg.runner.run(&spec)

    // TODO: Migrate remaining shell-outs to use runner abstraction:
    // - scarb command executions (currently empty module)
    // - starknet CLI integrations
    //
    // Completed migrations:
    // ✅ bb command executions
    // ✅ garaga command executions
    // ✅ foundry command executions
    // ✅ nargo command executions
}

/// Run a garaga command with consolidated argument building, logging, and dry-run handling
///
/// This is the primary helper for executing garaga commands consistently across all
/// command modules. It handles:
/// - Verbose logging (when enabled and not quiet)
/// - Dry-run mode (prints command without executing)
/// - Command execution via the configured runner
///
/// # Arguments
/// * `cfg` - The global configuration containing all flags and runner
/// * `args` - Arguments to pass to garaga
///
/// # Returns
/// * `Result<()>` - Success or error from command execution
///
/// # Example
/// ```ignore
/// // Execute "garaga gen --system ultra_starknet_zk_honk --vk ./target/starknet/vk"
/// run_garaga_command(&config, &["gen", "--system", "ultra_starknet_zk_honk", "--vk", "./target/starknet/vk"])?;
/// ```
pub fn run_garaga_command(cfg: &Config, args: &[&str]) -> Result<()> {
    let args_vec: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    if cfg.verbose && !cfg.quiet {
        info!("Running: garaga {}", args_vec.join(" "));
    }

    // Create command specification for garaga
    let spec = CmdSpec::new("garaga".to_string(), args_vec);

    // Use the runner to execute the command (handles dry-run automatically)
    cfg.runner.run(&spec)
}

/// Run a foundry command with consolidated argument building, logging, and dry-run handling
///
/// This is the primary helper for executing foundry commands consistently across all
/// command modules. It handles:
/// - Verbose logging (when enabled and not quiet)
/// - Dry-run mode (prints command without executing)
/// - Command execution via the configured runner
///
/// # Arguments
/// * `cfg` - The global configuration containing all flags and runner
/// * `command` - The foundry command to run (forge, cast, anvil)
/// * `args` - Arguments to pass to the foundry command
///
/// # Returns
/// * `Result<()>` - Success or error from command execution
///
/// # Example
/// ```ignore
/// // Execute "forge init --force contracts/evm"
/// run_foundry_command(&config, "forge", &["init", "--force", "contracts/evm"])?;
/// ```
pub fn run_foundry_command(cfg: &Config, command: &str, args: &[&str]) -> Result<()> {
    let args_vec: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    if cfg.verbose && !cfg.quiet {
        info!("Running: {} {}", command, args_vec.join(" "));
    }

    // Create command specification for foundry command
    let spec = CmdSpec::new(command.to_string(), args_vec);

    // Use the runner to execute the command (handles dry-run automatically)
    cfg.runner.run(&spec)
}

/// Run a bb command with consolidated argument building, logging, and dry-run handling
///
/// This is the primary helper for executing bb commands consistently across all
/// command modules. It handles:
/// - Verbose logging (when enabled and not quiet)
/// - Dry-run mode (prints command without executing)
/// - Command execution via the configured runner
///
/// # Arguments
/// * `cfg` - The global configuration containing all flags and runner
/// * `args` - Arguments to pass to bb
///
/// # Returns
/// * `Result<()>` - Success or error from command execution
///
/// # Example
/// ```ignore
/// // Execute "bb prove --scheme ultra_honk -b bytecode.json -w witness.gz"
/// run_bb_command(&config, &["prove", "--scheme", "ultra_honk", "-b", "bytecode.json", "-w", "witness.gz"])?;
/// ```
pub fn run_bb_command(cfg: &Config, args: &[&str]) -> Result<()> {
    let args_vec: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    if cfg.verbose && !cfg.quiet {
        info!("Running: bb {}", args_vec.join(" "));
    }

    // Create command specification for bb
    let spec = CmdSpec::new("bb".to_string(), args_vec);

    // Use the runner to execute the command (handles dry-run automatically)
    cfg.runner.run(&spec)
}
