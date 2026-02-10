use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use eldritch_core::Value;
use sysinfo::{IS_SUPPORTED_SYSTEM, System, Users};

pub fn list() -> Result<Vec<BTreeMap<String, Value>>, String> {
    if !IS_SUPPORTED_SYSTEM {
        return Err("System not supported".to_string());
    }

    let sys = System::new_all();
    let users = Users::new_with_refreshed_list();

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
            .and_then(|uid| users.iter().find(|u| u.id() == uid))
            .map(|u| u.name())
            .unwrap_or("???");
        map.insert("username".to_string(), Value::String(user_name.to_string()));

        map.insert(
            "path".to_string(),
            Value::String(
                process
                    .exe()
                    .map(|e| e.to_string_lossy().into_owned())
                    .unwrap_or_default(),
            ),
        );
        map.insert(
            "command".to_string(),
            Value::String(
                process
                    .cmd()
                    .iter()
                    .map(|c| c.to_string_lossy())
                    .collect::<Vec<_>>()
                    .join(" "),
            ),
        );
        map.insert(
            "cwd".to_string(),
            Value::String(
                process
                    .cwd()
                    .map(|c| c.to_string_lossy().into_owned())
                    .unwrap_or_default(),
            ),
        );
        map.insert(
            "environ".to_string(),
            Value::String(
                process
                    .environ()
                    .iter()
                    .map(|e| e.to_string_lossy())
                    .collect::<Vec<_>>()
                    .join(" "),
            ),
        );
        map.insert(
            "name".to_string(),
            Value::String(process.name().to_string_lossy().into_owned()),
        );

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
            assert!(process.contains_key("username"));
            assert!(process.contains_key("status"));
        }
    }
}
