use color_eyre::Result;
use std::path::Path;
use tracing::debug;

use super::{find_project_root, get_bytecode_path, get_witness_path, Flavour};

/// Check if source files are newer than target files (for smart rebuilds)
pub fn needs_rebuild(pkg_name: &str) -> Result<bool> {
    let current_dir = std::env::current_dir()?;
    let project_root = find_project_root(&current_dir)?;

    // Check if target files exist
    let bytecode_path = get_bytecode_path(pkg_name, Flavour::Bb);
    let witness_path = get_witness_path(pkg_name, Flavour::Bb);

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
