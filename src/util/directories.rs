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
    fn test_validate_required_files_success() {
        let temp_dir = tempdir().unwrap();

        // Create test files
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        fs::write(&file1, "content1").unwrap();
        fs::write(&file2, "content2").unwrap();

        let files = vec![file1.as_path(), file2.as_path()];
        let result = super::super::backends::validate_required_files(&files);
        assert!(
            result.is_ok(),
            "Validation should succeed when all files exist"
        );
    }

    #[test]
    fn test_validate_required_files_missing() {
        let temp_dir = tempdir().unwrap();

        // Create only one file
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("missing.txt");
        fs::write(&file1, "content1").unwrap();

        let files = vec![file1.as_path(), file2.as_path()];
        let result = super::super::backends::validate_required_files(&files);
        assert!(
            result.is_err(),
            "Validation should fail when files are missing"
        );
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("missing.txt"),
            "Error should mention missing file"
        );
    }

    #[test]
    fn test_ensure_target_dir() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Test creating EVM target directory
        let result = ensure_target_dir(Flavour::Evm);
        assert!(
            result.is_ok(),
            "Failed to create EVM target dir: {:?}",
            result.err()
        );
        assert!(
            Path::new("target/evm").exists(),
            "target/evm directory should exist"
        );

        // Test creating Starknet target directory
        let result = ensure_target_dir(Flavour::Starknet);
        assert!(
            result.is_ok(),
            "Failed to create Starknet target dir: {:?}",
            result.err()
        );
        assert!(
            Path::new("target/starknet").exists(),
            "target/starknet directory should exist"
        );

        // Test creating BB target directory
        let result = ensure_target_dir(Flavour::Bb);
        assert!(
            result.is_ok(),
            "Failed to create BB target dir: {:?}",
            result.err()
        );
        assert!(
            Path::new("target/bb").exists(),
            "target/bb directory should exist"
        );

        // Cleanup
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_ensure_contracts_dir() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = ensure_contracts_dir();
        assert!(
            result.is_ok(),
            "Failed to create contracts dir: {:?}",
            result.err()
        );
        assert!(
            Path::new("contracts").exists(),
            "contracts directory should exist after creation"
        );

        // Test that calling it again doesn't fail (idempotent)
        let result2 = ensure_contracts_dir();
        assert!(result2.is_ok(), "Second call should also succeed");
        assert!(
            Path::new("contracts").exists(),
            "contracts directory should still exist"
        );

        // Cleanup
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_move_generated_project() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Create source directory with files (using relative paths)
        fs::create_dir_all("source/subdir").unwrap();
        fs::write("source/test.txt", "test content").unwrap();
        fs::write("source/subdir/nested.txt", "nested content").unwrap();

        // Verify source exists before move
        assert!(
            Path::new("source").exists(),
            "Source should exist before move"
        );
        assert!(
            Path::new("source/test.txt").exists(),
            "Source file should exist before move"
        );

        // Move to destination
        let result = move_generated_project("source", "destination");
        assert!(result.is_ok(), "Move should succeed: {:?}", result.err());

        // Verify move was successful (using relative paths)
        assert!(
            !Path::new("source").exists(),
            "Source should not exist after move"
        );
        assert!(
            Path::new("destination").exists(),
            "Destination should exist after move"
        );
        assert!(
            Path::new("destination/test.txt").exists(),
            "Moved file should exist"
        );
        assert!(
            Path::new("destination/subdir/nested.txt").exists(),
            "Moved nested file should exist"
        );

        // Verify content is preserved
        let content = fs::read_to_string("destination/test.txt").unwrap();
        assert_eq!(content, "test content");

        // Test error case - moving non-existent directory
        let error_result = move_generated_project("nonexistent", "should_fail");
        assert!(
            error_result.is_err(),
            "Moving non-existent directory should fail"
        );

        // Cleanup
        std::env::set_current_dir(original_dir).unwrap();
    }
}
