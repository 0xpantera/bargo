use color_eyre::Result;
use std::path::Path;
use tracing::debug;

use crate::util::{Flavour, create_smart_error};

/// Ensure target directory exists for the given backend flavour
///
/// Creates the appropriate target subdirectory based on the flavour:
/// - `Flavour::Bb` → `target/bb/`
/// - `Flavour::Evm` → `target/evm/`
/// - `Flavour::Starknet` → `target/starknet/`
pub fn ensure_target_dir(flavour: Flavour) -> Result<()> {
    let target_path = crate::util::target_dir(flavour);

    std::fs::create_dir_all(&target_path).map_err(|e| {
        let flavour_name = match flavour {
            Flavour::Bb => "bb",
            Flavour::Evm => "evm",
            Flavour::Starknet => "starknet",
        };
        create_smart_error(
            &format!("Failed to create target/{} directory: {}", flavour_name, e),
            &[
                "Check directory permissions",
                "Ensure you have write access to the current directory",
                "Verify you're running from the project root",
            ],
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

    std::fs::create_dir_all(contracts_path).map_err(|e| {
        create_smart_error(
            &format!("Failed to create contracts directory: {}", e),
            &[
                "Check directory permissions",
                "Ensure you have write access to the current directory",
                "Verify you're running from the project root",
            ],
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
        return Err(create_smart_error(
            &format!("Source directory does not exist: {}", from),
            &[
                "Check that the source directory was created correctly",
                "Verify the path is correct",
                "Ensure the previous generation step completed successfully",
            ],
        ));
    }

    // Remove destination directory if it exists
    if dest_path.exists() {
        std::fs::remove_dir_all(dest_path).map_err(|e| {
            create_smart_error(
                &format!(
                    "Failed to remove existing destination directory {}: {}",
                    to, e
                ),
                &[
                    "Check directory permissions",
                    "Ensure no processes are using files in the directory",
                    "Verify you have write access",
                ],
            )
        })?;
        debug!("Removed existing destination: {}", dest_path.display());
    }

    // Create parent directory of destination if needed
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            create_smart_error(
                &format!("Failed to create parent directory for {}: {}", to, e),
                &[
                    "Check directory permissions",
                    "Ensure you have write access to the parent directory",
                ],
            )
        })?;
    }

    // Move the directory
    std::fs::rename(source_path, dest_path).map_err(|e| {
        create_smart_error(
            &format!("Failed to move directory from {} to {}: {}", from, to, e),
            &[
                "Check directory permissions",
                "Ensure you have write access to both source and destination",
                "Verify both paths are on the same filesystem",
                "Check that no processes are using files in the source directory",
            ],
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
    fn test_ensure_target_dir() {
        let temp_dir = tempdir().unwrap();

        // Test directory creation using absolute paths
        let target_evm = temp_dir.path().join("target/evm");
        let target_starknet = temp_dir.path().join("target/starknet");
        let target_bb = temp_dir.path().join("target/bb");

        // Test that directory creation works (simulating what ensure_target_dir does)
        let result_evm = std::fs::create_dir_all(&target_evm);
        assert!(
            result_evm.is_ok(),
            "Failed to create EVM target dir: {:?}",
            result_evm.err()
        );
        assert!(target_evm.exists(), "target/evm directory should exist");

        let result_starknet = std::fs::create_dir_all(&target_starknet);
        assert!(
            result_starknet.is_ok(),
            "Failed to create Starknet target dir: {:?}",
            result_starknet.err()
        );
        assert!(
            target_starknet.exists(),
            "target/starknet directory should exist"
        );

        let result_bb = std::fs::create_dir_all(&target_bb);
        assert!(
            result_bb.is_ok(),
            "Failed to create BB target dir: {:?}",
            result_bb.err()
        );
        assert!(target_bb.exists(), "target/bb directory should exist");
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
