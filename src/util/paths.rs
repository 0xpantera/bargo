use color_eyre::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

use crate::util::create_smart_error;

/// Backend flavour for artifact generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flavour {
    /// Barretenberg backend (shared base artifacts)
    Bb,
    /// EVM backend (Keccak oracle)
    Evm,
    /// Starknet backend (Cairo)
    Starknet,
}

/// Find the project root by walking up the directory tree looking for Nargo.toml
pub fn find_project_root(current_path: &Path) -> Result<PathBuf> {
    for path in current_path.ancestors() {
        let nargo_toml = path.join("Nargo.toml");
        if nargo_toml.exists() {
            debug!("Found Nargo.toml at: {}", nargo_toml.display());
            return Ok(path.to_path_buf());
        }
    }

    Err(color_eyre::eyre::eyre!(
        "Could not find Nargo.toml in current directory or any parent directory.\n\
         Make sure you're running bargo from within a Noir project."
    ))
}

/// Get the package name from Nargo.toml, with optional override
pub fn get_package_name(pkg_override: Option<&String>) -> Result<String> {
    if let Some(pkg_name) = pkg_override {
        debug!("Using package name override: {}", pkg_name);
        return Ok(pkg_name.clone());
    }

    let current_dir = std::env::current_dir()?;
    let project_root = find_project_root(&current_dir)?;
    let nargo_toml_path = project_root.join("Nargo.toml");

    parse_package_name(&nargo_toml_path)
}

/// Parse the package name from a Nargo.toml file
pub fn parse_package_name(nargo_toml_path: &Path) -> Result<String> {
    let toml_content = std::fs::read_to_string(nargo_toml_path).map_err(|e| {
        color_eyre::eyre::eyre!(
            "Failed to read Nargo.toml at {}: {}",
            nargo_toml_path.display(),
            e
        )
    })?;

    let config: TomlConfig = toml::from_str(&toml_content).map_err(|e| {
        color_eyre::eyre::eyre!(
            "Failed to parse Nargo.toml at {}: {}",
            nargo_toml_path.display(),
            e
        )
    })?;

    match config {
        TomlConfig::Package { package } => package.name.ok_or_else(|| {
            color_eyre::eyre::eyre!(
                "Missing 'name' field in [package] section of Nargo.toml at {}",
                nargo_toml_path.display()
            )
        }),
        TomlConfig::Workspace { .. } => {
            warn!("Found workspace Nargo.toml, using directory name as package name");
            let dir_name = nargo_toml_path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|name| name.to_str())
                .map(|s| s.to_string())
                .ok_or_else(|| {
                    color_eyre::eyre::eyre!(
                        "Could not determine package name from workspace directory"
                    )
                })?;
            Ok(dir_name)
        }
    }
}

/// Get the target directory path for a specific backend flavour
pub fn target_dir(flavour: Flavour) -> PathBuf {
    match flavour {
        Flavour::Bb => PathBuf::from("target/bb"),
        Flavour::Evm => PathBuf::from("target/evm"),
        Flavour::Starknet => PathBuf::from("target/starknet"),
    }
}

/// Get the bytecode file path for a package with specific backend flavour
pub fn get_bytecode_path(pkg_name: &str, flavour: Flavour) -> PathBuf {
    target_dir(flavour).join(format!("{}.json", pkg_name))
}

/// Get the witness file path for a package with specific backend flavour
pub fn get_witness_path(pkg_name: &str, flavour: Flavour) -> PathBuf {
    target_dir(flavour).join(format!("{}.gz", pkg_name))
}

/// Get the proof file path for specific backend flavour
pub fn get_proof_path(flavour: Flavour) -> PathBuf {
    target_dir(flavour).join("proof")
}

/// Get the verification key file path for specific backend flavour
pub fn get_vk_path(flavour: Flavour) -> PathBuf {
    target_dir(flavour).join("vk")
}

/// Get the public inputs file path for specific backend flavour
pub fn get_public_inputs_path(flavour: Flavour) -> PathBuf {
    target_dir(flavour).join("public_inputs")
}

/// Organize build artifacts by moving nargo output to appropriate flavour directory
pub fn organize_build_artifacts(pkg_name: &str, flavour: Flavour) -> Result<()> {
    // Create the target directory for the flavour if it doesn't exist
    let flavour_dir = target_dir(flavour);
    std::fs::create_dir_all(&flavour_dir).map_err(|e| {
        color_eyre::eyre::eyre!(
            "Failed to create target directory {}: {}",
            flavour_dir.display(),
            e
        )
    })?;

    // Move bytecode file from target/ to target/flavour/
    let source_bytecode = PathBuf::from("target").join(format!("{}.json", pkg_name));
    let dest_bytecode = get_bytecode_path(pkg_name, flavour);

    if source_bytecode.exists() {
        std::fs::rename(&source_bytecode, &dest_bytecode).map_err(|e| {
            color_eyre::eyre::eyre!(
                "Failed to move {} to {}: {}",
                source_bytecode.display(),
                dest_bytecode.display(),
                e
            )
        })?;
        debug!(
            "Moved bytecode: {} -> {}",
            source_bytecode.display(),
            dest_bytecode.display()
        );
    }

    // Move witness file from target/ to target/flavour/
    let source_witness = PathBuf::from("target").join(format!("{}.gz", pkg_name));
    let dest_witness = get_witness_path(pkg_name, flavour);

    if source_witness.exists() {
        std::fs::rename(&source_witness, &dest_witness).map_err(|e| {
            color_eyre::eyre::eyre!(
                "Failed to move {} to {}: {}",
                source_witness.display(),
                dest_witness.display(),
                e
            )
        })?;
        debug!(
            "Moved witness: {} -> {}",
            source_witness.display(),
            dest_witness.display()
        );
    }

    Ok(())
}

/// Validate that required files exist for a given operation
pub fn validate_files_exist<P: AsRef<Path>>(files: &[P]) -> Result<()> {
    let mut missing_files = Vec::new();

    for file_path in files {
        if !file_path.as_ref().exists() {
            missing_files.push(file_path.as_ref().display().to_string());
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

/// Simplified Nargo.toml configuration structure
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TomlConfig {
    Package {
        package: PackageMetadata,
    },
    Workspace {
        #[allow(dead_code)]
        workspace: WorkspaceMetadata,
    },
}

#[derive(Debug, Deserialize)]
struct PackageMetadata {
    name: Option<String>,
    #[serde(alias = "type")]
    #[allow(dead_code)]
    package_type: Option<String>,
    #[allow(dead_code)]
    version: Option<String>,
    #[allow(dead_code)]
    authors: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct WorkspaceMetadata {
    #[allow(dead_code)]
    members: Vec<String>,
    #[serde(alias = "default-member")]
    #[allow(dead_code)]
    default_member: Option<String>,
}
