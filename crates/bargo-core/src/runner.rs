//! Command execution abstraction for bargo
//!
//! This module provides a unified interface for command execution that supports
//! both real execution and dry-run mode, making it easier to test commands and
//! provide user feedback about what operations would be performed.

use std::path::PathBuf;
use std::process::Command;

use color_eyre::Result;

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
        let output = cmd.output().map_err(|e| {
            color_eyre::eyre::eyre!("Failed to execute command '{}': {}", spec.cmd, e)
        })?;

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
            ));
        }

        // Print stdout if there's any output
        if !output.stdout.is_empty() {
            print!("{}", String::from_utf8_lossy(&output.stdout));
        }

        Ok(())
    }
}

/// Dry-run command runner that prints commands but doesn't execute them
///
/// This runner prints what commands would be executed without actually running them.
/// It should be used in dry-run mode to show users what operations would be performed.
#[derive(Debug)]
pub struct DryRunRunner;

impl DryRunRunner {
    /// Create a new dry-run command runner
    pub fn new() -> Self {
        Self
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
}
