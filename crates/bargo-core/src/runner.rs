//! Command execution abstraction for bargo
//!
//! This module provides a unified interface for command execution that supports
//! both real execution and dry-run mode, making it easier to test commands and
//! provide user feedback about what operations would be performed.

use std::path::PathBuf;
use std::process::Command;

use color_eyre::Result;
use color_eyre::eyre::WrapErr;

/// Specification for a command to be executed
///
/// This struct encapsulates all the information needed to execute a command,
/// including the command itself, arguments, working directory, and environment variables.
#[derive(Debug, Clone)]
pub struct CmdSpec {
    /// The command to execute (e.g., "cargo", "nargo", "bb")
    pub cmd: String,

    /// Arguments to pass to the command
    pub args: Vec<String>,

    /// Optional working directory to execute the command in
    pub cwd: Option<PathBuf>,

    /// Environment variables to set for the command (key, value pairs)
    pub env: Vec<(String, String)>,
}

impl CmdSpec {
    /// Create a new command specification
    ///
    /// # Arguments
    /// * `cmd` - The command to execute
    /// * `args` - Arguments to pass to the command
    ///
    /// # Returns
    /// * `CmdSpec` - New command specification with no working directory or environment variables
    ///
    /// # Example
    /// ```ignore
    /// let spec = CmdSpec::new("cargo".to_string(), vec!["check".to_string()]);
    /// ```
    pub fn new(cmd: String, args: Vec<String>) -> Self {
        Self {
            cmd,
            args,
            cwd: None,
            env: Vec::new(),
        }
    }

    /// Set the working directory for the command
    ///
    /// # Arguments
    /// * `cwd` - Working directory path
    ///
    /// # Returns
    /// * `Self` - Modified command specification
    ///
    /// # Example
    /// ```ignore
    /// let spec = CmdSpec::new("cargo".to_string(), vec!["check".to_string()])
    ///     .with_cwd(PathBuf::from("./my-project"));
    /// ```
    pub fn with_cwd(mut self, cwd: PathBuf) -> Self {
        self.cwd = Some(cwd);
        self
    }

    /// Add an environment variable to the command
    ///
    /// # Arguments
    /// * `key` - Environment variable name
    /// * `value` - Environment variable value
    ///
    /// # Returns
    /// * `Self` - Modified command specification
    ///
    /// # Example
    /// ```ignore
    /// let spec = CmdSpec::new("cargo".to_string(), vec!["check".to_string()])
    ///     .with_env("RUST_LOG".to_string(), "debug".to_string());
    /// ```
    pub fn with_env(mut self, key: String, value: String) -> Self {
        self.env.push((key, value));
        self
    }

    /// Add multiple environment variables to the command
    ///
    /// # Arguments
    /// * `env_vars` - Vector of (key, value) pairs
    ///
    /// # Returns
    /// * `Self` - Modified command specification
    ///
    /// # Example
    /// ```ignore
    /// let spec = CmdSpec::new("cargo".to_string(), vec!["check".to_string()])
    ///     .with_envs(vec![
    ///         ("RUST_LOG".to_string(), "debug".to_string()),
    ///         ("CARGO_TERM_COLOR".to_string(), "always".to_string()),
    ///     ]);
    /// ```
    pub fn with_envs(mut self, env_vars: Vec<(String, String)>) -> Self {
        self.env.extend(env_vars);
        self
    }
}

/// Trait for command execution strategies
///
/// This trait provides a unified interface for different command execution strategies,
/// allowing the same command specification to be executed in different ways
/// (real execution vs. dry-run) based on runtime configuration.
pub trait Runner: std::fmt::Debug {
    /// Execute a command specification
    ///
    /// # Arguments
    /// * `spec` - Command specification to execute
    ///
    /// # Returns
    /// * `Result<()>` - Success or error from command execution
    ///
    /// # Example
    /// ```ignore
    /// let runner = RealRunner;
    /// let spec = CmdSpec::new("echo".to_string(), vec!["hello".to_string()]);
    /// runner.run(&spec)?;
    /// ```
    fn run(&self, spec: &CmdSpec) -> Result<()>;

    /// Execute a command specification and capture its stdout
    ///
    /// # Arguments
    /// * `spec` - Command specification to execute
    ///
    /// # Returns
    /// * `Result<String>` - Stdout from command execution or error
    ///
    /// # Example
    /// ```ignore
    /// let runner = RealRunner;
    /// let spec = CmdSpec::new("echo".to_string(), vec!["hello".to_string()]);
    /// let output = runner.run_capture(&spec)?;
    /// ```
    fn run_capture(&self, spec: &CmdSpec) -> Result<String>;
}

/// Real command runner that actually executes commands
///
/// This runner executes commands using the system's process spawning mechanisms.
/// It should be used in production mode when commands need to actually run.
#[derive(Debug)]
pub struct RealRunner;

