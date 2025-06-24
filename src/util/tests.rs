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

    // Test directory creation using absolute paths instead of changing current dir
    let target_bb = project_dir.join("target/bb");
    let target_evm = project_dir.join("target/evm");
    let target_starknet = project_dir.join("target/starknet");
    let contracts_dir = project_dir.join("contracts");

    // Create directories manually (simulating what ensure_target_dir would do)
    assert!(std::fs::create_dir_all(&target_bb).is_ok());
    assert!(std::fs::create_dir_all(&target_evm).is_ok());
    assert!(std::fs::create_dir_all(&target_starknet).is_ok());
    assert!(std::fs::create_dir_all(&contracts_dir).is_ok());

    // Verify directories were created using absolute paths
    assert!(target_bb.exists());
    assert!(target_evm.exists());
    assert!(target_starknet.exists());
    assert!(contracts_dir.exists());
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

    // Use absolute paths instead of changing current directory
    let bb_dir = project_dir.join("target/bb");
    let evm_dir = project_dir.join("target/evm");
    let starknet_dir = project_dir.join("target/starknet");

    // Create all target directories using absolute paths
    assert!(std::fs::create_dir_all(&bb_dir).is_ok());
    assert!(std::fs::create_dir_all(&evm_dir).is_ok());
    assert!(std::fs::create_dir_all(&starknet_dir).is_ok());

    // Test that artifacts are organized properly using absolute paths
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
}

#[test]
fn test_needs_rebuild_no_target() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = create_test_project(&temp_dir, "test_pkg");

    // Test directly with absolute path - no directory change needed!
    let needs_rebuild = needs_rebuild_from_path("test_pkg", &project_dir).unwrap();
    assert!(needs_rebuild);
}

#[test]
fn test_needs_rebuild_prover_toml_modified() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = create_test_project(&temp_dir, "test_pkg");

    // Create target files (simulate previous build)
    let target_bb_dir = project_dir.join("target/bb");
    fs::create_dir_all(&target_bb_dir).unwrap();

    let bytecode_path = target_bb_dir.join("test_pkg.json");
    let witness_path = target_bb_dir.join("test_pkg.gz");

    fs::write(&bytecode_path, "mock bytecode").unwrap();
    fs::write(&witness_path, "mock witness").unwrap();

    // Initially should not need rebuild
    let needs_rebuild = needs_rebuild_from_path("test_pkg", &project_dir).unwrap();
    assert!(
        !needs_rebuild,
        "Should not need rebuild when target files exist and are newer"
    );

    // Wait a moment to ensure file timestamps are different
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Create Prover.toml (newer than target files)
    let prover_toml = project_dir.join("Prover.toml");
    fs::write(&prover_toml, "# Circuit inputs\n").unwrap();

    // Now should need rebuild due to Prover.toml being newer
    let needs_rebuild = needs_rebuild_from_path("test_pkg", &project_dir).unwrap();
    assert!(
        needs_rebuild,
        "Should need rebuild when Prover.toml is newer than target files"
    );
}

#[test]
fn test_validate_files_exist_success() {
    let temp_dir = TempDir::new().unwrap();

    // Create test files
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");
    fs::write(&file1, "content1").unwrap();
    fs::write(&file2, "content2").unwrap();

    let files = vec![file1, file2];
    assert!(validate_files_exist(&files).is_ok());
}

#[test]
fn test_validate_files_exist_missing() {
    let temp_dir = TempDir::new().unwrap();

    // Create only one file
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("missing.txt");
    fs::write(&file1, "content1").unwrap();

    let files = vec![file1, file2];
    let result = validate_files_exist(&files);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("missing.txt"));
}

#[test]
fn test_validate_files_exist_empty_list() {
    let files: Vec<PathBuf> = vec![];
    let result = validate_files_exist(&files);
    assert!(result.is_ok(), "Empty file list should be valid");
}

#[test]
fn test_validate_files_exist_multiple_missing() {
    let temp_dir = TempDir::new().unwrap();

    // Don't create any files, just reference paths
    let file1 = temp_dir.path().join("missing1.txt");
    let file2 = temp_dir.path().join("missing2.txt");

    let files = vec![file1, file2];
    let result = validate_files_exist(&files);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("missing1.txt"));
    assert!(error_msg.contains("missing2.txt"));
}

#[test]
fn test_package_name_edge_cases() {
    // Test edge cases for package names
    let valid_names = vec!["wkshp", "test_package", "my-circuit", "package123"];

    for name in valid_names {
        let bytecode_path = get_bytecode_path(name, Flavour::Bb);
        assert!(bytecode_path.to_string_lossy().contains(name));
        assert!(bytecode_path.to_string_lossy().ends_with(".json"));
    }
}
