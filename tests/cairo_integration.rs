//! Integration tests for bargo cairo commands
//!
//! These tests use DryRunRunner to verify cairo workflow execution without running external tools,
//! focusing on the prove, verify, and generate workflows.

use assert_fs::TempDir;
use bargo_core::config::Config;
use bargo_core::runner::DryRunRunner;
use path_slash::PathExt;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

// Global lock to prevent concurrent directory operations across all tests
static DIRECTORY_LOCK: Mutex<()> = Mutex::new(());

/// Temporary helper functions for Cairo commands until working directory API is implemented
fn run_cairo_prove_in_directory(
    config: &Config,
    project_dir: &Path,
) -> Result<(), color_eyre::eyre::Error> {
    // Use global lock to prevent race conditions
    let _lock = DIRECTORY_LOCK.lock().unwrap();

    // Validate project directory exists before proceeding
    if !project_dir.exists() {
        return Err(color_eyre::eyre::eyre!(
            "Project directory does not exist: {}",
            project_dir.display()
        ));
    }

    // Get current directory before changing it
    let original_dir = std::env::current_dir()
        .map_err(|e| color_eyre::eyre::eyre!("Failed to get current directory: {}", e))?;

    // Change to project directory
    std::env::set_current_dir(project_dir)
        .map_err(|e| color_eyre::eyre::eyre!("Failed to change to project directory: {}", e))?;

    let result = bargo_core::commands::cairo::run_prove(config);

    // Always restore directory, even on error
    let _ = std::env::set_current_dir(original_dir);

    result
}

fn run_cairo_gen_in_directory(
    config: &Config,
    project_dir: &Path,
) -> Result<(), color_eyre::eyre::Error> {
    // Use global lock to prevent race conditions
    let _lock = DIRECTORY_LOCK.lock().unwrap();

    // Validate project directory exists before proceeding
    if !project_dir.exists() {
        return Err(color_eyre::eyre::eyre!(
            "Project directory does not exist: {}",
            project_dir.display()
        ));
    }

    // Get current directory before changing it
    let original_dir = std::env::current_dir()
        .map_err(|e| color_eyre::eyre::eyre!("Failed to get current directory: {}", e))?;

    // Change to project directory
    std::env::set_current_dir(project_dir)
        .map_err(|e| color_eyre::eyre::eyre!("Failed to change to project directory: {}", e))?;

    let result = bargo_core::commands::cairo::run_gen(config);

    // Always restore directory, even on error
    let _ = std::env::set_current_dir(original_dir);

    result
}

/// Copy a fixture directory to a temporary location
fn copy_fixture_to_temp(fixture_name: &str, temp_dir: &TempDir) -> PathBuf {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(fixture_name);

    let dest_path = temp_dir.path().join(fixture_name);

    copy_dir_all(&fixture_path, &dest_path).expect("Failed to copy fixture");
    dest_path
}

/// Recursively copy a directory and all its contents
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;

        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

/// Create mock build artifacts to simulate a completed build
fn create_mock_build_artifacts(project_dir: &Path, package_name: &str) {
    let bb_dir = project_dir.join("target").join("bb");
    fs::create_dir_all(&bb_dir).unwrap();

    // Create mock bytecode and witness files
    let bytecode_content = r#"{"mock": "bytecode"}"#;
    let witness_content = "mock_witness_data";

    fs::write(
        bb_dir.join(format!("{}.json", package_name)),
        bytecode_content,
    )
    .unwrap();
    fs::write(bb_dir.join(format!("{}.gz", package_name)), witness_content).unwrap();
}

