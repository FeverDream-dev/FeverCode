use assert_cmd::Command;

#[test]
fn cli_help_succeeds() {
    Command::cargo_bin("fever")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Fever Code"))
        .stdout(predicates::str::contains("Usage"));
}

#[test]
fn cli_version_subcommand_succeeds() {
    Command::cargo_bin("fever")
        .unwrap()
        .arg("version")
        .assert()
        .success()
        .stdout(predicates::str::contains("fever"));
}

#[test]
fn cli_doctor_runs() {
    Command::cargo_bin("fever")
        .unwrap()
        .arg("doctor")
        .assert()
        .stdout(predicates::str::contains("Fever Doctor"));
}

#[test]
fn cli_config_path_succeeds() {
    Command::cargo_bin("fever")
        .unwrap()
        .args(["config", "--path"])
        .assert()
        .success()
        .stdout(predicates::str::contains("fevercode"));
}

#[test]
fn cli_config_validate_runs() {
    let _ = Command::cargo_bin("fever")
        .unwrap()
        .args(["config", "--validate"])
        .assert();
}

#[test]
fn cli_session_list_runs() {
    Command::cargo_bin("fever")
        .unwrap()
        .args(["session", "list"])
        .assert()
        .success();
}
