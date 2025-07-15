use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("bytestashy").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "CLI to push snippets to ByteStash",
        ));
}

#[test]
fn test_version_command() {
    let mut cmd = Command::cargo_bin("bytestashy").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("bytestashy"));
}

#[test]
fn test_list_help() {
    let mut cmd = Command::cargo_bin("bytestashy").unwrap();
    cmd.args(&["list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Show a paginated list of snippets",
        ));
}

#[test]
fn test_get_help() {
    let mut cmd = Command::cargo_bin("bytestashy").unwrap();
    cmd.args(&["get", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Retrieve a snippet by ID"));
}

#[test]
fn test_login_help() {
    let mut cmd = Command::cargo_bin("bytestashy").unwrap();
    cmd.args(&["login", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Fetches an API token"));
}

#[test]
fn test_no_files_provided() {
    let mut cmd = Command::cargo_bin("bytestashy").unwrap();
    cmd.args(&["create"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Provide at least one file"));
}

#[test]
fn test_nonexistent_file() {
    let mut cmd = Command::cargo_bin("bytestashy").unwrap();
    cmd.args(&["create", "/nonexistent/file.txt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("File does not exist"));
}

#[test]
fn test_path_traversal_protection() {
    let mut cmd = Command::cargo_bin("bytestashy").unwrap();
    cmd.args(&["create", "../../../etc/passwd"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(".."));
}

#[test]
fn test_invalid_url_scheme() {
    let mut cmd = Command::cargo_bin("bytestashy").unwrap();
    cmd.args(&["login", "ftp://example.com"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "URL must use http or https scheme",
        ));
}

#[test]
fn test_list_command_runs() {
    // This test just checks that the list command can be executed
    // It might succeed if there's a valid config, or fail if not logged in
    let mut cmd = Command::cargo_bin("bytestashy").unwrap();
    cmd.arg("list")
        .assert()
        .code(predicate::in_iter(vec![0, 1])); // Either success or failure is acceptable
}
