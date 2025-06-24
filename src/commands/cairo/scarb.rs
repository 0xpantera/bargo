//! Scarb operations for Cairo project building
//!
//! This module provides focused functions for building Cairo projects using Scarb,
//! Cairo's native build tool and package manager.

use color_eyre::Result;
use std::path::Path;

use crate::backends;

/// Build a Cairo project using Scarb
///
/// This function builds a Cairo project in the specified directory using Scarb.
/// It delegates to Garaga's scarb integration for enhanced functionality.
///
/// # Arguments
/// * `project_path` - Path to the Cairo project directory containing Scarb.toml
///
/// # Returns
/// * `Result<()>` - Success or error from build process
pub fn build_cairo_project(project_path: &str) -> Result<()> {
    backends::garaga::run(&["build", project_path])
}

/// Build a Cairo project at the default contracts/cairo location
///
/// Convenience function that builds the Cairo project at the standard
/// location used by the Bargo workflow.
///
/// # Returns
/// * `Result<()>` - Success or error from build process
pub fn build_default_cairo_project() -> Result<()> {
    build_cairo_project("./contracts/cairo/")
}

/// Check if a directory contains a valid Scarb project
///
/// This function validates that a directory contains the necessary files
/// for a Scarb project (primarily Scarb.toml).
///
/// # Arguments
/// * `project_path` - Path to check for Scarb project files
///
/// # Returns
/// * `bool` - True if directory contains a valid Scarb project
pub fn is_valid_scarb_project(project_path: &str) -> bool {
    let scarb_toml = Path::new(project_path).join("Scarb.toml");
    scarb_toml.exists()
}

/// Initialize a new Scarb project in the specified directory
///
/// This function creates a new Scarb project structure with the necessary
/// configuration files.
///
/// # Arguments
/// * `project_path` - Path where the new Scarb project should be created
/// * `project_name` - Name for the new Scarb project
///
/// # Returns
/// * `Result<()>` - Success or error from initialization
pub fn init_scarb_project(project_path: &str, project_name: &str) -> Result<()> {
    backends::garaga::run(&["init", project_path, "--name", project_name])
}

/// Clean build artifacts from a Scarb project
///
/// This function removes build artifacts and temporary files from a Scarb project.
///
/// # Arguments
/// * `project_path` - Path to the Scarb project directory
///
/// # Returns
/// * `Result<()>` - Success or error from clean operation
pub fn clean_cairo_project(project_path: &str) -> Result<()> {
    backends::garaga::run(&["clean", project_path])
}

/// Test a Scarb project
///
/// This function runs the test suite for a Scarb project.
///
/// # Arguments
/// * `project_path` - Path to the Scarb project directory
///
/// # Returns
/// * `Result<()>` - Success or error from test execution
pub fn test_cairo_project(project_path: &str) -> Result<()> {
    backends::garaga::run(&["test", project_path])
}

/// Check a Scarb project for compilation errors
///
/// This function validates the Cairo code in a Scarb project without building.
///
/// # Arguments
/// * `project_path` - Path to the Scarb project directory
///
/// # Returns
/// * `Result<()>` - Success or error from check operation
pub fn check_cairo_project(project_path: &str) -> Result<()> {
    backends::garaga::run(&["check", project_path])
}
