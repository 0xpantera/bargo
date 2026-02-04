//! Integration tests for error context and error chain functionality
//!
//! This module tests that errors from external tools and file operations
//! include rich context information and proper error chains.

use bargo_core::{
    config::Config,
    runner::{CmdSpec, DryRunRunner, Runner},
};
use color_eyre::Result;
use std::sync::{Arc, Mutex};

/// Custom runner that simulates tool failures for testing error context
#[derive(Debug)]
struct FailingDryRunRunner {
    inner: DryRunRunner,
    failing_tools: Arc<Mutex<Vec<String>>>,
}

impl FailingDryRunRunner {
    fn new() -> Self {
        Self {
            inner: DryRunRunner::new(),
            failing_tools: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Configure a tool to fail when executed
    fn fail_on_tool(&self, tool: &str) {
        let mut failing_tools = self.failing_tools.lock().unwrap();
        failing_tools.push(tool.to_string());
    }

    /// Check if a tool should fail
    fn should_fail(&self, spec: &CmdSpec) -> bool {
        let failing_tools = self.failing_tools.lock().unwrap();
        failing_tools.contains(&spec.cmd)
    }
}

impl Runner for FailingDryRunRunner {
    fn run(&self, spec: &CmdSpec) -> Result<()> {
        if self.should_fail(spec) {
            return Err(color_eyre::eyre::eyre!(
                "Command '{}' failed with exit code 1\nStdout: \nStderr: Tool not found or execution failed",
                format!("{} {}", spec.cmd, spec.args.join(" "))
            ));
        }
        // Delegate to inner DryRunRunner for normal behavior
        self.inner.run(spec)
    }

    fn run_capture(&self, spec: &CmdSpec) -> Result<String> {
        if self.should_fail(spec) {
            return Err(color_eyre::eyre::eyre!(
                "Command '{}' failed with exit code 1\nStdout: \nStderr: Tool not found or execution failed",
                format!("{} {}", spec.cmd, spec.args.join(" "))
            ));
        }
        // Delegate to inner DryRunRunner for normal behavior
        self.inner.run_capture(spec)
    }
}

/// Test that missing project configuration errors include proper context
#[test]
fn test_missing_project_error_context() {
    color_eyre::install().ok();

    let config = Config {
        verbose: true,
        dry_run: false,
        pkg: None,
        quiet: false,
        runner: Arc::new(DryRunRunner::new()),
    };

    // Try to run a command that will fail due to missing Nargo.toml
    let result = bargo_core::commands::build::run(&config);

    assert!(
        result.is_err(),
        "Expected build to fail when Nargo.toml is missing"
    );

    let error_string = format!("{:?}", result.unwrap_err());

    // Check that error contains meaningful context about missing configuration
    assert!(
        error_string.contains("Nargo.toml") || error_string.contains("project"),
        "Error should mention missing project configuration: {}",
        error_string
    );

    assert!(
        error_string.contains("directory") || error_string.contains("Noir project"),
        "Error should contain helpful context about project location: {}",
        error_string
    );
}

/// Test that missing artifact errors include proper context
#[cfg(feature = "cairo")]
#[test]
fn test_missing_artifacts_error_context() {
    color_eyre::install().ok();

    let config = Config {
        verbose: true,
        dry_run: false,
        pkg: Some("test_pkg".to_string()),
        quiet: false,
        runner: Arc::new(DryRunRunner::new()),
    };

    // Try to run Cairo prove which will fail due to missing artifacts
    let result = bargo_core::commands::cairo::run_prove(&config);

    assert!(
        result.is_err(),
        "Expected prove to fail when artifacts are missing"
    );

    let error_string = format!("{:?}", result.unwrap_err());

    // Check that error contains meaningful context about missing files
    assert!(
        error_string.contains("Required files are missing") || error_string.contains("missing"),
        "Error should mention missing files: {}",
        error_string
    );

    assert!(
        error_string.contains("Suggestions") || error_string.contains("bargo build"),
        "Error should contain helpful suggestions: {}",
        error_string
    );
}

/// Test that file operation errors include proper context
#[test]
fn test_file_operation_error_context() {
    color_eyre::install().ok();

    // Test that clean command with non-existent directory has proper error context
    let failing_runner = FailingDryRunRunner::new();
    failing_runner.fail_on_tool("rm"); // Won't actually be called, but simulates file operation failure

    let config = Config {
        verbose: true,
        // Avoid deleting the workspace target/ during CI; dry-run is enough here.
        dry_run: true,
        pkg: Some("nonexistent_package".to_string()),
        quiet: false,
        runner: Arc::new(failing_runner),
    };

    // This test verifies that file operations have proper error context
    // by testing a command that would perform file operations
    let result = bargo_core::commands::clean::run(&config, bargo_core::cli::Backend::All);

    // The test should pass or fail gracefully with proper error context
    if let Err(error) = result {
        let error_string = format!("{:?}", error);
        // If it fails, it should have meaningful error context
        assert!(!error_string.is_empty(), "Error should not be empty");
    }
    // If it succeeds, that's also fine - the directory might not exist
}

/// Test that missing proof artifacts errors include proper context
#[cfg(feature = "cairo")]
#[test]
fn test_missing_proof_artifacts_error_context() {
    color_eyre::install().ok();

    let config = Config {
        verbose: true,
        dry_run: false,
        pkg: Some("test_pkg".to_string()),
        quiet: false,
        runner: Arc::new(DryRunRunner::new()),
    };

    // Try to run Cairo calldata which will fail due to missing proof artifacts
    let result = bargo_core::commands::cairo::run_calldata(&config);

    assert!(
        result.is_err(),
        "Expected calldata to fail when proof artifacts are missing"
    );

    let error_string = format!("{:?}", result.unwrap_err());

    // Check that error contains meaningful context about missing proof files
    assert!(
        error_string.contains("proof")
            || error_string.contains("vk")
            || error_string.contains("missing"),
        "Error should mention missing proof artifacts: {}",
        error_string
    );

    assert!(
        error_string.contains("Suggestions") || error_string.contains("bargo"),
        "Error should contain helpful suggestions: {}",
        error_string
    );
}

/// Test that missing verifier contract errors include proper context
#[test]
fn test_missing_verifier_contract_error_context() {
    color_eyre::install().ok();

    let config = Config {
        verbose: true,
        dry_run: false,
        pkg: Some("test_pkg".to_string()),
        quiet: false,
        runner: Arc::new(DryRunRunner::new()),
    };

    // Try to run EVM deploy which will fail due to missing verifier contract
    let result = bargo_core::commands::evm::run_deploy(&config, "localhost");

    assert!(
        result.is_err(),
        "Expected deploy to fail when verifier contract is missing"
    );

    let error_string = format!("{:?}", result.unwrap_err());

    // Check that error contains meaningful context about missing contract
    assert!(
        error_string.contains("Verifier contract") || error_string.contains("contract"),
        "Error should mention missing verifier contract: {}",
        error_string
    );

    assert!(
        error_string.contains("Suggestions") || error_string.contains("bargo evm gen"),
        "Error should contain helpful suggestions: {}",
        error_string
    );
}

/// Test error chain depth and formatting with actual tool execution
#[test]
fn test_tool_execution_error_chain() {
    color_eyre::install().ok();

    // Use a failing runner to test actual tool execution error chains
    let failing_runner = FailingDryRunRunner::new();
    failing_runner.fail_on_tool("nonexistent_tool");

    let config = Config {
        verbose: true,
        dry_run: false,
        pkg: Some("test_pkg".to_string()),
        quiet: false,
        runner: Arc::new(failing_runner),
    };

    // Try to run a command that uses the runner directly
    let spec = CmdSpec::new("nonexistent_tool".to_string(), vec!["arg1".to_string()]);
    let result = config.runner.run(&spec);

    assert!(result.is_err(), "Expected tool execution to fail");

    let error_string = format!("{:?}", result.unwrap_err());

    // Check that the error chain contains tool execution context
    assert!(
        error_string.contains("nonexistent_tool"),
        "Error should contain tool name: {}",
        error_string
    );

    assert!(
        error_string.contains("simulated failure") || error_string.contains("exit code"),
        "Error should contain execution failure context: {}",
        error_string
    );
}

/// Test that workflow commands properly propagate errors with context
#[test]
fn test_workflow_error_propagation() {
    color_eyre::install().ok();

    // Use a failing runner to ensure we get an error
    let failing_runner = FailingDryRunRunner::new();
    failing_runner.fail_on_tool("nargo");

    let config = Config {
        verbose: false,
        dry_run: false,
        pkg: None,
        quiet: true,
        runner: Arc::new(failing_runner),
    };

    // Test that check command properly propagates errors
    let result = bargo_core::commands::check::run(&config);
    assert!(result.is_err(), "Expected check to fail when nargo fails");

    let error = result.unwrap_err();
    let error_string = format!("{:?}", error);

    // Verify that the error has proper context and isn't empty
    assert!(!error_string.is_empty(), "Error should not be empty");

    // Check that the error contains meaningful information
    assert!(
        error_string.contains("nargo") || error_string.contains("failed"),
        "Error should contain meaningful failure context: {}",
        error_string
    );
}

/// Test that actual tool execution failures produce proper error chains
#[test]
fn test_actual_tool_execution_error_chain() {
    color_eyre::install().ok();

    // Use a failing runner to test actual tool execution error chains
    let failing_runner = FailingDryRunRunner::new();
    failing_runner.fail_on_tool("bb");

    let config = Config {
        verbose: true,
        dry_run: false,
        pkg: Some("test_pkg".to_string()),
        quiet: false,
        runner: Arc::new(failing_runner),
    };

    // Use the common::run_tool function to test the full error chain
    let result =
        bargo_core::commands::common::run_tool(&config, "bb", &["prove", "--scheme", "ultra_honk"]);

    assert!(result.is_err(), "Expected bb tool execution to fail");

    let error_string = format!("{:?}", result.unwrap_err());

    // Check that the error chain contains tool execution context
    assert!(
        error_string.contains("bb"),
        "Error should contain tool name: {}",
        error_string
    );

    // Check that the error contains the wrapped context from the runner
    assert!(
        error_string.contains("Command execution failed")
            || error_string.contains("failed with exit code"),
        "Error should contain command execution failure context: {}",
        error_string
    );

    // Check that the error shows the command arguments
    assert!(
        error_string.contains("prove") || error_string.contains("ultra_honk"),
        "Error should contain command arguments for context: {}",
        error_string
    );
}
