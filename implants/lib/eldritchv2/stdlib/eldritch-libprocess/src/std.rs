use super::ProcessLibrary;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;
use sysinfo::{Pid, PidExt, ProcessExt, Signal, System, SystemExt, UserExt};

#[derive(Default, Debug)]
#[eldritch_library_impl(ProcessLibrary)]
pub struct StdProcessLibrary;

impl ProcessLibrary for StdProcessLibrary {
    fn info(&self, pid: Option<i64>) -> Result<BTreeMap<String, Value>, String> {
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
            map.insert(
                "environ".to_string(),
                Value::String(process.environ().join(",")),
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

    fn kill(&self, pid: i64) -> Result<(), String> {
        if !System::IS_SUPPORTED {
            return Err("System not supported".to_string());
        }

        let mut sys = System::new();
        sys.refresh_processes();

        if let Some(process) = sys.process(Pid::from(pid as usize)) {
            if process.kill_with(Signal::Kill).unwrap_or(false) {
                Ok(())
            } else {
                Err(format!("Failed to kill process {pid}"))
            }
        } else {
            Err(format!("Process {pid} not found"))
        }
    }

    fn list(&self) -> Result<Vec<BTreeMap<String, Value>>, String> {
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
            map.insert("username".to_string(), Value::String(user_name.to_string()));

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

    fn name(&self, pid: i64) -> Result<String, String> {
        if !System::IS_SUPPORTED {
            return Err("System not supported".to_string());
        }
        let mut sys = System::new();
        sys.refresh_processes();

        if let Some(process) = sys.process(Pid::from(pid as usize)) {
            Ok(process.name().to_string())
        } else {
            Err(format!("Process {pid} not found"))
        }
    }

    #[cfg(target_os = "freebsd")]
    fn netstat(&self) -> Result<Vec<BTreeMap<String, Value>>, String> {
        Err("Not implemented for FreeBSD".to_string())
    }

    #[cfg(not(target_os = "freebsd"))]
    fn netstat(&self) -> Result<Vec<BTreeMap<String, Value>>, String> {
        let mut out = Vec::new();
        if let Ok(listeners) = listeners::get_all() {
            for l in listeners {
                let mut map = BTreeMap::new();
                map.insert("socket_type".to_string(), Value::String("TCP".to_string()));
                map.insert(
                    "local_address".to_string(),
                    Value::String(l.socket.ip().to_string()),
                );
                map.insert("local_port".to_string(), Value::Int(l.socket.port() as i64));
                map.insert("pid".to_string(), Value::Int(l.process.pid as i64));
                out.push(map);
            }
        }
        Ok(out)
    }
}

#[cfg(all(test, feature = "stdlib"))]
mod tests {

    use super::*;
    use ::std::process::Command;
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
            assert!(process.contains_key("username"));
            assert!(process.contains_key("status"));
        }
    }

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

    #[test]
    fn test_std_process_kill() {
        // Spawn a sleep process
        let mut cmd = Command::new("sleep");
        cmd.arg("10");

        // Handle windows
        #[cfg(windows)]
        let mut cmd = Command::new("ping");
        #[cfg(windows)]
        cmd.args(["-n", "10", "127.0.0.1"]);

        if let Ok(mut child) = cmd.spawn() {
            let pid = child.id() as i64;
            let lib = StdProcessLibrary;

            // Wait a bit for process to start?
            ::std::thread::sleep(::std::time::Duration::from_millis(100));

            // Check if it exists
            assert!(lib.name(pid).is_ok());

            // Kill it
            lib.kill(pid).unwrap();

            // Wait for it to die
            let _ = child.wait();
        } else {
            // If sleep command not found, skip?
        }
    }

    #[test]
    fn test_std_process_netstat() {
        let lib = StdProcessLibrary;
        // netstat relies on system permissions and open ports, so we just check it doesn't crash
        // and returns a result (even empty).
        let res = lib.netstat();
        assert!(res.is_ok());
    }

    #[test]
    fn test_std_process_errors() {
        let lib = StdProcessLibrary;
        // Using a very large PID that shouldn't exist
        let invalid_pid = 999999999;

        assert!(lib.info(Some(invalid_pid)).is_err());
        assert!(lib.name(invalid_pid).is_err());
        assert!(lib.kill(invalid_pid).is_err());
    }
}
