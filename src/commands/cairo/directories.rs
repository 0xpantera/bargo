//! Directory management for Cairo workflow
//!
//! This module provides focused functions for managing directories and file organization
//! specific to the Cairo/Starknet workflow, including artifact organization and project movement.

use color_eyre::Result;
use std::path::{Path, PathBuf};

use crate::util::{self, Flavour};

/// Ensure the Starknet target directory exists
///
/// This function creates the target/starknet directory if it doesn't exist,
/// which is where Starknet-specific artifacts (proofs, VKs, public inputs) are stored.
///
/// # Returns
/// * `Result<()>` - Success or error from directory creation
pub fn ensure_starknet_target_dir() -> Result<()> {
    util::ensure_target_dir(Flavour::Starknet)
}

/// Ensure the Cairo contracts directory exists
///
/// This function creates the contracts/cairo directory if it doesn't exist,
/// which is where generated Cairo verifier contracts are stored.
///
/// # Returns
/// * `Result<()>` - Success or error from directory creation
pub fn ensure_cairo_contracts_dir() -> Result<()> {
    let cairo_dir = Path::new("./contracts/cairo");
    if !cairo_dir.exists() {
        std::fs::create_dir_all(cairo_dir)?;
    }
    Ok(())
}

/// Move generated Cairo project to the contracts directory
///
/// This function moves a generated Cairo project from a temporary location
/// to the standard contracts/cairo directory used by the Bargo workflow.
///
/// # Arguments
/// * `from` - Source path of the generated project
/// * `to` - Optional destination path (defaults to ./contracts/cairo)
///
/// # Returns
/// * `Result<()>` - Success or error from move operation
pub fn move_cairo_project(from: Option<&str>, to: Option<&str>) -> Result<()> {
    let source = from.unwrap_or("./cairo_project");
    let destination = to.unwrap_or("./contracts/cairo");

    util::move_generated_project(source, destination)
}

/// Clean Cairo-specific build artifacts
///
/// This function removes Cairo-specific build artifacts and temporary files,
/// including the target/starknet directory and any temporary project directories.
///
/// # Returns
/// * `Result<()>` - Success or error from cleanup
pub fn clean_cairo_artifacts() -> Result<()> {
    let starknet_dir = Path::new("./target/starknet");
    if starknet_dir.exists() {
        std::fs::remove_dir_all(starknet_dir)?;
    }

    // Clean temporary project directories that might be left behind
    let temp_dirs = vec!["./cairo_project", "./temp_cairo"];
    for temp_dir in temp_dirs {
        let path = Path::new(temp_dir);
        if path.exists() {
            std::fs::remove_dir_all(path)?;
        }
    }

    Ok(())
}

/// Validate Cairo workflow directory structure
///
/// This function checks that all necessary directories exist for the Cairo workflow
/// and creates them if they don't exist.
///
/// # Returns
/// * `Result<()>` - Success or error from validation/creation
pub fn validate_cairo_directory_structure() -> Result<()> {
    ensure_starknet_target_dir()?;
    util::ensure_contracts_dir()?;
    ensure_cairo_contracts_dir()?;
    Ok(())
}

/// Get the path to the Cairo contracts directory
///
/// # Returns
/// * `PathBuf` - Path to the Cairo contracts directory
pub fn get_cairo_contracts_dir() -> PathBuf {
    PathBuf::from("./contracts/cairo")
}

/// Get the path to the Starknet artifacts directory
///
/// # Returns
/// * `PathBuf` - Path to the Starknet target directory
pub fn get_starknet_artifacts_dir() -> PathBuf {
    util::target_dir(Flavour::Starknet)
}

/// Check if Cairo project structure is valid
///
/// This function validates that the Cairo project has the expected structure
/// with all necessary files and directories.
///
/// # Arguments
/// * `project_path` - Path to the Cairo project directory
///
/// # Returns
/// * `bool` - True if project structure is valid
pub fn is_valid_cairo_project_structure(project_path: &str) -> bool {
    let project = Path::new(project_path);
    let scarb_toml = project.join("Scarb.toml");
    let src_dir = project.join("src");

    scarb_toml.exists() && src_dir.exists() && src_dir.is_dir()
}

/// Create standard Cairo project directory structure
///
/// This function creates the standard directory structure for a Cairo project
/// including src/ directory and basic configuration files.
///
/// # Arguments
/// * `project_path` - Path where the project structure should be created
///
/// # Returns
/// * `Result<()>` - Success or error from structure creation
pub fn create_cairo_project_structure(project_path: &str) -> Result<()> {
    let project = Path::new(project_path);
    let src_dir = project.join("src");

    std::fs::create_dir_all(&src_dir)?;

    Ok(())
}

/// Get all Cairo-related artifact paths
///
/// This function returns a vector of all paths where Cairo-related artifacts
/// are stored, useful for cleanup or validation operations.
///
/// # Returns
/// * `Vec<PathBuf>` - Vector of artifact directory paths
pub fn get_cairo_artifact_paths() -> Vec<PathBuf> {
    vec![
        get_starknet_artifacts_dir(),
        get_cairo_contracts_dir(),
        PathBuf::from("./target/bb"), // Shared artifacts
    ]
}
