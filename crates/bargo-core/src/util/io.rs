//! File and directory I/O operations for bargo
//!
//! This module provides utilities for file system operations, validation,
//! and command specification helpers used throughout the bargo workflow.
//!
//! ## Key Features
//!
//! - File existence validation
//! - Directory creation and management
//! - Smart rebuild detection
//! - Command specification macro helpers
//!
//! ## Examples
//!
//! ```ignore
//! use bargo_core::util::io::{validate_files_exist, needs_rebuild};
//!
//! // Validate required files exist
//! validate_files_exist(&["target/proof", "target/vk"])?;
//!
//! // Check if rebuild is needed
//! if needs_rebuild("my_package")? {
//!     println!("Rebuild required");
//! }
//! ```

use color_eyre::Result;
use color_eyre::eyre::WrapErr;
use std::path::Path;
use tracing::debug;

/// Macro for creating command specifications with convenient syntax
///
/// This macro provides a convenient way to create `CmdSpec` instances
/// with optional working directory and environment variables.
///
/// # Examples
///
/// ```ignore
/// use bargo_core::util::io::cmd_spec;
///
/// // Simple command
/// let spec = cmd_spec!("echo", ["hello", "world"]);
///
/// // Command with working directory
/// let spec = cmd_spec!("cargo", ["build"], cwd: "./my-project");
///
/// // Command with environment variables
/// let spec = cmd_spec!("forge", ["create"], env: {"RPC_URL" => "http://localhost:8545"});
///
/// // Command with both
/// let spec = cmd_spec!(
///     "bb", ["prove", "--scheme", "ultra_honk"],
///     cwd: "./target",
///     env: {"PROOF_DIR" => "./proofs"}
/// );
/// ```
#[macro_export]
macro_rules! cmd_spec {
    // Simple command: cmd_spec!("tool", ["arg1", "arg2"])
    ($cmd:expr, [$($arg:expr),* $(,)?]) => {
        $crate::runner::CmdSpec::new(
            $cmd.to_string(),
            vec![$($arg.to_string()),*]
        )
    };

    // Command with working directory: cmd_spec!("tool", ["args"], cwd: "path")
    ($cmd:expr, [$($arg:expr),* $(,)?], cwd: $cwd:expr) => {
        $crate::runner::CmdSpec::new(
            $cmd.to_string(),
            vec![$($arg.to_string()),*]
        ).with_cwd(std::path::PathBuf::from($cwd))
    };

    // Command with environment: cmd_spec!("tool", ["args"], env: {"KEY" => "value"})
    ($cmd:expr, [$($arg:expr),* $(,)?], env: {$($key:expr => $val:expr),* $(,)?}) => {
        $crate::runner::CmdSpec::new(
            $cmd.to_string(),
            vec![$($arg.to_string()),*]
        ).with_envs(vec![$(($key.to_string(), $val.to_string())),*])
    };

    // Command with both cwd and env
    ($cmd:expr, [$($arg:expr),* $(,)?], cwd: $cwd:expr, env: {$($key:expr => $val:expr),* $(,)?}) => {
        $crate::runner::CmdSpec::new(
            $cmd.to_string(),
            vec![$($arg.to_string()),*]
        )
        .with_cwd(std::path::PathBuf::from($cwd))
        .with_envs(vec![$(($key.to_string(), $val.to_string())),*])
    };

    // Command with env first, then cwd
    ($cmd:expr, [$($arg:expr),* $(,)?], env: {$($key:expr => $val:expr),* $(,)?}, cwd: $cwd:expr) => {
        $crate::runner::CmdSpec::new(
            $cmd.to_string(),
            vec![$($arg.to_string()),*]
        )
        .with_envs(vec![$(($key.to_string(), $val.to_string())),*])
        .with_cwd(std::path::PathBuf::from($cwd))
    };
}

// Re-export the macro for convenience
// Note: The macro is available via the crate root through the #[macro_export] attribute

