use super::super::insert_dict_kv;
use super::CommandOutput;
use anyhow::{Context, Result};
use starlark::{
    collections::SmallMap,
    const_frozen_string,
    values::{dict::Dict, Heap},
};
use std::collections::HashMap;
use std::process::Command;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
use {
    nix::sys::wait::wait,
    nix::unistd::{fork, setsid, ForkResult},
    std::process::{exit, Stdio},
};

pub fn exec(
    starlark_heap: &'_ Heap,
    path: String,
    args: Vec<String>,
    disown: Option<bool>,
    env_vars: Option<SmallMap<String, String>>,
) -> Result<Dict<'_>> {
    let mut env_vars_map = HashMap::new();
    if let Some(e) = env_vars {
        for (k, v) in e {
            env_vars_map.insert(k, v);
        }
    }

    let should_disown = disown.unwrap_or(false);

    let cmd_res = handle_exec(path, args, env_vars_map, should_disown)?;

    let res = SmallMap::new();
    let mut dict_res = Dict::new(res);
    insert_dict_kv!(dict_res, starlark_heap, "stdout", cmd_res.stdout, String);
    insert_dict_kv!(dict_res, starlark_heap, "stderr", cmd_res.stderr, String);
    insert_dict_kv!(dict_res, starlark_heap, "status", cmd_res.status, i32);

    Ok(dict_res)
}

