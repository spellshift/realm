use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use anyhow::Result;
use eldritch_core::Value;
use spin::RwLock;
use std::process;
use sysinfo::{Pid, ProcessExt, System, SystemExt, UserExt};

pub fn get_user() -> Result<BTreeMap<String, Value>> {
    let mut dict_res = BTreeMap::new();
    let mut dict_user: BTreeMap<Value, Value> = BTreeMap::new();

    let sys = System::new_all();
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
        dict_user.insert(Value::String("uid".to_string()), Value::Int(**uid as i64));

        let user = match sys.get_user_by_id(uid) {
            Some(user) => user,
            None => return Err(anyhow::anyhow!("Failed to get user")),
        };
        dict_user.insert(
            Value::String("name".to_string()),
            Value::String(user.name().to_string()),
        );
        dict_user.insert(
            Value::String("gid".to_string()),
            Value::Int(*user.group_id() as i64),
        );

        let groups: Vec<Value> = user
            .groups()
            .iter()
            .map(|g| Value::String(g.clone()))
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
            dict_euser.insert(Value::String("uid".to_string()), Value::Int(**euid as i64));

            let euser = match sys.get_user_by_id(euid) {
                Some(euser) => euser,
                None => return Err(anyhow::anyhow!("Failed to get euser")),
            };
            dict_euser.insert(
                Value::String("name".to_string()),
                Value::String(euser.name().to_string()),
            );
            dict_euser.insert(
                Value::String("gid".to_string()),
                Value::Int(*euser.group_id() as i64),
            );

            let egroups: Vec<Value> = euser
                .groups()
                .iter()
                .map(|g| Value::String(g.clone()))
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
            dict_res.insert("gid".to_string(), Value::Int(*gid as i64));

            let egid = match process.effective_group_id() {
                Some(egid) => egid,
                None => return Err(anyhow::anyhow!("Failed to get egid")),
            };
            dict_res.insert("egid".to_string(), Value::Int(*egid as i64));
        }

        dict_res.insert(
            "uid".to_string(),
            Value::Dictionary(Arc::new(RwLock::new(dict_user))),
        );
        return Ok(dict_res);
    }

    #[cfg(unix)]
    {
        // Fallback for systems where sysinfo fails to get the current process (e.g., Solaris)
        let uid = unsafe { libc::getuid() };
        let gid = unsafe { libc::getgid() };
        let euid = unsafe { libc::geteuid() };
        let egid = unsafe { libc::getegid() };

        // sysinfo's User::id() returns a &Uid. On unix, it derefs to the underlying integer type.
        // We cast both to i64 for comparison to avoid type mismatch.
        if let Some(user) = sys
            .users()
            .iter()
            .find(|u| (**u.id() as i64) == (uid as i64))
        {
            dict_user.insert(Value::String("uid".to_string()), Value::Int(uid as i64));
            dict_user.insert(
                Value::String("name".to_string()),
                Value::String(user.name().to_string()),
            );
            dict_user.insert(
                Value::String("gid".to_string()),
                Value::Int(*user.group_id() as i64),
            );

            let groups: Vec<Value> = user
                .groups()
                .iter()
                .map(|g| Value::String(g.clone()))
                .collect();
            dict_user.insert(
                Value::String("groups".to_string()),
                Value::List(Arc::new(RwLock::new(groups))),
            );

            let mut dict_euser: BTreeMap<Value, Value> = BTreeMap::new();
            dict_euser.insert(Value::String("uid".to_string()), Value::Int(euid as i64));

            if let Some(euser) = sys
                .users()
                .iter()
                .find(|u| (**u.id() as i64) == (euid as i64))
            {
                dict_euser.insert(
                    Value::String("name".to_string()),
                    Value::String(euser.name().to_string()),
                );
                dict_euser.insert(
                    Value::String("gid".to_string()),
                    Value::Int(*euser.group_id() as i64),
                );
                let egroups: Vec<Value> = euser
                    .groups()
                    .iter()
                    .map(|g| Value::String(g.clone()))
                    .collect();
                dict_euser.insert(
                    Value::String("groups".to_string()),
                    Value::List(Arc::new(RwLock::new(egroups))),
                );
            } else {
                return Err(anyhow::anyhow!("Failed to get euser"));
            }

            dict_res.insert(
                "euid".to_string(),
                Value::Dictionary(Arc::new(RwLock::new(dict_euser))),
            );
            dict_res.insert("gid".to_string(), Value::Int(gid as i64));
            dict_res.insert("egid".to_string(), Value::Int(egid as i64));

            dict_res.insert(
                "uid".to_string(),
                Value::Dictionary(Arc::new(RwLock::new(dict_user))),
            );

            return Ok(dict_res);
        }
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
