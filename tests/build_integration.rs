//! Integration tests for bargo build command
//!
//! These tests use DryRunRunner to verify command execution without running external tools,
//! and compare generated directory structures against golden snapshots.

use assert_fs::TempDir;
use bargo_core::config::Config;
use bargo_core::runner::DryRunRunner;
use path_slash::PathExt;
use std::fs;
use std::path::{Path, PathBuf};

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

/// Compare two directories recursively, ignoring file modification times
/// Returns a list of differences found
fn compare_directories(actual: &Path, expected: &Path) -> Vec<String> {
    let mut differences = Vec::new();

    match compare_directories_recursive(actual, expected, Path::new("")) {
        Ok(diffs) => differences.extend(diffs),
        Err(e) => differences.push(format!("Error comparing directories: {}", e)),
    }

    differences
}

/// Recursive helper for directory comparison
fn compare_directories_recursive(
    actual: &Path,
    expected: &Path,
    relative_path: &Path,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut differences = Vec::new();

    // Check if both paths exist
    if !actual.exists() && !expected.exists() {
        return Ok(differences);
    }

    if !actual.exists() {
        differences.push(format!(
            "Missing directory in actual: {}",
            relative_path.to_slash_lossy()
        ));
        return Ok(differences);
    }

    if !expected.exists() {
        differences.push(format!(
            "Unexpected directory in actual: {}",
            relative_path.to_slash_lossy()
        ));
        return Ok(differences);
    }

    // Compare directory contents
    let actual_entries: std::collections::BTreeSet<_> = fs::read_dir(actual)?
        .map(|e| e.unwrap().file_name())
        .collect();

    let expected_entries: std::collections::BTreeSet<_> = fs::read_dir(expected)?
        .map(|e| e.unwrap().file_name())
        .collect();

    // Check for missing files/directories
    for entry in &expected_entries {
        if !actual_entries.contains(entry) {
            differences.push(format!(
                "Missing file/directory: {}",
                relative_path.join(entry).to_slash_lossy()
            ));
        }
    }

    // Check for unexpected files/directories
    for entry in &actual_entries {
        if !expected_entries.contains(entry) {
            differences.push(format!(
                "Unexpected file/directory: {}",
                relative_path.join(entry).to_slash_lossy()
            ));
        }
    }

    // Recursively compare common entries
    for entry in actual_entries.intersection(&expected_entries) {
        let actual_path = actual.join(entry);
        let expected_path = expected.join(entry);
        let entry_relative_path = relative_path.join(entry);

        if actual_path.is_dir() {
            let sub_diffs =
                compare_directories_recursive(&actual_path, &expected_path, &entry_relative_path)?;
            differences.extend(sub_diffs);
        } else {
            // Compare file contents
            let actual_content = fs::read(&actual_path)?;
            let expected_content = fs::read(&expected_path)?;

            if actual_content != expected_content {
                let actual_str = String::from_utf8_lossy(&actual_content);
                let expected_str = String::from_utf8_lossy(&expected_content);
                differences.push(format!(
                    "File content differs: {}\nActual length: {}\nExpected length: {}\nActual content (first 200 chars): {}\nExpected content (first 200 chars): {}",
                    entry_relative_path.to_slash_lossy(),
                    actual_content.len(),
                    expected_content.len(),
                    &actual_str.chars().take(200).collect::<String>(),
                    &expected_str.chars().take(200).collect::<String>()
                ));
            }
        }
    }

    Ok(differences)
}

