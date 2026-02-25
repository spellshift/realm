
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use eldritch_core::Value;
use spin::RwLock;

#[cfg(not(target_os = "solaris"))]
use sysinfo::{Pid, PidExt, ProcessExt, System, SystemExt};

#[cfg(target_os = "solaris")]
use crate::std::solaris_proc;

pub fn info(pid: Option<i64>) -> Result<BTreeMap<String, Value>, String> {
    #[cfg(not(target_os = "solaris"))]
    {
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
    {
        use libc;

        let target_pid = pid
            .map(|p| p as i32)
            .unwrap_or_else(|| unsafe { libc::getpid() });

        let info = solaris_proc::read_psinfo(target_pid)
            .map_err(|e| format!("Failed to read process info for {}: {}", target_pid, e))?;

        let mut map = BTreeMap::new();
        map.insert("pid".to_string(), Value::Int(info.pr_pid as i64));
        map.insert("ppid".to_string(), Value::Int(info.pr_ppid as i64));
        map.insert("uid".to_string(), Value::Int(info.pr_uid as i64));
        map.insert("gid".to_string(), Value::Int(info.pr_gid as i64));

        map.insert("virtual_memory_usage".to_string(), Value::Int((info.pr_size * 1024) as i64));
        map.insert("memory_usage".to_string(), Value::Int((info.pr_rssize * 1024) as i64));

        let name = String::from_utf8_lossy(&info.pr_fname)
            .trim_matches(char::from(0))
            .to_string();
        map.insert("name".to_string(), Value::String(name.clone()));

        let args = String::from_utf8_lossy(&info.pr_psargs)
            .trim_matches(char::from(0))
            .to_string();
        map.insert("cmd".to_string(), Value::String(args));

        // Attempt exe and cwd via /proc links
        if let Ok(path) = std::fs::read_link(format!("/proc/{}/path/a.out", target_pid)) {
             map.insert("exe".to_string(), Value::String(path.to_string_lossy().into_owned()));
        } else {
             map.insert("exe".to_string(), Value::String(name));
        }

        if let Ok(cwd) = std::fs::read_link(format!("/proc/{}/path/cwd", target_pid)) {
             map.insert("cwd".to_string(), Value::String(cwd.to_string_lossy().into_owned()));
        } else {
             map.insert("cwd".to_string(), Value::String("/".to_string()));
        }

        map.insert("root".to_string(), Value::String("/".to_string()));
        map.insert("status".to_string(), Value::String("Unknown".to_string()));

        let mut env_map = BTreeMap::new();
        if pid.is_none() {
            for (key, val) in std::env::vars() {
                env_map.insert(Value::String(key), Value::String(val));
            }
        }
        map.insert("environ".to_string(), Value::Dictionary(Arc::new(RwLock::new(env_map))));

        Ok(map)
    }
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
            // Just check it exists and is accessible
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
