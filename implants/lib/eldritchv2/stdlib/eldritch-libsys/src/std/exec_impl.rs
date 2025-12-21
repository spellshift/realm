use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use anyhow::{Context, Result};
use eldritch_core::Value;
use std::collections::HashMap;
use std::io::Write; // Required for writing to stdin
use std::process::Command;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
use {
    nix::sys::wait::wait,
    nix::unistd::{ForkResult, fork, setsid},
    std::process::{Stdio, exit},
};

struct CommandOutput {
    stdout: String,
    stderr: String,
    status: i32,
}

pub fn exec(
    path: String,
    args: Vec<String>,
    disown: Option<bool>,
    env_vars: Option<BTreeMap<String, String>>,
    input: Option<String>, // Added input option
) -> Result<BTreeMap<String, Value>> {
    let mut env_vars_map = HashMap::new();
    if let Some(e) = env_vars {
        for (k, v) in e {
            env_vars_map.insert(k, v);
        }
    }

    let should_disown = disown.unwrap_or(false);

    let cmd_res = handle_exec(path, args, env_vars_map, should_disown, input)?;

    let mut dict_res = BTreeMap::new();
    dict_res.insert("stdout".to_string(), Value::String(cmd_res.stdout));
    dict_res.insert("stderr".to_string(), Value::String(cmd_res.stderr));
    dict_res.insert("status".to_string(), Value::Int(cmd_res.status as i64));

    Ok(dict_res)
}

fn handle_exec(
    path: String,
    args: Vec<String>,
    env_vars: HashMap<String, String>,
    disown: bool,
    input: Option<String>, // Added input option
) -> Result<CommandOutput> {
    // Setup stdin configuration, null if no input is given
    let stdinpipe = if input.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    };
    if !disown {
        let mut child = Command::new(path)
            .args(args)
            .envs(env_vars)
            .stdin(stdinpipe)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // If we have input, write it to the pipe
        if let Some(text) = input {
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(text.as_bytes())?;
                // Stdin is closed here when 'stdin' is dropped, sending EOF to the child
            }
        }

        let res = child.wait_with_output()?;
        let res = CommandOutput {
            stdout: String::from_utf8_lossy(&res.stdout).to_string(),
            stderr: String::from_utf8_lossy(&res.stderr).to_string(),
            status: res
                .status
                .code()
                .context("Failed to retrieve status code")?,
        };
        Ok(res)
    } else {
        #[cfg(target_os = "windows")]
        {
            let mut child = Command::new(path)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .stdin(stdinpipe)
                .args(args)
                .envs(env_vars)
                .spawn()?;

            if let Some(text) = input {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(text.as_bytes());
                }
            }

            Ok(CommandOutput {
                stdout: "".to_string(),
                stderr: "".to_string(),
                status: 0,
            })
        }
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
        match unsafe { fork()? } {
            ForkResult::Parent { child } => {
                if child.as_raw() < 0 {
                    return Err(anyhow::anyhow!("Pid was negative. ERR".to_string()));
                }

                let _ = wait();

                Ok(CommandOutput {
                    stdout: "".to_string(),
                    stderr: "".to_string(),
                    status: 0,
                })
            }
            ForkResult::Child => {
                setsid()?;
                match unsafe { fork()? } {
                    ForkResult::Parent { child } => {
                        if child.as_raw() < 0 {
                            return Err(anyhow::anyhow!("Pid was negative. ERR".to_string()));
                        }
                        exit(0);
                    }
                    ForkResult::Child => {
                        let mut res = Command::new(path)
                            .args(args)
                            .envs(env_vars)
                            .stdin(stdinpipe)
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .spawn()?;

                        if let Some(text) = input {
                            if let Some(mut stdin) = res.stdin.take() {
                                let _ = stdin.write_all(text.as_bytes());
                                let _ = stdin.flush();
                            }
                        }
                        exit(0);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exec_simple() -> Result<()> {
        #[cfg(target_os = "windows")]
        let (cmd, args) = (
            "cmd.exe".to_string(),
            vec!["/C".to_string(), "echo".to_string(), "hello".to_string()],
        );
        #[cfg(not(target_os = "windows"))]
        let (cmd, args) = ("echo".to_string(), vec!["hello".to_string()]);

        let res = exec(cmd, args, Some(false), None, None)?;
        assert_eq!(res.get("status").unwrap(), &Value::Int(0));

        let stdout = res.get("stdout").unwrap();
        match stdout {
            Value::String(s) => assert!(s.trim() == "hello"),
            _ => panic!("Expected string stdout"),
        }
        Ok(())
    }

    #[test]
    fn test_exec_env() -> Result<()> {
        #[cfg(target_os = "windows")]
        let (cmd, args) = (
            "cmd.exe".to_string(),
            vec!["/C".to_string(), "echo".to_string(), "%MY_VAR%".to_string()],
        );
        #[cfg(not(target_os = "windows"))]
        let (cmd, args) = (
            "sh".to_string(),
            vec!["-c".to_string(), "echo $MY_VAR".to_string()],
        );

        let mut env = BTreeMap::new();
        env.insert("MY_VAR".to_string(), "my_value".to_string());

        let res = exec(cmd, args, Some(false), Some(env), None)?;
        assert_eq!(res.get("status").unwrap(), &Value::Int(0));

        let stdout = res.get("stdout").unwrap();
        match stdout {
            Value::String(s) => assert!(s.trim() == "my_value"),
            _ => panic!("Expected string stdout"),
        }
        Ok(())
    }

    #[test]
    fn test_exec_input() -> Result<()> {
        #[cfg(target_os = "windows")]
        let (cmd, args) = (
            "cmd.exe".to_string(),
            vec!["/C".to_string(), "echo".to_string(), "hello".to_string()],
        );
        #[cfg(not(target_os = "windows"))]
        let (cmd, input) = ("cat".to_string(), "hello".to_string());

        let res = exec(cmd, Vec::new(), Some(false), None, Some(input))?;
        assert_eq!(res.get("status").unwrap(), &Value::Int(0));

        let stdout = res.get("stdout").unwrap();
        match stdout {
            Value::String(s) => assert!(s.trim() == "hello"),
            _ => panic!("Expected string stdout"),
        }
        Ok(())
    }
}
