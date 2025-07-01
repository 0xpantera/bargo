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
}

/// Run any external tool with unified command execution
///
/// This is the unified helper for executing external tools consistently across all
/// command modules. It handles:
/// - Verbose logging (when enabled and not quiet)
/// - Dry-run mode (prints command without executing)
/// - Command execution via the configured runner
///
/// # Arguments
/// * `cfg` - The global configuration containing all flags and runner
/// * `tool` - The tool command to run (bb, garaga, forge, cast, nargo, etc.)
/// * `args` - Arguments to pass to the tool
///
/// # Returns
/// * `Result<()>` - Success or error from command execution
///
/// # Example
/// ```ignore
/// // Execute "bb prove --scheme ultra_honk -b bytecode.json"
/// run_tool(&config, "bb", &["prove", "--scheme", "ultra_honk", "-b", "bytecode.json"])?;
///
/// // Execute "garaga gen --system ultra_starknet_zk_honk --vk ./target/starknet/vk"
/// run_tool(&config, "garaga", &["gen", "--system", "ultra_starknet_zk_honk", "--vk", "./target/starknet/vk"])?;
/// ```
pub fn run_tool(cfg: &Config, tool: &str, args: &[&str]) -> Result<()> {
    let args_vec: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    if cfg.verbose && !cfg.quiet {
        info!("Running: {} {}", tool, args_vec.join(" "));
    }

    // Create command specification for the tool
    let spec = CmdSpec::new(tool.to_string(), args_vec);

    // Use the runner to execute the command (handles dry-run automatically)
    cfg.runner.run(&spec)
}

/// Run any external tool and capture its stdout
///
/// This is the unified helper for executing external tools that need to capture output.
/// It handles the same features as `run_tool` but returns the stdout as a string.
///
/// # Arguments
/// * `cfg` - The global configuration containing all flags and runner
/// * `tool` - The tool command to run (bb, garaga, forge, cast, nargo, etc.)
/// * `args` - Arguments to pass to the tool
///
/// # Returns
/// * `Result<String>` - Stdout from command execution or error
///
/// # Example
/// ```ignore
/// // Execute "garaga calldata ..." and capture output
/// let output = run_tool_capture(&config, "garaga", &["calldata", "--system", "ultra_starknet_zk_honk"])?;
/// ```
pub fn run_tool_capture(cfg: &Config, tool: &str, args: &[&str]) -> Result<String> {
    let args_vec: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    if cfg.verbose && !cfg.quiet {
        info!(
            "Running (capturing output): {} {}",
            tool,
            args_vec.join(" ")
        );
    }

    // Create command specification for the tool
    let spec = CmdSpec::new(tool.to_string(), args_vec);

    // Use the runner to execute the command and capture output
    cfg.runner.run_capture(&spec)
}
