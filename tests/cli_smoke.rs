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

#[test]
fn check_command_dry_run() {
    Command::cargo_bin("bargo")
        .unwrap()
        .args(["--dry-run", "check"])
        .assert()
        .success();
}

#[test]
fn check_command_pkg_flag_propagated() {
    use predicates::str::contains;

    Command::cargo_bin("bargo")
        .unwrap()
        .args(["--dry-run", "--pkg", "my_pkg", "check"])
        .assert()
        .stdout(contains("--package my_pkg"));
}

#[test]
fn verbose_flag_shows_command() {
    use predicates::str::contains;

    Command::cargo_bin("bargo")
        .unwrap()
        .args(["--verbose", "--dry-run", "check"])
        .assert()
        .stdout(contains("Would run: nargo check"));
}

#[test]
fn all_global_flags_work_together() {
    use predicates::str::contains;

    Command::cargo_bin("bargo")
        .unwrap()
        .args(["--verbose", "--dry-run", "--pkg", "test_pkg", "check"])
        .assert()
        .stdout(contains("--package test_pkg"))
        .stdout(contains("Would run: nargo check"));
}

#[test]
fn clean_command_dry_run() {
    Command::cargo_bin("bargo")
        .unwrap()
        .args(["--dry-run", "clean"])
        .assert()
        .success();
}

#[test]
fn clean_command_dry_run_shows_action() {
    use predicates::str::contains;

    Command::cargo_bin("bargo")
        .unwrap()
        .args(["--dry-run", "clean"])
        .assert()
        .stdout(contains("Would run: rm -rf target/"));
}

#[test]
fn rebuild_command_dry_run() {
    Command::cargo_bin("bargo")
        .unwrap()
        .args(["--dry-run", "--pkg", "test_pkg", "rebuild"])
        .assert()
        .success();
}

#[test]
fn rebuild_command_pkg_flag_propagated() {
    use predicates::str::contains;

    Command::cargo_bin("bargo")
        .unwrap()
        .args(["--dry-run", "--pkg", "my_pkg", "rebuild"])
        .assert()
        .stdout(contains("--package my_pkg"));
}

#[test]
fn cairo_gen_dry_run() {
    Command::cargo_bin("bargo")
        .unwrap()
        .args(["--dry-run", "--pkg", "test_pkg", "cairo", "gen"])
        .assert()
        .success();
}

#[test]
fn cairo_gen_pkg_flag_propagated() {
    Command::cargo_bin("bargo")
        .unwrap()
        .args(["--dry-run", "--pkg", "my_pkg", "cairo", "gen"])
        .assert()
        .success();
}

#[test]
fn evm_gen_dry_run() {
    Command::cargo_bin("bargo")
        .unwrap()
        .args(["--dry-run", "--pkg", "test_pkg", "evm", "gen"])
        .assert()
        .success();
}

#[test]
fn evm_gen_pkg_flag_propagated() {
    Command::cargo_bin("bargo")
        .unwrap()
        .args(["--dry-run", "--pkg", "my_pkg", "evm", "gen"])
        .assert()
        .success();
}
