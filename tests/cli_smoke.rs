use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn help_shows_description() {
    Command::cargo_bin("great")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("The managed AI dev environment"));
}

#[test]
fn version_shows_semver() {
    Command::cargo_bin("great")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn init_help_shows_initialize() {
    Command::cargo_bin("great")
        .unwrap()
        .args(["init", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialize"));
}

#[test]
fn no_args_shows_usage() {
    Command::cargo_bin("great")
        .unwrap()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}
