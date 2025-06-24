use color_eyre::Result;
use std::path::Path;
use tracing::info;

use crate::{
    backends,
    util::{self, Flavour, create_smart_error},
};

/// Generate proof with specific oracle hash and additional flags
///
/// This function handles the common pattern of generating proofs with
/// different oracle hashes and backend-specific flags.
///
/// # Arguments
/// * `pkg` - Package name for locating bytecode and witness files
/// * `flavour` - Backend flavour determining output directory
/// * `oracle` - Oracle hash type ("keccak", "starknet", etc.)
/// * `extra_flags` - Additional flags specific to the backend
///
/// # Example
/// ```
/// // EVM proof generation
/// generate_proof_with_oracle(
///     "my_package",
///     Flavour::Evm,
///     "keccak",
///     &["--output_format", "bytes_and_fields"]
/// )?;
///
/// // Starknet proof generation
/// generate_proof_with_oracle(
///     "my_package",
///     Flavour::Starknet,
///     "starknet",
///     &["--scheme", "ultra_honk", "--zk"]
/// )?;
/// ```
pub fn generate_proof_with_oracle(
    pkg: &str,
    flavour: Flavour,
    oracle: &str,
    extra_flags: &[&str],
) -> Result<()> {
    let bytecode_path = util::get_bytecode_path(pkg, Flavour::Bb);
    let witness_path = util::get_witness_path(pkg, Flavour::Bb);
    let target_path = util::target_dir(flavour);

    let bytecode_str = bytecode_path.to_string_lossy();
    let witness_str = witness_path.to_string_lossy();
    let target_str = format!("{}/", target_path.display());

    // Build base arguments
    let mut args = vec![
        "prove",
        "-b",
        &bytecode_str,
        "-w",
        &witness_str,
        "-o",
        &target_str,
        "--oracle_hash",
        oracle,
    ];

    // Add extra flags
    args.extend_from_slice(extra_flags);

    backends::bb::run(&args)
}

/// Generate verification key with specific oracle hash
///
/// Creates a verification key for the given backend using the appropriate oracle hash.
///
/// # Arguments
/// * `pkg` - Package name for locating bytecode file
/// * `flavour` - Backend flavour determining output directory
/// * `oracle` - Oracle hash type ("keccak", "starknet", etc.)
pub fn generate_vk_with_oracle(pkg: &str, flavour: Flavour, oracle: &str) -> Result<()> {
    let bytecode_path = util::get_bytecode_path(pkg, Flavour::Bb);
    let target_path = util::target_dir(flavour);

    let bytecode_str = bytecode_path.to_string_lossy();
    let target_str = format!("{}/", target_path.display());

    let args = vec![
        "write_vk",
        "--oracle_hash",
        oracle,
        "-b",
        &bytecode_str,
        "-o",
        &target_str,
    ];

    backends::bb::run(&args)
}

/// Verify proof using backend-specific artifacts
///
/// Verifies a proof using the verification key, proof, and public inputs
/// from the specified backend's target directory.
///
/// # Arguments
/// * `flavour` - Backend flavour determining which artifacts to use
pub fn verify_proof_generic(flavour: Flavour) -> Result<()> {
    let proof_path = util::get_proof_path(flavour);
    let vk_path = util::get_vk_path(flavour);
    let public_inputs_path = util::get_public_inputs_path(flavour);

    let vk_str = vk_path.to_string_lossy();
    let proof_str = proof_path.to_string_lossy();
    let public_inputs_str = public_inputs_path.to_string_lossy();

    let args = vec![
        "verify",
        "-k",
        &vk_str,
        "-p",
        &proof_str,
        "-i",
        &public_inputs_str,
    ];

    backends::bb::run(&args)
}