impl RealRunner {
    /// Create a new real command runner
    pub fn new() -> Self {
        Self
    }
}

impl Default for RealRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl Runner for RealRunner {
    /// Execute a command specification using real process spawning
    ///
    /// This method creates a new process and executes the specified command
    /// with the given arguments, working directory, and environment variables.
    ///
    /// # Arguments
    /// * `spec` - Command specification to execute
    ///
    /// # Returns
    /// * `Result<()>` - Success if command completed successfully, error otherwise
    fn run(&self, spec: &CmdSpec) -> Result<()> {
        let mut cmd = Command::new(&spec.cmd);

        // Add arguments
        cmd.args(&spec.args);

        // Set working directory if specified
        if let Some(ref cwd) = spec.cwd {
            cmd.current_dir(cwd);
        }

        // Set environment variables
        for (key, value) in &spec.env {
            cmd.env(key, value);
        }

        // Execute the command
        let output = cmd
            .output()
            .wrap_err_with(|| format!("Failed to execute command '{}'", spec.cmd))?;

        // Check if command succeeded
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);

            return Err(color_eyre::eyre::eyre!(
                "Command '{}' failed with exit code {:?}\nStdout: {}\nStderr: {}",
                spec.cmd,
                output.status.code(),
                stdout,
                stderr
            ))
            .wrap_err_with(|| {
                format!(
                    "Command execution failed: {} {}",
                    spec.cmd,
                    spec.args.join(" ")
                )
            });
        }

        // Print stdout if there's any output
        if !output.stdout.is_empty() {
            print!("{}", String::from_utf8_lossy(&output.stdout));
        }

        Ok(())
    }

    fn run_capture(&self, spec: &CmdSpec) -> Result<String> {
        let mut cmd = Command::new(&spec.cmd);

        // Add arguments
        cmd.args(&spec.args);

        // Set working directory if specified
        if let Some(ref cwd) = spec.cwd {
            cmd.current_dir(cwd);
        }

        // Set environment variables
        for (key, value) in &spec.env {
            cmd.env(key, value);
        }

        // Execute the command
        let output = cmd
            .output()
            .wrap_err_with(|| format!("Failed to execute command '{}'", spec.cmd))?;

        // Check if command succeeded
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);

            return Err(color_eyre::eyre::eyre!(
                "Command '{}' failed with exit code {:?}\nStdout: {}\nStderr: {}",
                spec.cmd,
                output.status.code(),
                stdout,
                stderr
            ))
            .wrap_err_with(|| {
                format!(
                    "Command execution failed: {} {}",
                    spec.cmd,
                    spec.args.join(" ")
                )
            });
        }

        // Return stdout as string
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

/// Dry-run command runner that prints commands but doesn't execute them
///
/// This runner prints what commands would be executed without actually running them.
/// It should be used in dry-run mode to show users what operations would be performed.
/// It also maintains a history of all commands for testing purposes.
#[derive(Debug)]
pub struct DryRunRunner {
    history: std::sync::Mutex<Vec<(CmdSpec, Option<String>)>>,
}

