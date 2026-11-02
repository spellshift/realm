use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use anyhow::Result;
use eldritch_core::Value;
use spin::RwLock;
use sysinfo::{System, SystemExt, UserExt};

pub fn list_users() -> Result<Vec<BTreeMap<String, Value>>> {
    let mut users_list = Vec::new();
    let mut sys = System::new();
    sys.refresh_users_list();

    for user in sys.users() {
        let mut dict_user: BTreeMap<String, Value> = BTreeMap::new();

        dict_user.insert(
            "principal".to_string(),
            Value::String(user.name().to_string()),
        );

        #[cfg(target_os = "windows")]
        {
            // On Windows, UIDs are SIDs (strings)
            dict_user.insert("uid".to_string(), Value::String(user.id().to_string()));
            // GIDs are also likely strings or handled differently on Windows
            dict_user.insert(
                "gid".to_string(),
                Value::String(user.group_id().to_string()),
            );
        }

        #[cfg(not(target_os = "windows"))]
        {
            // On *nix, UIDs are usually integers wrapped in a struct
            // sysinfo usually exposes them as &Uid, which derefs to the underlying integer type
            dict_user.insert("uid".to_string(), Value::Int(**user.id() as i64));
            dict_user.insert("gid".to_string(), Value::Int(*user.group_id() as i64));
        }

        let groups: Vec<Value> = user
            .groups()
            .iter()
            .map(|g| Value::String(g.clone()))
            .collect();

        dict_user.insert(
            "groups".to_string(),
            Value::List(Arc::new(RwLock::new(groups))),
        );

        users_list.push(dict_user);
    }

    Ok(users_list)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_users() -> anyhow::Result<()> {
        let users = list_users()?;
        assert!(!users.is_empty());

        // basic check that we got some user
        let first_user = &users[0];
        assert!(first_user.contains_key("principal"));
        assert!(first_user.contains_key("uid"));
        assert!(first_user.contains_key("gid"));
        assert!(first_user.contains_key("groups"));

        // Check for root user if on linux
        #[cfg(target_os = "linux")]
        {
            let root = users.iter().find(|u| {
                if let Some(Value::String(name)) = u.get("principal") {
                    name == "root"
                } else {
                    false
                }
            });

            // It's possible root is not listed in some environments, but usually it is.
            if let Some(root_user) = root {
                if let Some(Value::Int(uid)) = root_user.get("uid") {
                    assert_eq!(*uid, 0);
                } else {
                    panic!("Root uid should be Int(0)");
                }
            }
        }

        Ok(())
    }
}
