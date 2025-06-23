use color_eyre::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flavour {
    Bb,
    Starknet,
}

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

fn parse_package_name(nargo_toml_path: &Path) -> Result<String> {
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
                    color_eyre::eyre::eyre!("Could not determine package name from workspace directory")
                })?;
            Ok(dir_name)
        }
    }
}

pub(super) fn target_dir(flavour: Flavour) -> PathBuf {
    match flavour {
        Flavour::Bb => PathBuf::from("target/bb"),
        Flavour::Starknet => PathBuf::from("target/starknet"),
    }
}

pub(super) fn workspace_target() -> PathBuf {
    target_dir(Flavour::Bb)
}

pub fn get_bytecode_path(pkg_name: &str, flavour: Flavour) -> PathBuf {
    target_dir(flavour).join(format!("{}.json", pkg_name))
}

pub fn get_witness_path(pkg_name: &str, flavour: Flavour) -> PathBuf {
    target_dir(flavour).join(format!("{}.gz", pkg_name))
}

pub fn get_proof_path(flavour: Flavour) -> PathBuf {
    target_dir(flavour).join("proof")
}

pub fn get_vk_path(flavour: Flavour) -> PathBuf {
    target_dir(flavour).join("vk")
}

pub fn get_public_inputs_path(flavour: Flavour) -> PathBuf {
    target_dir(flavour).join("public_inputs")
}

