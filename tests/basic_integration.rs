//! Basic integration tests for bargo CLI
//!
//! These tests verify essential functionality without complex setup
//! and avoid global state pollution that can cause test interference.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Get the path to the bargo project root (where Cargo.toml is located)
fn get_bargo_manifest_path() -> PathBuf {
    env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::current_dir().expect("Failed to get current directory"))
        .join("Cargo.toml")
}

/// Run a bargo command from any directory without project requirements
fn run_bargo_global(args: &[&str]) -> std::process::Output {
    Command::new("cargo")
        .args(["run", "--manifest-path"])
        .arg(get_bargo_manifest_path())
        .arg("--")
        .args(args)
        .output()
        .expect("Failed to execute bargo command")
}

/// Create a temporary project with basic Noir circuit setup
fn create_test_project() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_dir = temp_dir.path().join("test_circuit");

    // Create project directory
    fs::create_dir_all(&project_dir).expect("Failed to create project directory");

    // Create Nargo.toml
    let nargo_toml_content = r#"[package]
name = "test_circuit"
type = "bin"
authors = ["test"]
compiler_version = ">=0.19.0"

[dependencies]
"#;
    fs::write(project_dir.join("Nargo.toml"), nargo_toml_content)
        .expect("Failed to write Nargo.toml");

    // Create src/main.nr
    let src_dir = project_dir.join("src");
    fs::create_dir_all(&src_dir).expect("Failed to create src directory");

    let main_nr_content = r#"fn main(a: Field, b: Field) -> pub Field {
    let result = a + b;
    assert(result != 0);
    result
}"#;
    fs::write(src_dir.join("main.nr"), main_nr_content).expect("Failed to write main.nr");

    // Create Prover.toml
    let prover_toml_content = r#"a = "3"
b = "4"
"#;
    fs::write(project_dir.join("Prover.toml"), prover_toml_content)
        .expect("Failed to write Prover.toml");

    (temp_dir, project_dir)
}

/// Run a bargo command in a specific project directory
fn run_bargo_in_project(project_dir: &PathBuf, args: &[&str]) -> std::process::Output {
    Command::new("cargo")
        .args(["run", "--manifest-path"])
        .arg(get_bargo_manifest_path())
        .arg("--")
        .args(args)
        .current_dir(project_dir)
        .output()
        .expect("Failed to execute bargo command")
}

#[test]
fn test_bargo_help() {
    let output = run_bargo_global(&["--help"]);

    assert!(
        output.status.success(),
        "Help command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("bargo consolidates nargo"));
    assert!(stdout.contains("Commands:"));
    assert!(stdout.contains("build"));
    assert!(stdout.contains("cairo"));
    assert!(stdout.contains("evm"));
    assert!(stdout.contains("check"));
    assert!(stdout.contains("clean"));
    assert!(stdout.contains("doctor"));
}

#[test]
fn test_bargo_version() {
    let output = run_bargo_global(&["--version"]);

    assert!(
        output.status.success(),
        "Version command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("bargo"));
}

#[test]
fn test_bargo_doctor() {
    let output = run_bargo_global(&["doctor"]);

    assert!(
        output.status.success(),
        "Doctor command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("dependencies"));
    assert!(stdout.contains("nargo:"));
    assert!(stdout.contains("bb:"));

    // Should show either available (✅) or missing (❌) status
    assert!(
        stdout.contains("✅") || stdout.contains("❌"),
        "Doctor should show status indicators"
    );
}

#[test]
fn test_evm_help() {
    let output = run_bargo_global(&["evm", "--help"]);

    assert!(
        output.status.success(),
        "EVM help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Generate Solidity verifiers"));
    assert!(stdout.contains("prove"));
    assert!(stdout.contains("verify"));
    assert!(stdout.contains("gen"));
    assert!(stdout.contains("deploy"));
}

#[test]
fn test_cairo_help() {
    let output = run_bargo_global(&["cairo", "--help"]);

    assert!(
        output.status.success(),
        "Cairo help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Generate Cairo verifiers"));
    assert!(stdout.contains("prove"));
    assert!(stdout.contains("verify"));
    assert!(stdout.contains("gen"));
    assert!(stdout.contains("declare"));
    assert!(stdout.contains("deploy"));
}

#[test]
fn test_invalid_command() {
    let output = run_bargo_global(&["invalid-command"]);

    assert!(!output.status.success(), "Invalid command should fail");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error:") || stderr.contains("unrecognized"),
        "Should show error for invalid command"
    );
}

#[test]
fn test_missing_project_error() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let empty_dir = temp_dir.path().join("empty");
    fs::create_dir_all(&empty_dir).expect("Failed to create empty directory");

    let output = run_bargo_in_project(&empty_dir, &["build"]);

    assert!(
        !output.status.success(),
        "Build should fail without Nargo.toml"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Could not find Nargo.toml"),
        "Should mention missing Nargo.toml, got: {}",
        stderr
    );
    assert!(
        stderr.contains("Noir project"),
        "Should mention Noir project requirement"
    );
}

