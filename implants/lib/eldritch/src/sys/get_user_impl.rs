use super::super::insert_dict_kv;
use anyhow::Result;
use starlark::collections::SmallMap;
use starlark::const_frozen_string;
use starlark::values::dict::Dict;
use starlark::values::Heap;
use std::process;
use sysinfo::{Pid, ProcessExt, System, SystemExt, UserExt};

pub fn get_user(starlark_heap: &'_ Heap) -> Result<Dict<'_>> {
    let res = SmallMap::new();
    let mut dict_res = Dict::new(res);
    let user = SmallMap::new();
    let mut dict_user = Dict::new(user);

    let sys = System::new_all();
    let pid = process::id() as usize;
    if let Some(process) = sys.process(Pid::from(pid)) {
        let uid = match process.user_id() {
            Some(uid) => uid,
            None => return Err(anyhow::anyhow!("Failed to get uid")),
        };
        #[cfg(target_os = "windows")]
        insert_dict_kv!(dict_user, starlark_heap, "uid", uid.to_string(), String);

        #[cfg(not(target_os = "windows"))]
        insert_dict_kv!(dict_user, starlark_heap, "uid", **uid, u32);

        let user = match sys.get_user_by_id(uid) {
            Some(user) => user,
            None => return Err(anyhow::anyhow!("Failed to get user")),
        };
        insert_dict_kv!(
            dict_user,
            starlark_heap,
            "name",
            user.name().to_string(),
            String
        );
        insert_dict_kv!(dict_user, starlark_heap, "gid", *user.group_id(), u32);
        insert_dict_kv!(
            dict_user,
            starlark_heap,
            "groups",
            Vec::from(user.groups()),
            Vec<_>
        );

        #[cfg(not(target_os = "windows"))]
        {
            let euser = SmallMap::new();
            let mut dict_euser = Dict::new(euser);
            let euid = match process.effective_user_id() {
                Some(euid) => euid,
                None => return Err(anyhow::anyhow!("Failed to get euid")),
            };
            insert_dict_kv!(dict_euser, starlark_heap, "uid", **euid, u32);

            let euser = match sys.get_user_by_id(euid) {
                Some(euser) => euser,
                None => return Err(anyhow::anyhow!("Failed to get euser")),
            };
            insert_dict_kv!(dict_euser, starlark_heap, "name", euser.name(), String);
            insert_dict_kv!(dict_euser, starlark_heap, "gid", *euser.group_id(), u32);
            insert_dict_kv!(
                dict_euser,
                starlark_heap,
                "groups",
                Vec::from(euser.groups()),
                Vec<_>
            );

            let dict_euser_value = starlark_heap.alloc(dict_euser);
            dict_res.insert_hashed(
                match const_frozen_string!("euid").to_value().get_hashed() {
                    Ok(val) => val,
                    Err(e) => {
                        return Err(anyhow::anyhow!("Failed to alloc euid information: {}", e))
                    }
                },
                dict_euser_value,
            );

            let gid = match process.group_id() {
                Some(gid) => gid,
                None => return Err(anyhow::anyhow!("Failed to get gid")),
            };
            insert_dict_kv!(dict_res, starlark_heap, "gid", *gid, u32);

            let egid = match process.effective_group_id() {
                Some(egid) => egid,
                None => return Err(anyhow::anyhow!("Failed to get egid")),
            };
            insert_dict_kv!(dict_res, starlark_heap, "egid", *egid, u32);
        }
        let dict_user_value = starlark_heap.alloc(dict_user);
        dict_res.insert_hashed(
            match const_frozen_string!("uid").to_value().get_hashed() {
                Ok(val) => val,
                Err(e) => return Err(anyhow::anyhow!("Failed to alloc uid information: {}", e)),
            },
            dict_user_value,
        );
        return Ok(dict_res);
    }
    Err(anyhow::anyhow!("Failed to obtain process information"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use starlark::values::{UnpackValue, Value};

    #[test]
    fn test_sys_get_user() -> anyhow::Result<()> {
        let test_heap = Heap::new();
        let res = get_user(&test_heap)?;
        let keys: Vec<&str> = res.keys().map(|key| key.unpack_str().unwrap()).collect();
        assert!(keys.contains(&"uid"));
        if !cfg!(target_os = "windows") {
            assert!(keys.contains(&"euid"));
            assert!(keys.contains(&"egid"));
            assert!(keys.contains(&"gid"));
        }
        let uid_data: Value<'_> = match res.get(const_frozen_string!("uid").to_value()) {
            Ok(v) => Ok(v),
            Err(err) => Err(err.into_anyhow()),
        }?
        .unwrap();
        let uid_data_map: SmallMap<String, Value<'_>> = SmallMap::unpack_value(uid_data).unwrap();
        let uid_data_keys: Vec<&str> = uid_data_map.keys().map(|key| &key[..]).collect();
        assert!(uid_data_keys.contains(&"uid"));
        assert!(uid_data_keys.contains(&"name"));
        assert!(uid_data_keys.contains(&"gid"));
        assert!(uid_data_keys.contains(&"groups"));
        if !cfg!(target_os = "windows") {
            let euid_data: Value<'_> = match res.get(const_frozen_string!("euid").to_value()) {
                Ok(v) => Ok(v),
                Err(err) => Err(err.into_anyhow()),
            }?
            .unwrap();
            let euid_data_map: SmallMap<String, Value<'_>> =
                SmallMap::unpack_value(euid_data).unwrap();
            let euid_data_keys: Vec<&str> = euid_data_map.keys().map(|key| &key[..]).collect();
            assert!(euid_data_keys.contains(&"uid"));
            assert!(euid_data_keys.contains(&"name"));
            assert!(euid_data_keys.contains(&"gid"));
            assert!(euid_data_keys.contains(&"groups"));
        }
        Ok(())
    }
}