impl DryRunRunner {
    /// Create a new dry-run command runner
    pub fn new() -> Self {
        Self {
            history: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Get the history of all commands that would have been executed
    ///
    /// This is useful for testing to verify that the correct commands
    /// were generated without actually executing them.
    ///
    /// # Returns
    /// * `Vec<(CmdSpec, Option<String>)>` - List of all command specifications and their captured output (if any)
    pub fn history(&self) -> Vec<(CmdSpec, Option<String>)> {
        self.history.lock().unwrap().clone()
    }

    /// Clear the command history
    ///
    /// This is useful for testing when you want to reset between test cases.
    pub fn clear_history(&self) {
        self.history.lock().unwrap().clear();
    }

    /// Generate realistic fake output for a command
    ///
    /// This method returns appropriate fake output based on the command and arguments,
    /// allowing tests to verify that parsing logic works correctly in dry-run mode.
    fn generate_fake_output(&self, spec: &CmdSpec) -> String {
        match spec.cmd.as_str() {
            "garaga" => {
                // For garaga calldata commands, return JSON with calldata field
                if spec.args.contains(&"calldata".to_string()) {
                    r#"{"calldata": ["0x1234567890abcdef", "0xfedcba0987654321"]}"#.to_string()
                } else {
                    // For other garaga commands, return generic output
                    "Garaga operation completed successfully".to_string()
                }
            }
            "forge" => {
                // For forge create commands, return deployment info
                if spec.args.contains(&"create".to_string()) {
                    "Deployed to: 0x742d35Cc6634C0532925a3b8D400d1b0fB000000".to_string()
                } else {
                    // For other forge commands, return generic output
                    "Forge operation completed successfully".to_string()
                }
            }
            "cast" => {
                // For cast commands, return generic output
                "Cast operation completed successfully".to_string()
            }
            "bb" => {
                // For bb commands, return generic output
                "BB operation completed successfully".to_string()
            }
            "nargo" => {
                // For nargo commands, return generic output
                "Nargo operation completed successfully".to_string()
            }
            _ => {
                // For other commands, return generic output
                format!("{} operation completed successfully", spec.cmd)
            }
        }
    }
}

impl Default for DryRunRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl Runner for DryRunRunner {
    /// Print a command specification without executing it
    ///
    /// This method formats and prints the command that would be executed,
    /// including working directory and environment variables if specified.
    ///
    /// # Arguments
    /// * `spec` - Command specification to print
    ///
    /// # Returns
    /// * `Result<()>` - Always succeeds unless there's a formatting error
    fn run(&self, spec: &CmdSpec) -> Result<()> {
        // Record command in history with no captured output
        self.history.lock().unwrap().push((spec.clone(), None));

        // Build the command string
        let mut cmd_parts = vec![spec.cmd.clone()];
        cmd_parts.extend(spec.args.iter().cloned());
        let cmd_str = cmd_parts.join(" ");

        // Print environment variables if any
        if !spec.env.is_empty() {
            let env_str = spec
                .env
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(" ");
            print!("{} ", env_str);
        }

        // Print working directory if specified
        if let Some(ref cwd) = spec.cwd {
            println!("Would run in directory '{}': {}", cwd.display(), cmd_str);
        } else {
            println!("Would run: {}", cmd_str);
        }

        Ok(())
    }

    fn run_capture(&self, spec: &CmdSpec) -> Result<String> {
        // Generate realistic fake output
        let fake_output = self.generate_fake_output(spec);

        // Record command in history with captured output
        self.history
            .lock()
            .unwrap()
            .push((spec.clone(), Some(fake_output.clone())));

        // Build the command string for display
        let mut cmd_parts = vec![spec.cmd.clone()];
        cmd_parts.extend(spec.args.iter().cloned());
        let cmd_str = cmd_parts.join(" ");

        // Print what would be captured
        if let Some(ref cwd) = spec.cwd {
            println!(
                "Would run in directory '{}' (capturing output): {}",
                cwd.display(),
                cmd_str
            );
        } else {
            println!("Would run (capturing output): {}", cmd_str);
        }

        // Return realistic fake output
        Ok(fake_output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_cmd_spec_new() {
        let spec = CmdSpec::new("echo".to_string(), vec!["hello".to_string()]);
        assert_eq!(spec.cmd, "echo");
        assert_eq!(spec.args, vec!["hello"]);
        assert!(spec.cwd.is_none());
        assert!(spec.env.is_empty());
    }

    #[test]
    fn test_cmd_spec_with_cwd() {
        let spec = CmdSpec::new("echo".to_string(), vec!["hello".to_string()])
            .with_cwd(PathBuf::from("/tmp"));
        assert_eq!(spec.cwd, Some(PathBuf::from("/tmp")));
    }

    #[test]
    fn test_cmd_spec_with_env() {
        let spec = CmdSpec::new("echo".to_string(), vec!["hello".to_string()])
            .with_env("TEST".to_string(), "value".to_string());
        assert_eq!(spec.env, vec![("TEST".to_string(), "value".to_string())]);
    }

    #[test]
    fn test_cmd_spec_with_envs() {
        let env_vars = vec![
            ("VAR1".to_string(), "val1".to_string()),
            ("VAR2".to_string(), "val2".to_string()),
        ];
        let spec =
            CmdSpec::new("echo".to_string(), vec!["hello".to_string()]).with_envs(env_vars.clone());
        assert_eq!(spec.env, env_vars);
    }

    #[test]
    fn test_dry_run_runner_simple_command() {
        let runner = DryRunRunner::new();
        let spec = CmdSpec::new("echo".to_string(), vec!["hello".to_string()]);

        // This should not panic and should succeed
        assert!(runner.run(&spec).is_ok());
    }

    #[test]
    fn test_dry_run_runner_with_cwd() {
        let runner = DryRunRunner::new();
        let spec = CmdSpec::new("echo".to_string(), vec!["hello".to_string()])
            .with_cwd(PathBuf::from("/tmp"));

        // This should not panic and should succeed
        assert!(runner.run(&spec).is_ok());
    }

    #[test]
    fn test_dry_run_runner_with_env() {
        let runner = DryRunRunner::new();
        let spec = CmdSpec::new("echo".to_string(), vec!["hello".to_string()])
            .with_env("TEST".to_string(), "value".to_string());

        // This should not panic and should succeed
        assert!(runner.run(&spec).is_ok());
    }

    #[test]
    fn test_real_runner_echo_command() {
        let runner = RealRunner::new();
        let spec = CmdSpec::new("echo".to_string(), vec!["hello".to_string()]);

        // Echo should always be available and succeed
        assert!(runner.run(&spec).is_ok());
    }

    #[test]
    fn test_real_runner_invalid_command() {
        let runner = RealRunner::new();
        let spec = CmdSpec::new("this_command_does_not_exist_12345".to_string(), vec![]);

        // This should fail
        assert!(runner.run(&spec).is_err());
    }

    #[test]
    fn test_real_runner_run_capture() {
        let runner = RealRunner::new();
        let spec = CmdSpec::new("echo".to_string(), vec!["hello world".to_string()]);

        // Echo should capture output successfully
        let result = runner.run_capture(&spec);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("hello world"));
    }

    #[test]
    fn test_dry_run_runner_history() {
        let runner = DryRunRunner::new();

        // Initially empty
        assert_eq!(runner.history().len(), 0);

        // Run a few commands
        let spec1 = CmdSpec::new("echo".to_string(), vec!["test1".to_string()]);
        let spec2 = CmdSpec::new("ls".to_string(), vec!["-la".to_string()]);

        runner.run(&spec1).unwrap();
        runner.run(&spec2).unwrap();

        // History should contain both commands
        let history = runner.history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].0.cmd, "echo");
        assert_eq!(history[0].0.args, vec!["test1"]);
        assert_eq!(history[0].1, None); // No captured output for run()
        assert_eq!(history[1].0.cmd, "ls");
        assert_eq!(history[1].0.args, vec!["-la"]);
        assert_eq!(history[1].1, None); // No captured output for run()
    }

    #[test]
    fn test_dry_run_runner_clear_history() {
        let runner = DryRunRunner::new();

        // Add a command
        let spec = CmdSpec::new("echo".to_string(), vec!["test".to_string()]);
        runner.run(&spec).unwrap();
        assert_eq!(runner.history().len(), 1);

        // Clear history
        runner.clear_history();
        assert_eq!(runner.history().len(), 0);
    }

    #[test]
    fn test_dry_run_runner_run_capture() {
        let runner = DryRunRunner::new();
        let spec = CmdSpec::new("echo".to_string(), vec!["test output".to_string()]);

        // Should return realistic fake output
        let result = runner.run_capture(&spec);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output, "echo operation completed successfully");

        // Should record in history with captured output
        let history = runner.history();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].0.cmd, "echo");
        assert_eq!(history[0].0.args, vec!["test output"]);
        assert_eq!(
            history[0].1,
            Some("echo operation completed successfully".to_string())
        );
    }

    #[test]
    fn test_dry_run_runner_mixed_run_and_capture() {
        let runner = DryRunRunner::new();

        let spec1 = CmdSpec::new("echo".to_string(), vec!["normal".to_string()]);
        let spec2 = CmdSpec::new("cat".to_string(), vec!["file.txt".to_string()]);

        // Mix normal run and capture
        runner.run(&spec1).unwrap();
        runner.run_capture(&spec2).unwrap();

        // Both should be in history
        let history = runner.history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].0.cmd, "echo");
        assert_eq!(history[0].1, None); // No captured output for run()
        assert_eq!(history[1].0.cmd, "cat");
        assert_eq!(
            history[1].1,
            Some("cat operation completed successfully".to_string())
        );
    }

    #[test]
    fn test_dry_run_runner_garaga_calldata_fake_output() {
        let runner = DryRunRunner::new();
        let spec = CmdSpec::new(
            "garaga".to_string(),
            vec![
                "calldata".to_string(),
                "--system".to_string(),
                "ultra_starknet_zk_honk".to_string(),
            ],
        );

        let result = runner.run_capture(&spec);
        assert!(result.is_ok());
        let output = result.unwrap();

        // Should return JSON with calldata field
        assert!(output.contains("calldata"));
        assert!(output.contains("0x1234567890abcdef"));

        // Should be valid JSON
        let parsed: serde_json::Value =
            serde_json::from_str(&output).expect("Should be valid JSON");
        assert!(parsed["calldata"].is_array());
    }

    #[test]
    fn test_dry_run_runner_forge_create_fake_output() {
        let runner = DryRunRunner::new();
        let spec = CmdSpec::new(
            "forge".to_string(),
            vec![
                "create".to_string(),
                "MyContract.sol:MyContract".to_string(),
            ],
        );

        let result = runner.run_capture(&spec);
        assert!(result.is_ok());
        let output = result.unwrap();

        // Should return deployment info
        assert!(output.contains("Deployed to:"));
        assert!(output.contains("0x742d35Cc6634C0532925a3b8D400d1b0fB000000"));
    }
}