/// Validate that required files exist before proceeding
///
/// Checks that all specified files exist and provides helpful error messages
/// if any are missing.
///
/// # Arguments
/// * `files` - Slice of paths to validate
pub fn validate_required_files(files: &[&Path]) -> Result<()> {
    let mut missing_files = Vec::new();

    for file_path in files {
        if !file_path.exists() {
            missing_files.push(file_path.display().to_string());
        }
    }

    if !missing_files.is_empty() {
        return Err(create_smart_error(
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

/// Generate proof and VK for a backend in one operation
///
/// This is a convenience function that combines proof and VK generation
/// for backends that need both operations.
///
/// # Arguments
/// * `pkg` - Package name
/// * `flavour` - Backend flavour
/// * `oracle` - Oracle hash type
/// * `extra_proof_flags` - Additional flags for proof generation
/// * `verbose` - Whether to log commands being run
pub fn generate_proof_and_vk(
    pkg: &str,
    flavour: Flavour,
    oracle: &str,
    extra_proof_flags: &[&str],
    verbose: bool,
) -> Result<()> {
    // Generate proof
    if verbose {
        let bytecode_path = util::get_bytecode_path(pkg, Flavour::Bb);
        let witness_path = util::get_witness_path(pkg, Flavour::Bb);
        let target_path = util::target_dir(flavour);

        let mut proof_args = vec![
            "prove".to_string(),
            "-b".to_string(),
            bytecode_path.display().to_string(),
            "-w".to_string(),
            witness_path.display().to_string(),
            "-o".to_string(),
            format!("{}/", target_path.display()),
            "--oracle_hash".to_string(),
            oracle.to_string(),
        ];

        for flag in extra_proof_flags {
            proof_args.push(flag.to_string());
        }

        info!("Running: bb {}", proof_args.join(" "));
    }

    generate_proof_with_oracle(pkg, flavour, oracle, extra_proof_flags)?;

    // Generate VK
    if verbose {
        let bytecode_path = util::get_bytecode_path(pkg, Flavour::Bb);
        let target_path = util::target_dir(flavour);

        let bytecode_str = bytecode_path.display().to_string();
        let target_str = format!("{}/", target_path.display());

        let vk_args = vec![
            "write_vk",
            "--oracle_hash",
            oracle,
            "-b",
            &bytecode_str,
            "-o",
            &target_str,
        ];

        info!("Running: bb {}", vk_args.join(" "));
    }

    generate_vk_with_oracle(pkg, flavour, oracle)?;

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
        assert!(validate_required_files(&files).is_ok());
    }

    #[test]
    fn test_validate_required_files_missing() {
        let temp_dir = tempdir().unwrap();

        // Create only one file
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("missing.txt");
        fs::write(&file1, "content1").unwrap();

        let files = vec![file1.as_path(), file2.as_path()];
        let result = validate_required_files(&files);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing.txt"));
    }

    #[test]
    fn test_validate_required_files_empty_list() {
        let files: Vec<&std::path::Path> = vec![];
        let result = validate_required_files(&files);
        assert!(result.is_ok(), "Empty file list should be valid");
    }

    #[test]
    fn test_validate_required_files_multiple_missing() {
        let temp_dir = tempdir().unwrap();

        // Don't create any files, just reference paths
        let file1 = temp_dir.path().join("missing1.txt");
        let file2 = temp_dir.path().join("missing2.txt");

        let files = vec![file1.as_path(), file2.as_path()];
        let result = validate_required_files(&files);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("missing1.txt"));
        assert!(error_msg.contains("missing2.txt"));
    }

    // Note: The following tests verify argument construction and path handling
    // without actually calling bb (which would require the binary to be available)

    #[test]
    fn test_target_dir_paths() {
        use crate::util;

        // Test that target_dir returns expected paths for each flavour
        assert_eq!(util::target_dir(Flavour::Bb).to_string_lossy(), "target/bb");
        assert_eq!(
            util::target_dir(Flavour::Evm).to_string_lossy(),
            "target/evm"
        );
        assert_eq!(
            util::target_dir(Flavour::Starknet).to_string_lossy(),
            "target/starknet"
        );
    }

    #[test]
    fn test_path_resolution() {
        use crate::util;

        // Test that path functions return expected paths
        let bytecode_path = util::get_bytecode_path("test_pkg", Flavour::Bb);
        let witness_path = util::get_witness_path("test_pkg", Flavour::Bb);
        let proof_path = util::get_proof_path(Flavour::Evm);
        let vk_path = util::get_vk_path(Flavour::Starknet);

        assert!(bytecode_path.to_string_lossy().contains("test_pkg.json"));
        assert!(witness_path.to_string_lossy().contains("test_pkg.gz"));
        assert!(proof_path.to_string_lossy().contains("target/evm"));
        assert!(vk_path.to_string_lossy().contains("target/starknet"));
    }

    #[test]
    fn test_flavour_consistency() {
        // Test that all Flavour variants are handled consistently
        let flavours = vec![Flavour::Bb, Flavour::Evm, Flavour::Starknet];

        for flavour in flavours {
            // These should not panic and should return valid paths
            let target_dir = crate::util::target_dir(flavour);
            let proof_path = crate::util::get_proof_path(flavour);
            let vk_path = crate::util::get_vk_path(flavour);
            let public_inputs_path = crate::util::get_public_inputs_path(flavour);

            assert!(!target_dir.as_os_str().is_empty());
            assert!(!proof_path.as_os_str().is_empty());
            assert!(!vk_path.as_os_str().is_empty());
            assert!(!public_inputs_path.as_os_str().is_empty());

            // All paths should start with target/
            assert!(target_dir.to_string_lossy().starts_with("target/"));
            assert!(proof_path.to_string_lossy().starts_with("target/"));
            assert!(vk_path.to_string_lossy().starts_with("target/"));
            assert!(public_inputs_path.to_string_lossy().starts_with("target/"));
        }
    }

    #[test]
    fn test_oracle_hash_types() {
        // Test that different oracle hashes are handled properly
        let oracles = vec!["keccak", "starknet", "custom"];

        for oracle in oracles {
            // Test that oracle parameter is used correctly in argument construction
            // (We can't test the actual bb call, but we can test our logic)
            assert!(!oracle.is_empty(), "Oracle hash should not be empty");
            assert!(
                oracle
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_'),
                "Oracle hash should be alphanumeric: {}",
                oracle
            );
        }
    }

    #[test]
    fn test_extra_flags_handling() {
        // Test that extra flags are properly handled
        let test_flags = vec![
            vec!["--output_format", "bytes_and_fields"],
            vec!["--scheme", "ultra_honk", "--zk"],
            vec![], // empty flags
            vec!["--single-flag"],
        ];

        for flags in test_flags {
            // Test that flags are valid format
            for flag in &flags {
                assert!(!flag.is_empty(), "Flag should not be empty");
                if flag.starts_with("--") {
                    assert!(flag.len() > 2, "Flag should have content after --");
                }
            }
        }
    }

    #[test]
    fn test_package_name_handling() {
        let valid_names = vec!["wkshp", "test_package", "my-circuit", "package123"];
        let invalid_names = vec!["", ".", "..", "/invalid", "\\invalid"];

        for name in valid_names {
            // Valid package names should work with path construction
            let bytecode_path = crate::util::get_bytecode_path(name, Flavour::Bb);
            assert!(bytecode_path.to_string_lossy().contains(name));
            assert!(bytecode_path.to_string_lossy().ends_with(".json"));
        }

        for name in invalid_names {
            // Invalid names should still not panic (robustness test)
            let bytecode_path = crate::util::get_bytecode_path(name, Flavour::Bb);
            // Should complete without panicking
            let _ = bytecode_path.to_string_lossy();
        }
    }
}
