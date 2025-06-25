use assert_cmd::Command;

#[test]
fn cli_still_builds_and_parses() {
    Command::cargo_bin("bargo")
        .unwrap()
        .args(["--dry-run", "build"])
        .assert()
        .success();
}

#[test]
fn pkg_flag_is_propagated() {
    use predicates::str::contains;

    Command::cargo_bin("bargo")
        .unwrap()
        .args(["--dry-run", "--pkg", "my_pkg", "build"])
        .assert()
        .stdout(contains("--package my_pkg"));
}