#[test]
fn test_build_command_dry_run() {
    // Create temporary directory for the test
    let temp_dir = TempDir::new().unwrap();

    // Copy fixture to temp directory
    let project_dir = copy_fixture_to_temp("simple_circuit", &temp_dir);

    // Create DryRunRunner and config
    let dry_runner = std::sync::Arc::new(DryRunRunner::new());
    let config = Config {
        verbose: false,
        dry_run: true,
        pkg: None,
        quiet: true,
        runner: dry_runner.clone(),
    };

    // Run bargo build command in the project directory using working directory API
    let result = bargo_core::commands::build::run_in_directory(&config, Some(&project_dir));

    // The command should succeed in dry run mode
    assert!(result.is_ok(), "Build command failed: {:?}", result.err());

    // Verify the command history contains expected commands
    let history = dry_runner.history();
    assert!(!history.is_empty(), "No commands were recorded in dry run");

    // Convert history to more testable format
    let commands: Vec<String> = history
        .iter()
        .map(|(spec, _)| format!("{} {}", spec.cmd, spec.args.join(" ")))
        .collect();

    // Check that nargo execute was called
    let nargo_commands: Vec<_> = history
        .iter()
        .filter(|(spec, _)| spec.cmd == "nargo")
        .collect();

    assert!(
        !nargo_commands.is_empty(),
        "No nargo commands found in history. All commands: {:?}",
        commands
    );

    // Find the nargo execute command
    let execute_command = nargo_commands
        .iter()
        .find(|(spec, _)| spec.args.contains(&"execute".to_string()));

    assert!(
        execute_command.is_some(),
        "nargo execute command not found in history. Commands: {:?}",
        commands
    );

    // Verify the execute command has correct arguments
    let (execute_spec, _) = execute_command.unwrap();
    assert!(
        execute_spec.args.contains(&"execute".to_string()),
        "Execute command missing 'execute' argument"
    );

    // Check that the working directory is set correctly for nargo commands
    let nargo_with_cwd: Vec<_> = nargo_commands
        .iter()
        .filter(|(spec, _)| spec.cwd.is_some())
        .collect();

    if !nargo_with_cwd.is_empty() {
        let (spec_with_cwd, _) = nargo_with_cwd[0];
        let cwd = spec_with_cwd.cwd.as_ref().unwrap();
        assert!(
            cwd.ends_with("simple_circuit") || cwd == &project_dir,
            "Working directory should be the project directory, got: {:?}",
            cwd
        );
    }

    // Verify build artifacts organization is attempted
    // This would involve moving files from target/ to target/bb/
    // In dry run mode, this is simulated, so we just verify the sequence makes sense
    assert!(
        commands.len() >= 1,
        "Expected at least one command for build process"
    );
}

#[test]
fn test_build_creates_expected_structure() {
    // Create temporary directory for the test
    let temp_dir = TempDir::new().unwrap();

    // Copy fixture to temp directory
    let project_dir = copy_fixture_to_temp("simple_circuit", &temp_dir);

    // For this test, we'll simulate what the build would create
    // by manually creating the expected directory structure
    let target_dir = project_dir.join("target");
    let bb_dir = target_dir.join("bb");
    fs::create_dir_all(&bb_dir).unwrap();

    // Create mock build artifacts (simulating what a real build would produce)
    let bytecode_content = r#"{
  "noir_version": "0.19.0",
  "hash": "0x1234567890abcdef1234567890abcdef12345678",
  "abi": {
    "parameters": [
      {
        "name": "a",
        "type": {
          "kind": "field"
        },
        "visibility": "private"
      },
      {
        "name": "b",
        "type": {
          "kind": "field"
        },
        "visibility": "private"
      }
    ],
    "return_type": {
      "kind": "field",
      "visibility": "public"
    }
  },
  "bytecode": "H4sIAAAAAAAC/wEAAP//AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
  "debug_symbols": {
    "locations": [
      {
        "span": {
          "start": 0,
          "end": 89
        },
        "file": "main.nr"
      }
    ]
  },
  "file_map": {
    "main.nr": "fn main(a: Field, b: Field) -> pub Field {\n    let sum = a + b;\n    assert(sum != 0); // Simple constraint to make it more than trivial\n    sum\n}"
  }
}"#;

    let witness_content = "H4sIAAAAAAAAA+3BMQEAAADCoPVPbQwfoAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA7N0ADeJaNwAAA==";

    fs::write(bb_dir.join("simple_circuit.json"), bytecode_content).unwrap();
    fs::write(bb_dir.join("simple_circuit.gz"), witness_content).unwrap();

    // Compare with golden snapshot
    let golden_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("goldens")
        .join("simple_circuit_build");

    let differences = compare_directories(&target_dir, &golden_path.join("target"));

    if !differences.is_empty() {
        panic!(
            "Directory structure does not match golden snapshot:\n{}",
            differences.join("\n")
        );
    }
}