pub fn organize_build_artifacts(pkg_name: &str, flavour: Flavour) -> Result<()> {
    let flavour_dir = target_dir(flavour);
    std::fs::create_dir_all(&flavour_dir).map_err(|e| {
        color_eyre::eyre::eyre!(
            "Failed to create target directory {}: {}",
            flavour_dir.display(),
            e
        )
    })?;

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

pub(super) fn organize_bb_artifacts(flavour: Flavour) -> Result<()> {
    let flavour_dir = target_dir(flavour);
    std::fs::create_dir_all(&flavour_dir).map_err(|e| {
        color_eyre::eyre::eyre!(
            "Failed to create target directory {}: {}",
            flavour_dir.display(),
            e
        )
    })?;

    let source_proof = PathBuf::from("target/proof");
    let dest_proof = get_proof_path(flavour);

    if source_proof.exists() {
        std::fs::rename(&source_proof, &dest_proof).map_err(|e| {
            color_eyre::eyre::eyre!(
                "Failed to move {} to {}: {}",
                source_proof.display(),
                dest_proof.display(),
                e
            )
        })?;
        debug!(
            "Moved proof: {} -> {}",
            source_proof.display(),
            dest_proof.display()
        );
    }

    let source_vk = PathBuf::from("target/vk");
    let dest_vk = get_vk_path(flavour);

    if source_vk.exists() {
        std::fs::rename(&source_vk, &dest_vk).map_err(|e| {
            color_eyre::eyre::eyre!(
                "Failed to move {} to {}: {}",
                source_vk.display(),
                dest_vk.display(),
                e
            )
        })?;
        debug!("Moved vk: {} -> {}", source_vk.display(), dest_vk.display());
    }

    let source_public_inputs = PathBuf::from("target/public_inputs");
    let dest_public_inputs = get_public_inputs_path(flavour);

    if source_public_inputs.exists() {
        std::fs::rename(&source_public_inputs, &dest_public_inputs).map_err(|e| {
            color_eyre::eyre::eyre!(
                "Failed to move {} to {}: {}",
                source_public_inputs.display(),
                dest_public_inputs.display(),
                e
            )
        })?;
        debug!(
            "Moved public_inputs: {} -> {}",
            source_public_inputs.display(),
            dest_public_inputs.display()
        );
    }

    Ok(())
}

pub fn validate_files_exist(files: &[PathBuf]) -> Result<()> {
    let mut missing_files = Vec::new();

    for file in files {
        if !file.exists() {
            missing_files.push(file);
        }
    }

    if !missing_files.is_empty() {
        let missing_list = missing_files
            .iter()
            .map(|p| format!("  - {}", p.display()))
            .collect::<Vec<_>>()
            .join("\n");

        return Err(color_eyre::eyre::eyre!(
            "Required files are missing:\n{}\n\nTry running `bargo build` first.",
            missing_list
        ));
    }

    Ok(())
}

pub fn needs_rebuild(pkg_name: &str) -> Result<bool> {
    let current_dir = std::env::current_dir()?;
    let project_root = find_project_root(&current_dir)?;

    let bytecode_path = get_bytecode_path(pkg_name, Flavour::Bb);
    let witness_path = get_witness_path(pkg_name, Flavour::Bb);

    if !bytecode_path.exists() || !witness_path.exists() {
        debug!("Target files don't exist, rebuild needed");
        return Ok(true);
    }

    let bytecode_time = std::fs::metadata(&bytecode_path)?.modified()?;
    let witness_time = std::fs::metadata(&witness_path)?.modified()?;
    let target_time = bytecode_time.min(witness_time);

    let nargo_toml = project_root.join("Nargo.toml");
    if nargo_toml.exists() {
        let nargo_time = std::fs::metadata(&nargo_toml)?.modified()?;
        if nargo_time > target_time {
            debug!("Nargo.toml is newer than target files, rebuild needed");
            return Ok(true);
        }
    }

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

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TomlConfig {
    Package { package: PackageMetadata },
    Workspace { workspace: WorkspaceMetadata },
}

#[derive(Debug, Deserialize)]
struct PackageMetadata {
    name: Option<String>,
    #[serde(alias = "type")]
    package_type: Option<String>,
    version: Option<String>,
    authors: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct WorkspaceMetadata {
    members: Vec<String>,
    #[serde(alias = "default-member")]
    default_member: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_project(temp_dir: &TempDir, name: &str) -> PathBuf {
        let project_dir = temp_dir.path().join("test_project");
        fs::create_dir_all(&project_dir).unwrap();

        let nargo_toml = format!(
            r#"[package]
name = "{}"
type = "bin"
authors = ["test"]
"#,
            name
        );

        fs::write(project_dir.join("Nargo.toml"), nargo_toml).unwrap();

        let src_dir = project_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("main.nr"), "fn main() {}").unwrap();

        project_dir
    }

    #[test]
    fn test_find_project_root() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project(&temp_dir, "test_pkg");
        let src_dir = project_dir.join("src");

        let found_root = find_project_root(&src_dir).unwrap();
        assert_eq!(found_root, project_dir);
    }

    #[test]
    fn test_parse_package_name() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project(&temp_dir, "my_package");
        let nargo_toml = project_dir.join("Nargo.toml");

        let name = parse_package_name(&nargo_toml).unwrap();
        assert_eq!(name, "my_package");
    }

    #[test]
    fn test_get_package_name_with_override() {
        let override_name = "override_pkg".to_string();
        let name = get_package_name(Some(&override_name)).unwrap();
        assert_eq!(name, "override_pkg");
    }

    #[test]
    fn test_path_helpers() {
        assert_eq!(get_bytecode_path("test", Flavour::Bb), PathBuf::from("target/bb/test.json"));
        assert_eq!(get_witness_path("test", Flavour::Bb), PathBuf::from("target/bb/test.gz"));
        assert_eq!(get_proof_path(Flavour::Bb), PathBuf::from("target/bb/proof"));
        assert_eq!(get_vk_path(Flavour::Bb), PathBuf::from("target/bb/vk"));
    }

    #[test]
    fn test_needs_rebuild_no_target() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = create_test_project(&temp_dir, "test_pkg");

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&project_dir).unwrap();

        let needs_rebuild = needs_rebuild("test_pkg").unwrap();
        assert!(needs_rebuild);

        std::env::set_current_dir(original_dir).unwrap();
    }
}