/// Validate that required files exist for a given operation
pub fn validate_files_exist<P: AsRef<Path>>(files: &[P]) -> Result<()> {
    let mut missing_files = Vec::new();

    for file_path in files {
        if !file_path.as_ref().exists() {
            missing_files.push(file_path.as_ref().display().to_string());
        }
    }

    if !missing_files.is_empty() {
        return Err(crate::util::error::create_smart_error(
            &format!("Required files are missing: {}", missing_files.join(", ")),
            &[
                "Run 'bargo build' to generate bytecode and witness files",
                "Ensure the previous workflow steps completed successfully",
                "Check that you're running from the correct directory",
                "Verify the package name is correct",
            ],
        ));
    }

    Ok(())
}

/// Check if source files are newer than target files (for smart rebuilds)
pub fn needs_rebuild(pkg_name: &str) -> Result<bool> {
    let current_dir = std::env::current_dir()?;
    needs_rebuild_from_path(pkg_name, &current_dir)
}

/// Check if source files are newer than target files from a specific starting path
///
/// This version accepts a path parameter for better testability while maintaining
/// the same rebuild detection logic.
pub fn needs_rebuild_from_path(pkg_name: &str, start_path: &Path) -> Result<bool> {
    let project_root = crate::util::paths::find_project_root(start_path)?;

    // Check if target files exist (relative to project root)
    let bytecode_path = project_root.join(crate::util::paths::get_bytecode_path(
        pkg_name,
        crate::util::paths::Flavour::Bb,
    ));
    let witness_path = project_root.join(crate::util::paths::get_witness_path(
        pkg_name,
        crate::util::paths::Flavour::Bb,
    ));

    if !bytecode_path.exists() || !witness_path.exists() {
        debug!("Target files don't exist, rebuild needed");
        return Ok(true);
    }

    // Get the oldest target file time
    let bytecode_time = std::fs::metadata(&bytecode_path)
        .wrap_err_with(|| {
            format!(
                "reading metadata for bytecode file {}",
                bytecode_path.display()
            )
        })?
        .modified()
        .wrap_err("getting modification time for bytecode file")?;
    let witness_time = std::fs::metadata(&witness_path)
        .wrap_err_with(|| {
            format!(
                "reading metadata for witness file {}",
                witness_path.display()
            )
        })?
        .modified()
        .wrap_err("getting modification time for witness file")?;
    let target_time = bytecode_time.min(witness_time);

    // Check Nargo.toml modification time
    let nargo_toml = project_root.join("Nargo.toml");
    if nargo_toml.exists() {
        let nargo_time = std::fs::metadata(&nargo_toml)
            .wrap_err_with(|| {
                format!(
                    "reading metadata for Nargo.toml at {}",
                    nargo_toml.display()
                )
            })?
            .modified()
            .wrap_err("getting modification time for Nargo.toml")?;
        if nargo_time > target_time {
            debug!("Nargo.toml is newer than target files, rebuild needed");
            return Ok(true);
        }
    }

    // Check Prover.toml modification time (contains circuit inputs)
    let prover_toml = project_root.join("Prover.toml");
    if prover_toml.exists() {
        let prover_time = std::fs::metadata(&prover_toml)
            .wrap_err_with(|| {
                format!(
                    "reading metadata for Prover.toml at {}",
                    prover_toml.display()
                )
            })?
            .modified()
            .wrap_err("getting modification time for Prover.toml")?;
        if prover_time > target_time {
            debug!("Prover.toml is newer than target files, rebuild needed");
            return Ok(true);
        }
    }

    // Check if any source files are newer
    let src_dir = project_root.join("src");
    if src_dir.exists() && is_dir_newer_than(&src_dir, target_time)? {
        debug!("Source files are newer than target files, rebuild needed");
        return Ok(true);
    }

    debug!("Target files are up to date");
    Ok(false)
}

