use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use eldritch_core::Value;

#[cfg(not(target_os = "solaris"))]
use sysinfo::{PidExt, ProcessExt, System, SystemExt, UserExt};

#[cfg(not(target_os = "solaris"))]
pub fn list() -> Result<Vec<BTreeMap<String, Value>>, String> {
    if !System::IS_SUPPORTED {
        return Err("System not supported".to_string());
    }

    let mut sys = System::new();
    sys.refresh_processes();
    sys.refresh_users_list();

    let mut list = Vec::new();
    for (pid, process) in sys.processes() {
        let mut map = BTreeMap::new();
        map.insert("pid".to_string(), Value::Int(pid.as_u32() as i64));

        if let Some(ppid) = process.parent() {
            map.insert("ppid".to_string(), Value::Int(ppid.as_u32() as i64));
        } else {
            map.insert("ppid".to_string(), Value::Int(0));
        }

        map.insert(
            "status".to_string(),
            Value::String(process.status().to_string()),
        );

        let user_name = process
            .user_id()
            .and_then(|uid| sys.get_user_by_id(uid))
            .map(|u| u.name())
            .unwrap_or("???");
        map.insert(
            "principal".to_string(),
            Value::String(user_name.to_string()),
        );

        map.insert(
            "path".to_string(),
            Value::String(process.exe().to_string_lossy().into_owned()),
        );
        map.insert(
            "command".to_string(),
            Value::String(process.cmd().join(" ")),
        );
        map.insert(
            "cwd".to_string(),
            Value::String(process.cwd().to_string_lossy().into_owned()),
        );
        map.insert(
            "environ".to_string(),
            Value::String(process.environ().join(" ")),
        );
        map.insert(
            "name".to_string(),
            Value::String(process.name().to_string()),
        );

        list.push(map);
    }
    Ok(list)
}

#[cfg(target_os = "solaris")]
pub fn list() -> Result<Vec<BTreeMap<String, Value>>, String> {
    use std::process::Command;

    // ps -e -o pid,ppid,user,comm,s,args
    let output = Command::new("ps")
        .args(&["-e", "-o", "pid,ppid,user,comm,s,args"])
        .output()
        .map_err(|e| format!("Failed to execute ps: {}", e))?;

    if !output.status.success() {
        return Err("ps command failed".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut list = Vec::new();

    // skip header
    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        // min 5 fields + args
        if parts.len() < 6 {
            continue;
        }

        let mut map = BTreeMap::new();
        // Parse fields
        // pid
        if let Ok(pid) = parts[0].parse::<i64>() {
            map.insert("pid".to_string(), Value::Int(pid));
        } else {
            continue;
        }

        // ppid
        if let Ok(ppid) = parts[1].parse::<i64>() {
            map.insert("ppid".to_string(), Value::Int(ppid));
        }

        // user
        map.insert(
            "principal".to_string(),
            Value::String(parts[2].to_string()),
        );

        // comm (name)
        map.insert("name".to_string(), Value::String(parts[3].to_string()));
        map.insert("path".to_string(), Value::String(parts[3].to_string())); // ps doesn't give full path easily without /proc

        // status
        map.insert("status".to_string(), Value::String(parts[4].to_string()));

        // args (command)
        let args = parts[5..].join(" ");
        map.insert("command".to_string(), Value::String(args.clone()));

        // Missing: cwd, environ (too expensive to get for all processes)
        map.insert("cwd".to_string(), Value::String("".to_string()));
        map.insert("environ".to_string(), Value::String("".to_string()));

        list.push(map);
    }

    Ok(list)
}

#[cfg(all(test, feature = "stdlib"))]
mod tests {
    use super::super::ProcessLibrary;
    use super::super::StdProcessLibrary;
    use eldritch_core::Value;

    #[test]
    fn test_std_process_list() {
        let lib = StdProcessLibrary;
        let list = lib.list().unwrap();
        assert!(!list.is_empty());

        // Ensure current process is in list
        let my_pid = ::std::process::id() as i64;
        let my_process = list.iter().find(|p| {
            if let Some(Value::Int(pid)) = p.get("pid") {
                *pid == my_pid
            } else {
                false
            }
        });
        assert!(my_process.is_some(), "Current process not found in list");

        if let Some(process) = my_process {
            // Check for expected fields
            assert!(process.contains_key("pid"));
            assert!(process.contains_key("ppid"));
            assert!(process.contains_key("name"));
            assert!(process.contains_key("path"));
            assert!(process.contains_key("principal"));
            assert!(process.contains_key("status"));
        }
    }
}
