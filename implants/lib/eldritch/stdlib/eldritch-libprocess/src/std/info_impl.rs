use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use eldritch_core::Value;
use spin::RwLock;
use sysinfo::{Pid, PidExt, ProcessExt, System, SystemExt};

pub fn info(pid: Option<i64>) -> Result<BTreeMap<String, Value>, String> {
    let mut sys = System::new();
    sys.refresh_processes();
    sys.refresh_users_list();

    let target_pid = pid
        .map(|p| p as usize)
        .unwrap_or_else(|| ::std::process::id() as usize);
    let pid_struct = Pid::from(target_pid);

    if let Some(process) = sys.process(pid_struct) {
        let mut map = BTreeMap::new();
        map.insert("pid".to_string(), Value::Int(target_pid as i64));
        map.insert(
            "name".to_string(),
            Value::String(process.name().to_string()),
        );
        map.insert("cmd".to_string(), Value::String(process.cmd().join(" ")));
        map.insert(
            "exe".to_string(),
            Value::String(process.exe().display().to_string()),
        );

        let mut env_map = BTreeMap::new();
        for env_str in process.environ() {
            if let Some((key, val)) = env_str.split_once('=') {
                env_map.insert(
                    Value::String(key.to_string()),
                    Value::String(val.to_string()),
                );
            }
        }
        map.insert(
            "environ".to_string(),
            Value::Dictionary(Arc::new(RwLock::new(env_map))),
        );

        map.insert(
            "cwd".to_string(),
            Value::String(process.cwd().display().to_string()),
        );
        map.insert(
            "root".to_string(),
            Value::String(process.root().display().to_string()),
        );
        map.insert(
            "memory_usage".to_string(),
            Value::Int(process.memory() as i64),
        );
        map.insert(
            "virtual_memory_usage".to_string(),
            Value::Int(process.virtual_memory() as i64),
        );

        if let Some(ppid) = process.parent() {
            map.insert("ppid".to_string(), Value::Int(ppid.as_u32() as i64));
        } else {
            map.insert("ppid".to_string(), Value::None);
        }

        map.insert(
            "status".to_string(),
            Value::String(process.status().to_string()),
        );
        map.insert(
            "start_time".to_string(),
            Value::Int(process.start_time() as i64),
        );
        map.insert(
            "run_time".to_string(),
            Value::Int(process.run_time() as i64),
        );

        #[cfg(not(windows))]
        {
            if let Some(gid) = process.group_id() {
                map.insert("gid".to_string(), Value::Int(*gid as i64));
            }
            if let Some(uid) = process.user_id() {
                map.insert("uid".to_string(), Value::Int(**uid as i64));
            }
        }

        Ok(map)
    } else {
        Err(format!("Process {target_pid} not found"))
    }
}

