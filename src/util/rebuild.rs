use color_eyre::Result;
use std::path::Path;
use tracing::debug;

use super::{Flavour, find_project_root, get_bytecode_path, get_witness_path};

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
    let project_root = find_project_root(start_path)?;

    // Check if target files exist (relative to project root)
    let bytecode_path = project_root.join(get_bytecode_path(pkg_name, Flavour::Bb));
    let witness_path = project_root.join(get_witness_path(pkg_name, Flavour::Bb));

    if !bytecode_path.exists() || !witness_path.exists() {
        debug!("Target files don't exist, rebuild needed");
        return Ok(true);
    }

    // Get the oldest target file time
    let bytecode_time = std::fs::metadata(&bytecode_path)?.modified()?;
    let witness_time = std::fs::metadata(&witness_path)?.modified()?;
    let target_time = bytecode_time.min(witness_time);

    // Check Nargo.toml modification time
    let nargo_toml = project_root.join("Nargo.toml");
    if nargo_toml.exists() {
        let nargo_time = std::fs::metadata(&nargo_toml)?.modified()?;
        if nargo_time > target_time {
            debug!("Nargo.toml is newer than target files, rebuild needed");
            return Ok(true);
        }
    }

    // Check Prover.toml modification time (contains circuit inputs)
    let prover_toml = project_root.join("Prover.toml");
    if prover_toml.exists() {
        let prover_time = std::fs::metadata(&prover_toml)?.modified()?;
        if prover_time > target_time {
            debug!("Prover.toml is newer than target files, rebuild needed");
            return Ok(true);
        }
    }

    // Check if any source files are newer
    let src_dir = project_root.join("src");
    if src_dir.exists() {
        if is_dir_newer_than(&src_dir, target_time)? {
            debug!("Source files are newer than target files, rebuild needed");
            return Ok(true);
        }
    }

    debug!("Target files are up to date");
    Ok(false)
}

/// Recursively check if any file in a directory is newer than the given time
fn is_dir_newer_than(dir: &Path, target_time: std::time::SystemTime) -> Result<bool> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let file_time = std::fs::metadata(&path)?.modified()?;
            if file_time > target_time {
                return Ok(true);
            }
        } else if path.is_dir() {
            if is_dir_newer_than(&path, target_time)? {
                return Ok(true);
            }
        }
    }

    Ok(false)
}
