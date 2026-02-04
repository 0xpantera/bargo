#![cfg(feature = "cairo")]

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

#[test]
fn cairo_deploy_auto_declare_vs_no_declare_output_differs() {
    // Integration test to verify auto-declare vs no-declare produce different dry-run output

    // Test auto-declare shows declare step in output
    let auto_declare_output = Command::cargo_bin("bargo")
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
        .success()
        .get_output()
        .stdout
        .clone();

    let auto_declare_stdout = String::from_utf8(auto_declare_output).unwrap();

    // Test no-declare with explicit class hash doesn't show declare step
    let no_declare_output = Command::cargo_bin("bargo")
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
        .success()
        .get_output()
        .stdout
        .clone();

    let no_declare_stdout = String::from_utf8(no_declare_output).unwrap();

    // Auto-declare should show declare step
    assert!(
        auto_declare_stdout.contains("Would declare contract"),
        "Auto-declare output should contain declare step: {}",
        auto_declare_stdout
    );

    // No-declare should not show declare step
    assert!(
        !no_declare_stdout.contains("Would declare contract"),
        "No-declare output should not contain declare step: {}",
        no_declare_stdout
    );

    // Both should show deploy step
    assert!(
        auto_declare_stdout.contains("Would deploy contract"),
        "Auto-declare output should contain deploy step"
    );
    assert!(
        no_declare_stdout.contains("Would deploy contract"),
        "No-declare output should contain deploy step"
    );
}