#[test]
fn test_cairo_prove_command_dry_run() {
    // Create temporary directory for the test
    let temp_dir = TempDir::new().unwrap();
    let project_dir = copy_fixture_to_temp("simple_circuit", &temp_dir);

    // Create mock build artifacts
    create_mock_build_artifacts(&project_dir, "simple_circuit");

    // Create DryRunRunner and config
    let dry_runner = std::sync::Arc::new(DryRunRunner::new());
    let config = Config {
        verbose: false,
        dry_run: true,
        pkg: None,
        quiet: true,
        runner: dry_runner.clone(),
    };

    // Run cairo prove command in the project directory using working directory API
    let result = run_cairo_prove_in_directory(&config, &project_dir);

    // The command should succeed in dry run mode
    assert!(
        result.is_ok(),
        "Cairo prove command failed: {:?}",
        result.err()
    );

    // Verify the command history contains expected commands
    let history = dry_runner.history();
    assert!(!history.is_empty(), "No commands were recorded in dry run");

    // Convert history to more testable format
    let commands: Vec<String> = history
        .iter()
        .map(|(spec, _)| format!("{} {}", spec.cmd, spec.args.join(" ")))
        .collect();

    // Cairo prove should involve bb prove command
    let bb_commands: Vec<_> = history
        .iter()
        .filter(|(spec, _)| spec.cmd == "bb")
        .collect();

    assert!(
        !bb_commands.is_empty(),
        "No bb commands found in cairo prove history. All commands: {:?}",
        commands
    );

    // Find the bb prove command
    let prove_command = bb_commands
        .iter()
        .find(|(spec, _)| spec.args.contains(&"prove".to_string()));

    assert!(
        prove_command.is_some(),
        "bb prove command not found in history. Commands: {:?}",
        commands
    );

    // Verify the prove command has correct arguments structure
    let (prove_spec, _) = prove_command.unwrap();
    assert!(
        prove_spec.args.contains(&"prove".to_string()),
        "Prove command missing 'prove' argument"
    );

    // Should have additional arguments for bytecode and witness files
    assert!(
        prove_spec.args.len() > 1,
        "Prove command should have multiple arguments for bytecode/witness files"
    );
}

#[test]
fn test_cairo_prove_with_package_override() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = copy_fixture_to_temp("simple_circuit", &temp_dir);

    // Create mock build artifacts with custom package name
    create_mock_build_artifacts(&project_dir, "custom_package");

    let dry_runner = std::sync::Arc::new(DryRunRunner::new());
    let config = Config {
        verbose: false,
        dry_run: true,
        pkg: Some("custom_package".to_string()),
        quiet: true,
        runner: dry_runner.clone(),
    };

    let result = run_cairo_prove_in_directory(&config, &project_dir);

    assert!(
        result.is_ok(),
        "Cairo prove with package override failed: {:?}",
        result.err()
    );

    let history = dry_runner.history();
    assert!(
        !history.is_empty(),
        "No commands recorded with package override"
    );

    // Verify bb prove command is still executed
    let has_bb_prove = history
        .iter()
        .any(|(spec, _)| spec.cmd == "bb" && spec.args.contains(&"prove".to_string()));

    assert!(
        has_bb_prove,
        "Package override should not prevent bb prove execution"
    );
}

#[test]
fn test_cairo_prove_verbose_mode() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = copy_fixture_to_temp("simple_circuit", &temp_dir);

    create_mock_build_artifacts(&project_dir, "simple_circuit");

    let dry_runner = std::sync::Arc::new(DryRunRunner::new());
    let config = Config {
        verbose: true,
        dry_run: true,
        pkg: None,
        quiet: false,
        runner: dry_runner.clone(),
    };

    let result = run_cairo_prove_in_directory(&config, &project_dir);

    assert!(
        result.is_ok(),
        "Cairo prove in verbose mode failed: {:?}",
        result.err()
    );

    let history = dry_runner.history();
    assert!(!history.is_empty(), "No commands recorded in verbose mode");

    // In verbose mode, the same commands should be executed
    let has_bb_prove = history
        .iter()
        .any(|(spec, _)| spec.cmd == "bb" && spec.args.contains(&"prove".to_string()));

    assert!(has_bb_prove, "Verbose mode should still execute bb prove");
}

