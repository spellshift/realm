use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs

// Test running `./golem ./working_dir/tomes/hello_world.tome`
#[test]
fn test_golem_main_basic_non_interactive() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("golem")?;

    cmd.arg("working_dir/tomes/hello_world.tome");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("[\"HELLO\"]"));

    Ok(())
}


// Test running `./golem ./working_dir/tomes/eldritch_test.tome`
#[test]
fn test_golem_main_basic_eldritch_non_interactive() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("golem")?;

    cmd.arg("working_dir/tomes/eldritch_test.tome");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("[\"[\\\"append\\\", \\\"copy\\\", \\\"download\\\", \\\"exists\\\", \\\"hash\\\", \\\"is_dir\\\", \\\"is_file\\\", \\\"mkdir\\\", \\\"read\\\", \\\"remove\\\", \\\"rename\\\", \\\"replace\\\", \\\"replace_all\\\", \\\"timestomp\\\", \\\"write\\\"]\"]"));

    Ok(())
}


// Test running `echo "dir(file)" | ./golem` for interactive mode.
#[test]
fn test_golem_main_basic_interactive() -> anyhow::Result<()> {
    // let mut cmd: Command = Command::cargo_bin("golem")?;

    let mut cmd: Command = Command::cargo_bin("golem")?;
    // let mut child = Command::new(cmd)?;

    // let mut child = Command::new("cargo")
    //     .stdin(Stdio::piped())
    //     .stdout(Stdio::piped())
    //     .spawn()?;

    // let child_stdin = child.stdin.as_mut().unwrap();
    // child_stdin.write_all("test_var = 'hello'\nprint(test_var)".as_bytes())?;

    // drop(child_stdin);

    // let output = child.wait_with_output()?;

    // let left_assert = format!("{:?}", output);
    // assert_eq!(left_assert, "test".to_string());

    cmd.assert()
        .success();
        // .stdout(predicate::str::contains("[\"[\\\"append\\\", \\\"copy\\\", \\\"download\\\", \\\"exists\\\", \\\"hash\\\", \\\"is_dir\\\", \\\"is_file\\\", \\\"mkdir\\\", \\\"read\\\", \\\"remove\\\", \\\"rename\\\", \\\"replace\\\", \\\"replace_all\\\", \\\"timestomp\\\", \\\"write\\\"]\"]"));

    Ok(())
}