#[cfg(target_os = "solaris")]
pub fn info(pid: Option<i64>) -> Result<BTreeMap<String, Value>, String> {
    use std::fs;
    use std::process::Command;

    let target_pid = pid
        .map(|p| p as usize)
        .unwrap_or_else(|| ::std::process::id() as usize);

    // ps -p <pid> -o pid,ppid,uid,gid,user,comm,vsz,rss,s,stime,args
    let output = Command::new("ps")
        .args(&[
            "-p",
            &target_pid.to_string(),
            "-o",
            "pid,ppid,uid,gid,user,comm,vsz,rss,s,stime,args",
        ])
        .output()
        .map_err(|e| format!("Failed to execute ps: {}", e))?;

    if !output.status.success() {
        return Err(format!("Process {} not found or ps failed", target_pid));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    if lines.len() < 2 {
        return Err(format!("Process {} not found", target_pid));
    }

    let parts: Vec<&str> = lines[1].split_whitespace().collect();
    // We expect at least 10 fields before args start (since args is last and can have spaces)
    if parts.len() < 10 {
        return Err(format!(
            "Failed to parse ps output for process {}",
            target_pid
        ));
    }

    let mut map = BTreeMap::new();
    map.insert("pid".to_string(), Value::Int(target_pid as i64));

    if let Ok(ppid) = parts[1].parse::<i64>() {
        map.insert("ppid".to_string(), Value::Int(ppid));
    }

    if let Ok(uid) = parts[2].parse::<i64>() {
        map.insert("uid".to_string(), Value::Int(uid));
    }

    if let Ok(gid) = parts[3].parse::<i64>() {
        map.insert("gid".to_string(), Value::Int(gid));
    }

    map.insert("user".to_string(), Value::String(parts[4].to_string()));
    map.insert("name".to_string(), Value::String(parts[5].to_string()));

    if let Ok(vsz) = parts[6].parse::<i64>() {
        map.insert("virtual_memory_usage".to_string(), Value::Int(vsz * 1024));
    }

    if let Ok(rss) = parts[7].parse::<i64>() {
        map.insert("memory_usage".to_string(), Value::Int(rss * 1024));
    }

    map.insert("status".to_string(), Value::String(parts[8].to_string()));

    // parts[9] is stime.
    // parts[10..] is args.
    let args = parts[10..].join(" ");
    map.insert("cmd".to_string(), Value::String(args));

    // Attempt to get exe path via /proc
    if let Ok(path) = fs::read_link(format!("/proc/{}/path/a.out", target_pid)) {
        map.insert(
            "exe".to_string(),
            Value::String(path.to_string_lossy().into_owned()),
        );
    } else {
        map.insert("exe".to_string(), Value::String(parts[5].to_string()));
    }

    // Attempt to get cwd via pwdx
    let pwdx_output = Command::new("pwdx")
        .arg(&target_pid.to_string())
        .output();

    if let Ok(output) = pwdx_output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Format: "1234: /path/to/cwd"
        if let Some(pos) = stdout.find(": ") {
            let cwd = stdout[pos + 2..].trim();
            map.insert("cwd".to_string(), Value::String(cwd.to_string()));
        } else {
            map.insert("cwd".to_string(), Value::String("/".to_string()));
        }
    } else {
        map.insert("cwd".to_string(), Value::String("/".to_string()));
    }

    map.insert("root".to_string(), Value::String("/".to_string()));

    // Environ via pargs -e
    let mut env_map = BTreeMap::new();
    let pargs_output = Command::new("pargs")
        .args(&["-e", &target_pid.to_string()])
        .output();

    if let Ok(output) = pargs_output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("envp[") {
                if let Some(pos) = line.find(": ") {
                    let env_entry = &line[pos + 2..];
                    if let Some((key, val)) = env_entry.split_once('=') {
                        env_map.insert(
                            Value::String(key.to_string()),
                            Value::String(val.to_string()),
                        );
                    }
                }
            }
        }
    }

    map.insert(
        "environ".to_string(),
        Value::Dictionary(Arc::new(RwLock::new(env_map))),
    );

    Ok(map)
}

#[cfg(all(test, feature = "stdlib"))]
mod tests {
    use super::super::ProcessLibrary;
    use super::super::StdProcessLibrary;
    use eldritch_core::Value;

    #[test]
    fn test_std_process_info_and_name() {
        let lib = StdProcessLibrary;
        let my_pid = ::std::process::id() as i64;

        let info = lib.info(Some(my_pid)).unwrap();
        assert_eq!(info.get("pid"), Some(&Value::Int(my_pid)));
        assert!(info.contains_key("name"));
        assert!(info.contains_key("cmd"));
        assert!(info.contains_key("exe"));
        assert!(info.contains_key("environ"));

        if let Some(Value::Dictionary(env_dict)) = info.get("environ") {
            let env_map = env_dict.read();
            // We can't guarantee any specific variable exists across all platforms, but we can check it's a dict.
            // On unix/windows PATH or HOME/USERPROFILE usually exists.
            // But just checking it is a Dictionary is enough per the request "assert_eq(type(proc['env']), dict)"
            // Using >= 0 is always true for usize, effectively we just want to ensure we could access the map.
            // So we'll just check that it's a valid map structure which we already did by matching Value::Dictionary.
            // Let's print the length just to use the variable and avoid warnings if we don't assert anything.
            let _ = env_map.len();
        } else {
            panic!("environ is not a dictionary");
        }

        assert!(info.contains_key("cwd"));
        assert!(info.contains_key("root"));
        assert!(info.contains_key("memory_usage"));
        assert!(info.contains_key("virtual_memory_usage"));
        assert!(info.contains_key("ppid"));
        assert!(info.contains_key("status"));
        assert!(info.contains_key("start_time"));
        assert!(info.contains_key("run_time"));

        #[cfg(not(windows))]
        {
            assert!(info.contains_key("uid"));
            assert!(info.contains_key("gid"));
        }

        let name = lib.name(my_pid).unwrap();
        assert!(!name.is_empty());

        // Check consistency
        if let Some(Value::String(info_name)) = info.get("name") {
            assert_eq!(info_name, &name);
        } else {
            panic!("name in info is not a string");
        }
    }
}
