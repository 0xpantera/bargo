use super::*;
use std::fs;
use std::path::PathBuf;
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

    // Create src directory and main.nr
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
    // Test Bb flavour
    assert_eq!(
        get_bytecode_path("test", Flavour::Bb),
        PathBuf::from("target/bb/test.json")
    );
    assert_eq!(
        get_witness_path("test", Flavour::Bb),
        PathBuf::from("target/bb/test.gz")
    );
    assert_eq!(
        get_proof_path(Flavour::Bb),
        PathBuf::from("target/bb/proof")
    );
    assert_eq!(get_vk_path(Flavour::Bb), PathBuf::from("target/bb/vk"));
    assert_eq!(
        get_public_inputs_path(Flavour::Bb),
        PathBuf::from("target/bb/public_inputs")
    );

    // Test EVM flavour
    assert_eq!(
        get_bytecode_path("test", Flavour::Evm),
        PathBuf::from("target/evm/test.json")
    );
    assert_eq!(
        get_witness_path("test", Flavour::Evm),
        PathBuf::from("target/evm/test.gz")
    );
    assert_eq!(
        get_proof_path(Flavour::Evm),
        PathBuf::from("target/evm/proof")
    );
    assert_eq!(get_vk_path(Flavour::Evm), PathBuf::from("target/evm/vk"));
    assert_eq!(
        get_public_inputs_path(Flavour::Evm),
        PathBuf::from("target/evm/public_inputs")
    );

    // Test Starknet flavour
    assert_eq!(
        get_bytecode_path("test", Flavour::Starknet),
        PathBuf::from("target/starknet/test.json")
    );
    assert_eq!(
        get_witness_path("test", Flavour::Starknet),
        PathBuf::from("target/starknet/test.gz")
    );
    assert_eq!(
        get_proof_path(Flavour::Starknet),
        PathBuf::from("target/starknet/proof")
    );
    assert_eq!(
        get_vk_path(Flavour::Starknet),
        PathBuf::from("target/starknet/vk")
    );
    assert_eq!(
        get_public_inputs_path(Flavour::Starknet),
        PathBuf::from("target/starknet/public_inputs")
    );
}

#[test]
fn test_target_dir_all_flavours() {
    assert_eq!(target_dir(Flavour::Bb), PathBuf::from("target/bb"));
    assert_eq!(target_dir(Flavour::Evm), PathBuf::from("target/evm"));
    assert_eq!(
        target_dir(Flavour::Starknet),
        PathBuf::from("target/starknet")
    );
}

#[test]
fn test_directory_creation_all_flavours() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = create_test_project(&temp_dir, "test_project");

    // Change to the test project directory
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&project_dir).unwrap();

    // Test that ensure_target_dir works for all flavours
    assert!(ensure_target_dir(Flavour::Bb).is_ok());
    assert!(ensure_target_dir(Flavour::Evm).is_ok());
    assert!(ensure_target_dir(Flavour::Starknet).is_ok());

    // Verify directories were created (check relative to current directory)
    assert!(std::path::Path::new("target/bb").exists());
    assert!(std::path::Path::new("target/evm").exists());
    assert!(std::path::Path::new("target/starknet").exists());

    // Test contracts directory creation
    assert!(ensure_contracts_dir().is_ok());
    assert!(std::path::Path::new("contracts").exists());

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_flavour_consistency() {
    // Test that all flavours have consistent path structure
    let flavours = [Flavour::Bb, Flavour::Evm, Flavour::Starknet];

    for flavour in flavours.iter() {
        let target = target_dir(*flavour);
        let proof = get_proof_path(*flavour);
        let vk = get_vk_path(*flavour);
        let public_inputs = get_public_inputs_path(*flavour);

        // All paths should be under the target directory
        assert!(proof.starts_with(&target));
        assert!(vk.starts_with(&target));
        assert!(public_inputs.starts_with(&target));

        // All should have consistent naming
        assert_eq!(proof.file_name().unwrap(), "proof");
        assert_eq!(vk.file_name().unwrap(), "vk");
        assert_eq!(public_inputs.file_name().unwrap(), "public_inputs");
    }
}

#[test]
fn test_artifact_organization() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = create_test_project(&temp_dir, "test_project");

    // Change to the test project directory
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&project_dir).unwrap();

    // Create base target directory first
    std::fs::create_dir_all("target").unwrap();

    // Create all target directories
    for flavour in [Flavour::Bb, Flavour::Evm, Flavour::Starknet].iter() {
        ensure_target_dir(*flavour).unwrap();
    }

    // Test that artifacts are organized properly (check relative to current dir)
    let bb_dir = std::path::PathBuf::from("target/bb");
    let evm_dir = std::path::PathBuf::from("target/evm");
    let starknet_dir = std::path::PathBuf::from("target/starknet");

    assert!(bb_dir.exists(), "BB directory should exist: {:?}", bb_dir);
    assert!(
        evm_dir.exists(),
        "EVM directory should exist: {:?}",
        evm_dir
    );
    assert!(
        starknet_dir.exists(),
        "Starknet directory should exist: {:?}",
        starknet_dir
    );

    // They should be separate directories
    assert_ne!(bb_dir, evm_dir);
    assert_ne!(bb_dir, starknet_dir);
    assert_ne!(evm_dir, starknet_dir);

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_needs_rebuild_no_target() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = create_test_project(&temp_dir, "test_pkg");

    // Change to project directory
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&project_dir).unwrap();

    let needs_rebuild = needs_rebuild("test_pkg").unwrap();
    assert!(needs_rebuild);

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
}