#[test]
fn test_build_dry_run_with_project() {
    let (_temp_dir, project_dir) = create_test_project();

    let output = run_bargo_in_project(&project_dir, &["--dry-run", "build"]);

    assert!(
        output.status.success(),
        "Dry run build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Would run:") || stdout.contains("nargo execute"),
        "Should show dry run output or command"
    );
}

#[test]
fn test_evm_prove_dry_run() {
    let (_temp_dir, project_dir) = create_test_project();

    let output = run_bargo_in_project(&project_dir, &["--dry-run", "evm", "prove"]);

    // This command might succeed (showing dry run) or fail (missing build artifacts)
    // Both are acceptable behaviors for this test
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("Would run:") || stdout.contains("bb prove"),
            "Successful dry run should show command that would be executed"
        );
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("Required files") || stderr.contains("build"),
            "Should provide helpful error about missing build artifacts"
        );
    }
}

#[test]
fn test_cairo_prove_dry_run() {
    let (_temp_dir, project_dir) = create_test_project();

    let output = run_bargo_in_project(&project_dir, &["--dry-run", "cairo", "prove"]);

    // Similar to EVM prove test - either succeeds with dry run or fails helpfully
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("Would run:") || stdout.contains("bb prove"),
            "Successful dry run should show command that would be executed"
        );
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("Required files") || stderr.contains("build"),
            "Should provide helpful error about missing build artifacts"
        );
    }
}

#[test]
fn test_clean_command() {
    let (_temp_dir, project_dir) = create_test_project();

    // Create some mock target directories to clean
    let target_dir = project_dir.join("target");
    let bb_dir = target_dir.join("bb");
    let evm_dir = target_dir.join("evm");

    fs::create_dir_all(&bb_dir).expect("Failed to create bb target dir");
    fs::create_dir_all(&evm_dir).expect("Failed to create evm target dir");
    fs::write(bb_dir.join("test.json"), "mock").expect("Failed to create mock file");
    fs::write(evm_dir.join("proof"), "mock").expect("Failed to create mock file");

    // Verify files exist before cleaning
    assert!(bb_dir.join("test.json").exists());
    assert!(evm_dir.join("proof").exists());

    let output = run_bargo_in_project(&project_dir, &["clean"]);

    assert!(
        output.status.success(),
        "Clean command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify files were removed
    assert!(!bb_dir.join("test.json").exists());
    assert!(!evm_dir.join("proof").exists());
}

#[test]
fn test_verbose_flag() {
    let (_temp_dir, project_dir) = create_test_project();

    let normal_output = run_bargo_in_project(&project_dir, &["--dry-run", "build"]);
    let verbose_output = run_bargo_in_project(&project_dir, &["--verbose", "--dry-run", "build"]);

    assert!(normal_output.status.success());
    assert!(verbose_output.status.success());

    let normal_stdout = String::from_utf8_lossy(&normal_output.stdout);
    let verbose_stdout = String::from_utf8_lossy(&verbose_output.stdout);

    // Verbose output should contain more information
    assert!(
        verbose_stdout.len() >= normal_stdout.len(),
        "Verbose output should be at least as long as normal output"
    );

    // Verbose should show command execution details
    assert!(
        verbose_stdout.contains("Running:") || verbose_stdout.contains("Executing:"),
        "Verbose output should show command execution details"
    );
}

#[test]
fn test_quiet_flag() {
    let (_temp_dir, project_dir) = create_test_project();

    let normal_output = run_bargo_in_project(&project_dir, &["--dry-run", "build"]);
    let quiet_output = run_bargo_in_project(&project_dir, &["--quiet", "--dry-run", "build"]);

    assert!(normal_output.status.success());
    assert!(quiet_output.status.success());

    let normal_stdout = String::from_utf8_lossy(&normal_output.stdout);
    let quiet_stdout = String::from_utf8_lossy(&quiet_output.stdout);

    // Quiet output should be shorter or equal
    assert!(
        quiet_stdout.len() <= normal_stdout.len(),
        "Quiet output should be shorter than or equal to normal output"
    );
}

#[test]
fn test_package_override_flag() {
    let (_temp_dir, project_dir) = create_test_project();

    let output = run_bargo_in_project(
        &project_dir,
        &["--pkg", "custom_name", "--dry-run", "build"],
    );

    assert!(
        output.status.success(),
        "Package override failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // The package override flag should be accepted without error
    // The actual package name usage would be visible in file operations
    // which we can't easily test in dry-run mode, so we just verify
    // the command succeeds with the flag
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Would run:") || stdout.contains("nargo execute"),
        "Should show dry run output when package override is used"
    );
}
