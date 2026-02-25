
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use eldritch_core::Value;

#[cfg(target_os = "solaris")]
use std::fs;

// Shared PsInfo struct and reading logic could be here, or imported if we put it in a common module.
// Since modules are separate files, we can put it in mod.rs and make it pub(crate).

#[cfg(target_os = "solaris")]
use crate::std::solaris_proc;

pub fn list() -> Result<Vec<BTreeMap<alloc::string::String, Value>>, alloc::string::String> {
    #[cfg(not(target_os = "solaris"))]
    {
        use sysinfo::{PidExt, ProcessExt, System, SystemExt, UserExt};

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
    {
        let mut list = Vec::new();
        let proc_dir = fs::read_dir("/proc").map_err(|e| format!("Failed to read /proc: {}", e))?;

        // println!("DEBUG: listing /proc");

        for entry in proc_dir {
            if let Ok(entry) = entry {
                if let Ok(file_name) = entry.file_name().into_string() {
                    // Filter numeric directories only
                    if let Ok(pid) = file_name.parse::<i32>() {
                        match solaris_proc::read_psinfo(pid) {
                            Ok(info) => {
                                let mut map = BTreeMap::new();
                                map.insert("pid".to_string(), Value::Int(info.pr_pid as i64));
                                map.insert("ppid".to_string(), Value::Int(info.pr_ppid as i64));

                                map.insert("principal".to_string(), Value::String(info.pr_uid.to_string()));

                                // Name from psinfo
                                let name = String::from_utf8_lossy(&info.pr_fname)
                                    .trim_matches(char::from(0))
                                    .to_string();
                                map.insert("name".to_string(), Value::String(name.clone()));
                                map.insert("path".to_string(), Value::String(name));

                                // Command args
                                let args = String::from_utf8_lossy(&info.pr_psargs)
                                    .trim_matches(char::from(0))
                                    .to_string();
                                map.insert("command".to_string(), Value::String(args));

                                map.insert("status".to_string(), Value::String("Unknown".to_string()));
                                map.insert("cwd".to_string(), Value::String("".to_string()));
                                map.insert("environ".to_string(), Value::String("".to_string()));

                                list.push(map);
                            },
                            Err(_e) => {
                                // println!("DEBUG: failed to read psinfo for {}: {}", pid, _e);
                            }
                        }
                    }
                }
            }
        }
        Ok(list)
    }
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
