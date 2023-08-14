use anyhow::Result;
use starlark::collections::SmallMap;
use starlark::values::dict::Dict;
use starlark::values::Heap;
use starlark::{const_frozen_string, values::ValueLike};
use std::process;
use sysinfo::{Pid, ProcessExt, System, SystemExt, UserExt};

pub fn get_user(starlark_heap: &Heap) -> Result<Dict> {
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
        #[cfg(target_os="windows")]
        let uid_value = starlark_heap.alloc(uid.to_string());
        #[cfg(not(target_os="windows"))]
        let uid_value = starlark_heap.alloc(**uid);
        dict_user.insert_hashed(
            match const_frozen_string!("uid").to_value().get_hashed() {
                Ok(val) => val,
                Err(e) => return Err(anyhow::anyhow!("Failed to alloc uid information: {}", e)),
            },
            uid_value.to_value(),
        );
        let user = match sys.get_user_by_id(uid) {
            Some(user) => user,
            None => return Err(anyhow::anyhow!("Failed to get user")),
        };
        let user_name_value = starlark_heap.alloc_str(user.name());
        dict_user.insert_hashed(
            match const_frozen_string!("name").to_value().get_hashed() {
                Ok(val) => val,
                Err(e) => return Err(anyhow::anyhow!("Failed to alloc name information: {}", e)),
            },
            user_name_value.to_value(),
        );
        let user_gid_value = starlark_heap.alloc(*user.group_id());
        dict_user.insert_hashed(
            match const_frozen_string!("gid").to_value().get_hashed() {
                Ok(val) => val,
                Err(e) => return Err(anyhow::anyhow!("Failed to alloc gid information: {}", e)),
            },
            user_gid_value.to_value(),
        );
        let user_groups_value = starlark_heap.alloc(Vec::from(user.groups()));
        dict_user.insert_hashed(
            match const_frozen_string!("groups").to_value().get_hashed() {
                Ok(val) => val,
                Err(e) => return Err(anyhow::anyhow!("Failed to alloc groups information: {}", e)),
            },
            user_groups_value.to_value(),
        );
        #[cfg(not(target_os="windows"))]
        {
            let euser = SmallMap::new();
            let mut dict_euser = Dict::new(euser);
            let euid = match process.effective_user_id() {
                Some(euid) => euid,
                None => return Err(anyhow::anyhow!("Failed to get euid")),
            };
            let euid_value = starlark_heap.alloc(**euid);
            dict_euser.insert_hashed(
                match const_frozen_string!("uid").to_value().get_hashed() {
                    Ok(val) => val,
                    Err(e) => return Err(anyhow::anyhow!("Failed to alloc uid information: {}", e)),
                },
                euid_value.to_value(),
            );
            let euser = match sys.get_user_by_id(euid) {
                Some(euser) => euser,
                None => return Err(anyhow::anyhow!("Failed to get euser")),
            };
            let euser_name_value = starlark_heap.alloc_str(euser.name());
            dict_euser.insert_hashed(
                match const_frozen_string!("name").to_value().get_hashed() {
                    Ok(val) => val,
                    Err(e) => return Err(anyhow::anyhow!("Failed to alloc name information: {}", e)),
                },
                euser_name_value.to_value(),
            );
            let euser_gid_value = starlark_heap.alloc(*euser.group_id());
            dict_euser.insert_hashed(
                match const_frozen_string!("gid").to_value().get_hashed() {
                    Ok(val) => val,
                    Err(e) => return Err(anyhow::anyhow!("Failed to alloc gid information: {}", e)),
                },
                euser_gid_value.to_value(),
            );
            let euser_groups_value = starlark_heap.alloc(Vec::from(euser.groups()));
            dict_euser.insert_hashed(
                match const_frozen_string!("groups").to_value().get_hashed() {
                    Ok(val) => val,
                    Err(e) => return Err(anyhow::anyhow!("Failed to alloc groups information: {}", e)),
                },
                euser_groups_value.to_value(),
            );
            let dict_euser_value = starlark_heap.alloc(dict_euser);
            dict_res.insert_hashed(
                match const_frozen_string!("euid").to_value().get_hashed() {
                    Ok(val) => val,
                    Err(e) => return Err(anyhow::anyhow!("Failed to alloc euid information: {}", e)),
                },
                dict_euser_value,
            );
            let gid = match process.group_id() {
                Some(gid) => gid,
                None => return Err(anyhow::anyhow!("Failed to get gid")),
            };
            let gid_value = starlark_heap.alloc(*gid);
            let egid = match process.effective_group_id() {
                Some(egid) => egid,
                None => return Err(anyhow::anyhow!("Failed to get egid")),
            };
            let egid_value = starlark_heap.alloc(*egid);
            dict_res.insert_hashed(
                match const_frozen_string!("gid").to_value().get_hashed() {
                    Ok(val) => val,
                    Err(e) => return Err(anyhow::anyhow!("Failed to alloc gid information: {}", e)),
                },
                gid_value.to_value(),
            );
            dict_res.insert_hashed(
                match const_frozen_string!("egid").to_value().get_hashed() {
                    Ok(val) => val,
                    Err(e) => return Err(anyhow::anyhow!("Failed to alloc egid information: {}", e)),
                },
                egid_value.to_value(),
            );
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
    use starlark::values::{UnpackValue, Value};
    use super::*;

    #[test]
    fn test_sys_get_user() -> anyhow::Result<()>{
        let test_heap = Heap::new();
        let res = get_user(&test_heap)?;
        let keys: Vec<&str> = res.keys().map(|key| key.unpack_str().unwrap()).collect();
        assert!(keys.contains(&"euid"));
        assert!(keys.contains(&"uid"));
        assert!(keys.contains(&"egid"));
        assert!(keys.contains(&"gid"));
        let uid_data: Value<'_> = res.get(const_frozen_string!("uid").to_value())?.unwrap();
        let uid_data_map: SmallMap<String, Value<'_>> = SmallMap::unpack_value(uid_data).unwrap();
        let uid_data_keys: Vec<&str> = uid_data_map.keys().map(|key| &key[..]).collect();
        assert!(uid_data_keys.contains(&"uid"));
        assert!(uid_data_keys.contains(&"name"));
        assert!(uid_data_keys.contains(&"gid"));
        assert!(uid_data_keys.contains(&"groups"));
        let euid_data: Value<'_> = res.get(const_frozen_string!("euid").to_value())?.unwrap();
        let euid_data_map: SmallMap<String, Value<'_>> = SmallMap::unpack_value(euid_data).unwrap();
        let euid_data_keys: Vec<&str> = euid_data_map.keys().map(|key| &key[..]).collect();
        assert!(euid_data_keys.contains(&"uid"));
        assert!(euid_data_keys.contains(&"name"));
        assert!(euid_data_keys.contains(&"gid"));
        assert!(euid_data_keys.contains(&"groups"));
        Ok(())
    }
}