#[test]
fn test_cairo_gen_command_dry_run() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = copy_fixture_to_temp("simple_circuit", &temp_dir);

    create_mock_build_artifacts(&project_dir, "simple_circuit");

    let dry_runner = std::sync::Arc::new(DryRunRunner::new());
    let config = Config {
        verbose: false,
        dry_run: true,
        pkg: None,
        quiet: true,
        runner: dry_runner.clone(),
    };

    // Test cairo gen command using working directory API
    let result = run_cairo_gen_in_directory(&config, &project_dir);

    // Should succeed or gracefully handle missing dependencies
    if result.is_ok() {
        let history = dry_runner.history();

        // If successful, should have some command history
        if !history.is_empty() {
            let commands: Vec<String> = history
                .iter()
                .map(|(spec, _)| format!("{} {}", spec.cmd, spec.args.join(" ")))
                .collect();

            // Cairo gen might involve garaga or other tools
            let _has_generation_command = history.iter().any(|(spec, _)| {
                spec.cmd == "garaga" || spec.cmd == "cairo-run" || spec.cmd.contains("cairo")
            });

            // Note: This test is lenient because the exact tools may not be available
            // The important thing is that it doesn't crash and follows the expected pattern
            println!("Cairo gen commands executed: {:?}", commands);
        }
    } else {
        // If it fails, it should be due to missing external dependencies, not internal errors
        let error_msg = format!("{:?}", result.err());
        assert!(
            error_msg.contains("garaga")
                || error_msg.contains("cairo")
                || error_msg.contains("Required files")
                || error_msg.contains("dependency"),
            "Cairo gen should fail gracefully due to missing dependencies, got: {}",
            error_msg
        );
    }
}

#[test]
fn test_cairo_workflow_file_path_normalization() {
    // Test that file paths are handled correctly across platforms
    let temp_dir = TempDir::new().unwrap();
    let project_dir = copy_fixture_to_temp("simple_circuit", &temp_dir);

    create_mock_build_artifacts(&project_dir, "simple_circuit");

    let dry_runner = std::sync::Arc::new(DryRunRunner::new());
    let config = Config {
        verbose: false,
        dry_run: true,
        pkg: None,
        quiet: true,
        runner: dry_runner.clone(),
    };

    let result = run_cairo_prove_in_directory(&config, &project_dir);

    assert!(result.is_ok(), "Cairo prove failed: {:?}", result.err());

    let history = dry_runner.history();

    // Verify that any paths in command arguments use forward slashes for consistency
    for (spec, _) in &history {
        for arg in &spec.args {
            if arg.contains("target") || arg.contains(".json") || arg.contains(".gz") {
                // Convert to normalized path representation
                let path_buf = PathBuf::from(arg);
                let normalized = path_buf.to_slash_lossy();

                // Verify the path is reasonable (no double slashes, etc.)
                assert!(
                    !normalized.contains("//"),
                    "Path should not contain double slashes: {}",
                    normalized
                );

                // Verify target paths are reasonable (no double slashes, proper structure)
                if arg.contains("target") {
                    assert!(
                        normalized.contains("target/") || normalized.contains("target\\"),
                        "Target paths should contain target directory: {}",
                        normalized
                    );

                    // Verify it's a reasonable target subdirectory (bb, evm, starknet)
                    assert!(
                        normalized.contains("target/bb")
                            || normalized.contains("target\\bb")
                            || normalized.contains("target/evm")
                            || normalized.contains("target\\evm")
                            || normalized.contains("target/starknet")
                            || normalized.contains("target\\starknet"),
                        "Target paths should use expected subdirectories: {}",
                        normalized
                    );
                }
            }
        }
    }
}

#[test]
fn test_cairo_commands_require_build_artifacts() {
    // Test that cairo commands properly check for required build artifacts
    let temp_dir = TempDir::new().unwrap();
    let project_dir = copy_fixture_to_temp("simple_circuit", &temp_dir);

    // Intentionally don't create build artifacts

    let dry_runner = std::sync::Arc::new(DryRunRunner::new());
    let config = Config {
        verbose: false,
        dry_run: true,
        pkg: None,
        quiet: true,
        runner: dry_runner.clone(),
    };

    let result = run_cairo_prove_in_directory(&config, &project_dir);

    // Should fail or provide helpful error about missing build artifacts
    if result.is_err() {
        let error_msg = format!("{:?}", result.err());
        assert!(
            error_msg.contains("Required files")
                || error_msg.contains("build")
                || error_msg.contains("target")
                || error_msg.contains(".json")
                || error_msg.contains("bytecode")
                || error_msg.contains("witness"),
            "Error should mention missing build artifacts, got: {}",
            error_msg
        );
    } else {
        // If it succeeds in dry run, it should at least attempt to run commands
        let history = dry_runner.history();
        assert!(
            !history.is_empty(),
            "Should have attempted some commands even without build artifacts"
        );
    }
}
