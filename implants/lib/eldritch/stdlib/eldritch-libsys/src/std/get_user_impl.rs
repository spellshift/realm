use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use anyhow::Result;
use eldritch_core::Value;
use spin::RwLock;
use std::process;
use sysinfo::{Pid, System, Users};

pub fn get_user() -> Result<BTreeMap<String, Value>> {
    let mut dict_res = BTreeMap::new();
    let mut dict_user: BTreeMap<Value, Value> = BTreeMap::new();

    let sys = System::new_all();
    let users = Users::new_with_refreshed_list();
    let pid = process::id() as usize;
    if let Some(process) = sys.process(Pid::from(pid)) {
        let uid = match process.user_id() {
            Some(uid) => uid,
            None => return Err(anyhow::anyhow!("Failed to get uid")),
        };
        #[cfg(target_os = "windows")]
        dict_user.insert(
            Value::String("uid".to_string()),
            Value::String(uid.to_string()),
        );

        #[cfg(not(target_os = "windows"))]
        {
            // Try to parse uid as integer, fallback to 0 or error?
            // Uid in sysinfo 0.33 is opaque but implements Display.
            // On unix it might be convertible, but safer to parse string representation if we don't have direct access.
            // However, typical usage expects an integer.
            // Let's try direct deref if it worked before, but it likely won't.
            // We'll use a hack: parse the string representation.
            let uid_i64 = uid.to_string().parse::<i64>().unwrap_or(0);
            dict_user.insert(Value::String("uid".to_string()), Value::Int(uid_i64));
        }

        let user = match users.iter().find(|u| u.id() == uid) {
            Some(user) => user,
            None => return Err(anyhow::anyhow!("Failed to get user")),
        };
        dict_user.insert(
            Value::String("name".to_string()),
            Value::String(user.name().to_string()),
        );
        dict_user.insert(
            Value::String("gid".to_string()),
            Value::Int(user.group_id().to_string().parse::<i64>().unwrap_or(0)),
        );

        let groups: Vec<Value> = user
            .groups()
            .iter()
            .map(|g| Value::String(g.name().to_string()))
            .collect();
        dict_user.insert(
            Value::String("groups".to_string()),
            Value::List(Arc::new(RwLock::new(groups))),
        );

        #[cfg(not(target_os = "windows"))]
        {
            let mut dict_euser: BTreeMap<Value, Value> = BTreeMap::new();
            let euid = match process.effective_user_id() {
                Some(euid) => euid,
                None => return Err(anyhow::anyhow!("Failed to get euid")),
            };
            dict_euser.insert(
                Value::String("uid".to_string()),
                Value::Int(euid.to_string().parse::<i64>().unwrap_or(0)),
            );

            let euser = match users.iter().find(|u| u.id() == euid) {
                Some(euser) => euser,
                None => return Err(anyhow::anyhow!("Failed to get euser")),
            };
            dict_euser.insert(
                Value::String("name".to_string()),
                Value::String(euser.name().to_string()),
            );
            dict_euser.insert(
                Value::String("gid".to_string()),
                Value::Int(euser.group_id().to_string().parse::<i64>().unwrap_or(0)),
            );

            let egroups: Vec<Value> = euser
                .groups()
                .iter()
                .map(|g| Value::String(g.name().to_string()))
                .collect();
            dict_euser.insert(
                Value::String("groups".to_string()),
                Value::List(Arc::new(RwLock::new(egroups))),
            );

            dict_res.insert(
                "euid".to_string(),
                Value::Dictionary(Arc::new(RwLock::new(dict_euser))),
            );

            let gid = match process.group_id() {
                Some(gid) => gid,
                None => return Err(anyhow::anyhow!("Failed to get gid")),
            };
            dict_res.insert(
                "gid".to_string(),
                Value::Int(gid.to_string().parse::<i64>().unwrap_or(0)),
            );

            let egid = match process.effective_group_id() {
                Some(egid) => egid,
                None => return Err(anyhow::anyhow!("Failed to get egid")),
            };
            dict_res.insert(
                "egid".to_string(),
                Value::Int(egid.to_string().parse::<i64>().unwrap_or(0)),
            );
        }

        dict_res.insert(
            "uid".to_string(),
            Value::Dictionary(Arc::new(RwLock::new(dict_user))),
        );
        return Ok(dict_res);
    }
    Err(anyhow::anyhow!("Failed to obtain process information"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sys_get_user() -> anyhow::Result<()> {
        let res = get_user()?;
        let keys: Vec<&String> = res.keys().collect();
        assert!(keys.contains(&&"uid".to_string()));
        if !cfg!(target_os = "windows") {
            assert!(keys.contains(&&"euid".to_string()));
            assert!(keys.contains(&&"egid".to_string()));
            assert!(keys.contains(&&"gid".to_string()));
        }
        Ok(())
    }
}