fn handle_exec(
    path: String,
    args: Vec<String>,
    env_vars: HashMap<String, String>,
    disown: bool,
) -> Result<CommandOutput> {
    if !disown {
        let res = Command::new(path).args(args).envs(env_vars).output()?;

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
            let _ = Command::new(path)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .args(args)
                .envs(env_vars)
                .spawn();

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
                        let _res = Command::new(path)
                            .args(args)
                            .envs(env_vars)
                            .stdin(Stdio::null())
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .spawn()?;
                        exit(0);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path, process, thread, time};

    use sysinfo::{PidExt, ProcessExt, System, SystemExt};
    use tempfile::NamedTempFile;

    fn init_logging() {
        let _ = pretty_env_logger::formatted_timed_builder()
            .filter_level(log::LevelFilter::Info)
            .parse_env("IMIX_LOG")
            .try_init();
    }

    fn get_zombie_child_processes(cur_pid: u32) -> Result<Vec<String>> {
        log::debug!("{:?}", cur_pid);
        if !System::IS_SUPPORTED {
            return Err(anyhow::anyhow!(
                "This OS isn't supported for process functions.
             Pleases see sysinfo docs for a full list of supported systems.
             https://docs.rs/sysinfo/0.23.5/sysinfo/index.html#supported-oses\n\n"
            ));
        }

        let mut final_res: Vec<String> = Vec::new();
        let mut sys = System::new();
        sys.refresh_processes();
        sys.refresh_users_list();

        for (pid, process) in sys.processes() {
            let mut tmp_ppid = 0;
            if process.parent().is_some() {
                tmp_ppid = process
                    .parent()
                    .context(format!("Failed to get parent process for {}", pid))?
                    .as_u32();
            }
            if tmp_ppid == cur_pid
                && process.status() == sysinfo::ProcessStatus::Zombie
                && process.exe().to_str() == Some("")
            {
                log::debug!("{:?}", process);
                final_res.push(process.name().to_string());
            }
        }
        Ok(final_res)
    }

    use super::*;
    #[test]
    #[ignore] // TODO: Remove this, but it's confusing Jules
    fn test_sys_exec_current_user() -> anyhow::Result<()> {
        if cfg!(target_os = "linux")
            || cfg!(target_os = "ios")
            || cfg!(target_os = "android")
            || cfg!(target_os = "freebsd")
            || cfg!(target_os = "openbsd")
            || cfg!(target_os = "netbsd")
        {
            let res = handle_exec(
                String::from("/bin/sh"),
                vec![String::from("-c"), String::from("id -u")],
                HashMap::new(),
                false,
            )?
            .stdout;
            let mut bool_res = false;
            if res == "1001\n" || res == "0\n" {
                bool_res = true;
            }
            assert!(bool_res);
        } else if cfg!(target_os = "macos") {
            let res = handle_exec(
                String::from("/bin/echo"),
                vec![String::from("hello")],
                HashMap::new(),
                false,
            )?
            .stdout;
            assert_eq!(res, "hello\n");
        } else if cfg!(target_os = "windows") {
            let res = handle_exec(
                String::from("C:\\Windows\\System32\\cmd.exe"),
                vec![String::from("/c"), String::from("whoami")],
                HashMap::new(),
                false,
            )?
            .stdout
            .to_lowercase();
            let mut bool_res = false;
            if res.contains("runneradmin") || res.contains("administrator") || res.contains("user")
            {
                bool_res = true;
            }
            assert!(bool_res);
        }
        Ok(())
    }
    #[test]
    fn test_sys_exec_complex_linux() -> anyhow::Result<()> {
        if cfg!(target_os = "linux")
            || cfg!(target_os = "ios")
            || cfg!(target_os = "macos")
            || cfg!(target_os = "android")
            || cfg!(target_os = "freebsd")
            || cfg!(target_os = "openbsd")
            || cfg!(target_os = "netbsd")
        {
            let res = handle_exec(
                String::from("/bin/sh"),
                 vec![
                    String::from("-c"),
                    String::from("cat /etc/passwd | awk '{print $1}' | grep -E '^root:' | awk -F \":\" '{print $3}'")
                    ],
            HashMap::new(),
            false)?.stdout;
            assert_eq!(res, "0\n");
        }
        Ok(())
    }

    #[test]
    fn test_sys_exec_disown_linux() -> anyhow::Result<()> {
        if cfg!(target_os = "linux")
            || cfg!(target_os = "ios")
            || cfg!(target_os = "macos")
            || cfg!(target_os = "android")
            || cfg!(target_os = "freebsd")
            || cfg!(target_os = "openbsd")
            || cfg!(target_os = "netbsd")
        {
            let tmp_file = NamedTempFile::new()?;
            let path = String::from(tmp_file.path().to_str().unwrap());
            tmp_file.close()?;

            let _res = handle_exec(
                String::from("/bin/sh"),
                vec![String::from("-c"), format!("touch {}", path.clone())],
                HashMap::new(),
                true,
            )?;
            thread::sleep(time::Duration::from_secs(2));

            println!("{:?}", path.clone().as_str());
            assert!(Path::new(path.clone().as_str()).exists());

            let _ = fs::remove_file(path.as_str());
        }
        Ok(())
    }

    #[test]
    #[ignore] // TODO: Remove this, but it's confusing Jules
    fn test_sys_exec_disown_no_defunct() -> anyhow::Result<()> {
        init_logging();

        if cfg!(target_os = "linux")
            || cfg!(target_os = "ios")
            || cfg!(target_os = "macos")
            || cfg!(target_os = "android")
            || cfg!(target_os = "freebsd")
            || cfg!(target_os = "openbsd")
            || cfg!(target_os = "netbsd")
        {
            let _res = handle_exec(
                String::from("/bin/sleep"),
                vec!["1".to_string()],
                HashMap::new(),
                true,
            )?;
            // Make sure our test process has no zombies
            let res = get_zombie_child_processes(process::id())?;
            assert_eq!(res.len(), 0);
        }
        Ok(())
    }

    #[test]
    fn test_sys_exec_complex_windows() -> anyhow::Result<()> {
        if cfg!(target_os = "windows") {
            let res = handle_exec(
                String::from("C:\\Windows\\System32\\cmd.exe"),
                vec![
                    String::from("/c"),
                    String::from("echo admin | findstr /i admin"),
                ],
                HashMap::new(),
                false,
            )?
            .stdout
            .to_lowercase();
            assert!(res.contains("admin"));
        }
        Ok(())
    }
}
