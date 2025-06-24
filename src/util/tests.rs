use super::*;
use std::fs;
use tempfile::TempDir;
use std::path::PathBuf;

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
    assert_eq!(get_bytecode_path("test", Flavour::Bb), PathBuf::from("target/bb/test.json"));
    assert_eq!(get_witness_path("test", Flavour::Bb), PathBuf::from("target/bb/test.gz"));
    assert_eq!(get_proof_path(Flavour::Bb), PathBuf::from("target/bb/proof"));
    assert_eq!(get_vk_path(Flavour::Bb), PathBuf::from("target/bb/vk"));
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
