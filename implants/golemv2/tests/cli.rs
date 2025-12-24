use assert_cmd::cargo_bin; // Find our binary
use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::io::prelude::*;
use std::process::{Command, Stdio}; // Run programs
use std::str;

const GOLEM_CLI_TEST_DIR: &str = "../../bin/golem_cli_test";
// Test running `./golem ./nonexistentdir/run.tome`
#[test]
fn test_golem_main_file_not_found() -> anyhow::Result<()> {
    let mut cmd = Command::new(cargo_bin!("golemv2"));
    cmd.arg("nonexistentdir/run.eldritch");
    #[cfg(target_os = "linux")]
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error: No such file or asset"));
    #[cfg(target_os = "windows")]
    cmd.assert().failure().stderr(predicate::str::contains(
        "Error: The system cannot find the path specified. (os error 3)",
    ));

    Ok(())
}
// Test running `./golem ../../bin/golem_cli_test/syntax_fail.tome`
#[test]
fn test_golem_main_syntax_fail() -> anyhow::Result<()> {
    let mut cmd = Command::new(cargo_bin!("golemv2"));

    cmd.arg(format!(
        "{GOLEM_CLI_TEST_DIR}_shadow/syntax_fail/main.eldritch"
    ));
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains(r#"Parser Error"#.to_string()));

    Ok(())
}
// Test running `./golem ../../bin/golem_cli_test/valid_tome/main.eldritch`
#[test]
fn test_golem_main_basic_non_interactive() -> anyhow::Result<()> {
    let mut cmd = Command::new(cargo_bin!("golemv2"));

    cmd.arg(format!("{GOLEM_CLI_TEST_DIR}/valid_tome/main.eldritch"));
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"HELLO"#))
        .stdout(predicate::str::contains(r#""append", "compress""#));

    Ok(())
}

// Test running `./golem ../../bin/golem_cli_test/eldritch_test.tome`
#[test]
fn test_golem_main_basic_async() -> anyhow::Result<()> {
    let mut cmd = Command::new(cargo_bin!("golemv2"));

    cmd.arg(format!("{GOLEM_CLI_TEST_DIR}/download_test/main.eldritch"));
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"OKAY!"#));
    Ok(())
}

// Test running `./golem -a ../../bin/golem_cli_test/`
#[test]
fn test_golem_main_loaded_files() -> anyhow::Result<()> {
    let mut cmd = Command::new(cargo_bin!("golemv2"));
    cmd.arg("-a");
    cmd.arg(GOLEM_CLI_TEST_DIR);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"["append", "compress""#));
    Ok(())
}

// Test running `./golem -a ../../bin/golem_cli_test/ -e`
#[test]
fn test_golem_main_loaded_and_embdedded_files() -> anyhow::Result<()> {
    let mut cmd = Command::new(cargo_bin!("golemv2"));
    cmd.arg("-e");
    cmd.arg("-a");
    cmd.arg(GOLEM_CLI_TEST_DIR);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            r#"hello from an embedded shell script"#,
        ))
        .stdout(predicate::str::contains(r#"hello from an asset directory"#));
    Ok(())
}

// Test running `./golem -a ./../bin/golem_cli_test/ -a ./../bin/golem_cli_test_shadow/`. Should fail
#[test]
fn test_golem_main_loaded_files_shadow() -> anyhow::Result<()> {
    let mut cmd = Command::new(cargo_bin!("golemv2"));
    cmd.arg("-a");
    cmd.arg(GOLEM_CLI_TEST_DIR);
    cmd.arg("-a");
    cmd.arg(format!("{GOLEM_CLI_TEST_DIR}_shadow"));
    cmd.assert().failure().stderr(predicate::str::contains(
        r#"Error: asset collision detected."#,
    ));
    Ok(())
}

// Test running `./golem` to execute embedded scripts.
#[test]
fn test_golem_main_embedded_files() -> anyhow::Result<()> {
    let mut cmd = Command::new(cargo_bin!("golemv2"));

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#"This script just prints"#));

    Ok(())
}
