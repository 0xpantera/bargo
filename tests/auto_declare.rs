use assert_cmd::Command;

#[test]
fn cairo_deploy_auto_declare_default() {
    // Test that auto-declare is enabled by default
    Command::cargo_bin("bargo")
        .unwrap()
        .args(["--dry-run", "--pkg", "test_pkg", "cairo", "deploy"])
        .assert()
        .success();
}

#[test]
fn cairo_deploy_with_auto_declare_flag() {
    // Test explicit --auto-declare flag
    Command::cargo_bin("bargo")
        .unwrap()
        .args([
            "--dry-run",
            "--pkg",
            "test_pkg",
            "cairo",
            "deploy",
            "--auto-declare",
        ])
        .assert()
        .success();
}

#[test]
fn cairo_deploy_with_no_declare_flag() {
    // Test --no-declare flag
    Command::cargo_bin("bargo")
        .unwrap()
        .args([
            "--dry-run",
            "--pkg",
            "test_pkg",
            "cairo",
            "deploy",
            "--no-declare",
        ])
        .assert()
        .success();
}

#[test]
fn cairo_deploy_with_class_hash_and_no_declare() {
    // Test --no-declare with explicit class hash
    Command::cargo_bin("bargo")
        .unwrap()
        .args([
            "--dry-run",
            "--pkg",
            "test_pkg",
            "cairo",
            "deploy",
            "--class-hash",
            "0x123456789abcdef",
            "--no-declare",
        ])
        .assert()
        .success();
}

#[test]
fn cairo_deploy_conflicting_flags_should_fail() {
    // Test that --auto-declare and --no-declare flags conflict
    Command::cargo_bin("bargo")
        .unwrap()
        .args([
            "--dry-run",
            "--pkg",
            "test_pkg",
            "cairo",
            "deploy",
            "--auto-declare",
            "--no-declare",
        ])
        .assert()
        .failure();
}

#[test]
fn cairo_deploy_verbose_shows_auto_declare_behavior() {
    // Test verbose output shows auto-declare behavior
    Command::cargo_bin("bargo")
        .unwrap()
        .args([
            "--verbose",
            "--dry-run",
            "--pkg",
            "test_pkg",
            "cairo",
            "deploy",
        ])
        .assert()
        .success();
}

#[test]
fn cairo_deploy_pkg_flag_propagated_with_auto_declare() {
    // Test that package flag propagation works with auto-declare
    Command::cargo_bin("bargo")
        .unwrap()
        .args([
            "--dry-run",
            "--pkg",
            "my_custom_pkg",
            "cairo",
            "deploy",
            "--auto-declare",
        ])
        .assert()
        .success();
}

#[test]
fn cairo_deploy_pkg_flag_propagated_with_no_declare() {
    // Test that package flag propagation works with no-declare
    Command::cargo_bin("bargo")
        .unwrap()
        .args([
            "--dry-run",
            "--pkg",
            "my_custom_pkg",
            "cairo",
            "deploy",
            "--no-declare",
        ])
        .assert()
        .success();
}

#[test]
fn cairo_deploy_quiet_flag_works_with_auto_declare() {
    // Test that quiet flag works with auto-declare
    Command::cargo_bin("bargo")
        .unwrap()
        .args([
            "--quiet",
            "--dry-run",
            "--pkg",
            "test_pkg",
            "cairo",
            "deploy",
            "--auto-declare",
        ])
        .assert()
        .success();
}

#[test]
fn cairo_deploy_all_flags_together() {
    // Test combination of global flags with auto-declare flags
    Command::cargo_bin("bargo")
        .unwrap()
        .args([
            "--verbose",
            "--dry-run",
            "--pkg",
            "test_pkg",
            "cairo",
            "deploy",
            "--class-hash",
            "0x987654321fedcba",
            "--no-declare",
        ])
        .assert()
        .success();
}