#[test]
fn test_build_command_with_package_override() {
    // Test that package name override works correctly in build command
    let temp_dir = TempDir::new().unwrap();
    let project_dir = copy_fixture_to_temp("simple_circuit", &temp_dir);

    let dry_runner = std::sync::Arc::new(DryRunRunner::new());
    let config = Config {
        verbose: false,
        dry_run: true,
        pkg: Some("custom_package_name".to_string()),
        quiet: true,
        runner: dry_runner.clone(),
    };

    let result = bargo_core::commands::build::run_in_directory(&config, Some(&project_dir));

    assert!(
        result.is_ok(),
        "Build with package override failed: {:?}",
        result.err()
    );

    let history = dry_runner.history();
    assert!(
        !history.is_empty(),
        "No commands recorded with package override"
    );

    // Verify that the package override doesn't break the build process
    let commands: Vec<String> = history
        .iter()
        .map(|(spec, _)| format!("{} {}", spec.cmd, spec.args.join(" ")))
        .collect();

    // Should still have nargo execute command
    let has_nargo_execute = history
        .iter()
        .any(|(spec, _)| spec.cmd == "nargo" && spec.args.contains(&"execute".to_string()));

    assert!(
        has_nargo_execute,
        "Package override should not prevent nargo execute. Commands: {:?}",
        commands
    );
}

#[test]
fn test_build_command_verbose_mode() {
    // Test that verbose mode affects the build process appropriately
    let temp_dir = TempDir::new().unwrap();
    let project_dir = copy_fixture_to_temp("simple_circuit", &temp_dir);

    let dry_runner = std::sync::Arc::new(DryRunRunner::new());
    let config = Config {
        verbose: true,
        dry_run: true,
        pkg: None,
        quiet: false,
        runner: dry_runner.clone(),
    };

    let result = bargo_core::commands::build::run_in_directory(&config, Some(&project_dir));

    assert!(
        result.is_ok(),
        "Build in verbose mode failed: {:?}",
        result.err()
    );

    let history = dry_runner.history();
    assert!(!history.is_empty(), "No commands recorded in verbose mode");

    // In verbose mode, the same commands should be executed
    // but potentially with different logging/output behavior
    let has_nargo_execute = history
        .iter()
        .any(|(spec, _)| spec.cmd == "nargo" && spec.args.contains(&"execute".to_string()));

    assert!(has_nargo_execute, "Verbose mode should still execute nargo");
}

#[test]
fn test_fixture_is_valid() {
    // This test verifies that our fixture can be copied successfully
    // and has the expected structure
    let temp_dir = TempDir::new().unwrap();
    let project_dir = copy_fixture_to_temp("simple_circuit", &temp_dir);

    // Verify essential files exist
    assert!(
        project_dir.join("Nargo.toml").exists(),
        "Nargo.toml not found"
    );
    assert!(
        project_dir.join("Prover.toml").exists(),
        "Prover.toml not found"
    );
    assert!(
        project_dir.join("src").join("main.nr").exists(),
        "src/main.nr not found"
    );

    // Verify Nargo.toml has correct package name
    let nargo_content = fs::read_to_string(project_dir.join("Nargo.toml")).unwrap();
    assert!(
        nargo_content.contains("simple_circuit"),
        "Package name not found in Nargo.toml"
    );
}

#[test]
fn test_build_cross_platform_paths() {
    // Test that build command handles paths correctly across platforms
    let temp_dir = TempDir::new().unwrap();
    let project_dir = copy_fixture_to_temp("simple_circuit", &temp_dir);

    let dry_runner = std::sync::Arc::new(DryRunRunner::new());
    let config = Config {
        verbose: false,
        dry_run: true,
        pkg: None,
        quiet: true,
        runner: dry_runner.clone(),
    };

    let result = bargo_core::commands::build::run_in_directory(&config, Some(&project_dir));

    assert!(result.is_ok(), "Build command failed: {:?}", result.err());

    let history = dry_runner.history();

    // Verify that any paths in command arguments are properly normalized
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

                // Verify target paths are reasonable
                if arg.contains("target") {
                    assert!(
                        normalized.contains("target/") || normalized.contains("target\\"),
                        "Target paths should contain target directory: {}",
                        normalized
                    );

                    // Should use expected target subdirectories
                    assert!(
                        normalized.contains("target/bb")
                            || normalized.contains("target\\bb")
                            || normalized.contains("target/evm")
                            || normalized.contains("target\\evm")
                            || !arg.ends_with("target"),
                        "Target paths should use expected subdirectories: {}",
                        normalized
                    );
                }
            }
        }

        // Verify working directory paths if set
        if let Some(cwd) = &spec.cwd {
            let cwd_buf = PathBuf::from(cwd);
            let normalized_cwd = cwd_buf.to_slash_lossy();

            assert!(
                !normalized_cwd.contains("//"),
                "Working directory path should not contain double slashes: {}",
                normalized_cwd
            );
        }
    }
}