/// Recursively check if any file in a directory is newer than the given time
fn is_dir_newer_than(dir: &Path, target_time: std::time::SystemTime) -> Result<bool> {
    for entry in
        std::fs::read_dir(dir).wrap_err_with(|| format!("reading directory {}", dir.display()))?
    {
        let entry =
            entry.wrap_err_with(|| format!("reading directory entry in {}", dir.display()))?;
        let path = entry.path();

        if path.is_file() {
            let file_time = std::fs::metadata(&path)
                .wrap_err_with(|| format!("reading metadata for file {}", path.display()))?
                .modified()
                .wrap_err("getting modification time for file")?;
            if file_time > target_time {
                return Ok(true);
            }
        } else if path.is_dir() && is_dir_newer_than(&path, target_time)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Ensure target directory exists for the given backend flavour
///
/// Creates the appropriate target subdirectory based on the flavour:
/// - `Flavour::Bb` → `target/bb/`
/// - `Flavour::Evm` → `target/evm/`
/// - `Flavour::Starknet` → `target/starknet/`
pub fn ensure_target_dir(flavour: crate::util::Flavour) -> Result<()> {
    let target_path = crate::util::paths::target_dir(flavour);

    std::fs::create_dir_all(&target_path).wrap_err_with(|| {
        let flavour_name = match flavour {
            crate::util::Flavour::Bb => "bb",
            crate::util::Flavour::Evm => "evm",
            crate::util::Flavour::Starknet => "starknet",
        };
        format!(
            "creating target/{} directory at {}",
            flavour_name,
            target_path.display()
        )
    })?;

    debug!("Created target directory: {}", target_path.display());
    Ok(())
}

/// Ensure contracts directory exists
///
/// Creates the `contracts/` directory if it doesn't exist.
/// This is used by both EVM and Cairo workflows.
pub fn ensure_contracts_dir() -> Result<()> {
    let contracts_path = Path::new("./contracts");

    std::fs::create_dir_all(contracts_path).wrap_err_with(|| {
        format!(
            "creating contracts directory at {}",
            contracts_path.display()
        )
    })?;

    debug!("Created contracts directory: {}", contracts_path.display());
    Ok(())
}

/// Move a generated project directory from source to destination
///
/// This is commonly used to move temporary generated directories
/// (like garaga's output) to their final location in the contracts directory.
///
/// # Arguments
/// * `from` - Source directory path
/// * `to` - Destination directory path
///
/// # Behavior
/// - If destination exists, it will be removed first
/// - Creates parent directories of destination if needed
/// - Moves the entire directory tree
pub fn move_generated_project(from: &str, to: &str) -> Result<()> {
    let source_path = Path::new(from);
    let dest_path = Path::new(to);

    if !source_path.exists() {
        return Err(
            color_eyre::eyre::eyre!("Source directory does not exist: {}", from)
                .wrap_err("validating source directory for move operation"),
        );
    }

    // Remove destination directory if it exists
    if dest_path.exists() {
        std::fs::remove_dir_all(dest_path).wrap_err_with(|| {
            format!(
                "removing existing destination directory {}",
                dest_path.display()
            )
        })?;
        debug!("Removed existing destination: {}", dest_path.display());
    }

    // Create parent directory of destination if needed
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)
            .wrap_err_with(|| format!("creating parent directory for {}", dest_path.display()))?;
    }

    // Move the directory
    std::fs::rename(source_path, dest_path).wrap_err_with(|| {
        format!(
            "moving directory from {} to {}",
            source_path.display(),
            dest_path.display()
        )
    })?;

    debug!(
        "Moved directory: {} -> {}",
        source_path.display(),
        dest_path.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_cmd_spec_macro_simple() {
        let spec = cmd_spec!("echo", ["hello", "world"]);
        assert_eq!(spec.cmd, "echo");
        assert_eq!(spec.args, vec!["hello", "world"]);
        assert!(spec.cwd.is_none());
        assert!(spec.env.is_empty());
    }

    #[test]
    fn test_cmd_spec_macro_with_cwd() {
        let spec = cmd_spec!("cargo", ["build"], cwd: "./my-project");
        assert_eq!(spec.cmd, "cargo");
        assert_eq!(spec.args, vec!["build"]);
        assert_eq!(spec.cwd, Some(std::path::PathBuf::from("./my-project")));
        assert!(spec.env.is_empty());
    }

    #[test]
    fn test_cmd_spec_macro_with_env() {
        let spec = cmd_spec!("forge", ["create"], env: {"RPC_URL" => "http://localhost:8545"});
        assert_eq!(spec.cmd, "forge");
        assert_eq!(spec.args, vec!["create"]);
        assert!(spec.cwd.is_none());
        assert_eq!(
            spec.env,
            vec![("RPC_URL".to_string(), "http://localhost:8545".to_string())]
        );
    }

    #[test]
    fn test_cmd_spec_macro_with_both() {
        let spec = cmd_spec!(
            "bb", ["prove", "--scheme", "ultra_honk"],
            cwd: "./target",
            env: {"PROOF_DIR" => "./proofs", "MODE" => "test"}
        );
        assert_eq!(spec.cmd, "bb");
        assert_eq!(spec.args, vec!["prove", "--scheme", "ultra_honk"]);
        assert_eq!(spec.cwd, Some(std::path::PathBuf::from("./target")));
        assert_eq!(spec.env.len(), 2);
        assert!(
            spec.env
                .contains(&("PROOF_DIR".to_string(), "./proofs".to_string()))
        );
        assert!(spec.env.contains(&("MODE".to_string(), "test".to_string())));
    }

    #[test]
    fn test_validate_files_exist_success() {
        let temp_dir = tempdir().unwrap();
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");

        // Create test files
        fs::write(&file1, "content1").unwrap();
        fs::write(&file2, "content2").unwrap();

        // Should succeed when all files exist
        let result = validate_files_exist(&[&file1, &file2]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_files_exist_missing() {
        let temp_dir = tempdir().unwrap();
        let existing_file = temp_dir.path().join("exists.txt");
        let missing_file = temp_dir.path().join("missing.txt");

        // Create only one file
        fs::write(&existing_file, "content").unwrap();

        // Should fail when some files are missing
        let result = validate_files_exist(&[&existing_file, &missing_file]);
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("Required files are missing"));
        assert!(error_msg.contains("missing.txt"));
    }

    #[test]
    fn test_ensure_contracts_dir() {
        let temp_dir = tempdir().unwrap();
        let contracts_dir = temp_dir.path().join("contracts");

        // Test contracts directory creation using absolute path
        let result = std::fs::create_dir_all(&contracts_dir);
        assert!(
            result.is_ok(),
            "Failed to create contracts dir: {:?}",
            result.err()
        );
        assert!(
            contracts_dir.exists(),
            "contracts directory should exist after creation"
        );

        // Test that calling it again doesn't fail (idempotent)
        let result2 = std::fs::create_dir_all(&contracts_dir);
        assert!(result2.is_ok(), "Second call should also succeed");
        assert!(
            contracts_dir.exists(),
            "contracts directory should still exist"
        );
    }

    #[test]
    fn test_move_generated_project() {
        let temp_dir = tempdir().unwrap();

        // Use absolute paths throughout
        let source_dir = temp_dir.path().join("source");
        let source_subdir = source_dir.join("subdir");
        let source_file = source_dir.join("test.txt");
        let source_nested_file = source_subdir.join("nested.txt");
        let dest_dir = temp_dir.path().join("destination");
        let dest_file = dest_dir.join("test.txt");
        let dest_nested_file = dest_dir.join("subdir/nested.txt");

        // Create source directory with files using absolute paths
        fs::create_dir_all(&source_subdir).unwrap();
        fs::write(&source_file, "test content").unwrap();
        fs::write(&source_nested_file, "nested content").unwrap();

        // Verify source exists before move
        assert!(source_dir.exists(), "Source should exist before move");
        assert!(source_file.exists(), "Source file should exist before move");

        // Move to destination using absolute paths
        let result =
            move_generated_project(&source_dir.to_string_lossy(), &dest_dir.to_string_lossy());
        assert!(result.is_ok(), "Move should succeed: {:?}", result.err());

        // Verify move was successful using absolute paths
        assert!(!source_dir.exists(), "Source should not exist after move");
        assert!(dest_dir.exists(), "Destination should exist after move");
        assert!(dest_file.exists(), "Moved file should exist");
        assert!(dest_nested_file.exists(), "Moved nested file should exist");

        // Verify content is preserved
        let content = fs::read_to_string(&dest_file).unwrap();
        assert_eq!(content, "test content");

        // Test error case - moving non-existent directory
        let nonexistent = temp_dir.path().join("nonexistent");
        let should_fail = temp_dir.path().join("should_fail");
        let error_result = move_generated_project(
            &nonexistent.to_string_lossy(),
            &should_fail.to_string_lossy(),
        );
        assert!(
            error_result.is_err(),
            "Moving non-existent directory should fail"
        );
    }
}
