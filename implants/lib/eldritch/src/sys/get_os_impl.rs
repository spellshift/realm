use anyhow::Result;
use starlark::collections::SmallMap;
use starlark::const_frozen_string;
use starlark::values::Heap;
use starlark::values::dict::Dict;

#[derive(Debug)]
struct OsInfo {
    arch:           String,
    desktop_env:    String,
    distro:         String,
    platform:       String,
}

pub fn get_os(starlark_heap: &Heap) -> Result<Dict> {

    let cmd_res = handle_get_os()?;

    let res = SmallMap::new();
    let mut dict_res = Dict::new(res);
    let arch_value = starlark_heap.alloc_str(&cmd_res.arch);
    dict_res.insert_hashed(const_frozen_string!("arch").to_value().get_hashed()?, arch_value.to_value());

    let desktop_env_value = starlark_heap.alloc_str(&cmd_res.desktop_env);
    dict_res.insert_hashed(const_frozen_string!("desktop_env").to_value().get_hashed()?, desktop_env_value.to_value());

    let distro = starlark_heap.alloc_str(&cmd_res.distro);
    dict_res.insert_hashed(const_frozen_string!("distro").to_value().get_hashed()?, distro.to_value());

    let platform = starlark_heap.alloc_str(&cmd_res.platform);
    dict_res.insert_hashed(const_frozen_string!("platform").to_value().get_hashed()?, platform.to_value());

    Ok(dict_res)
}

fn handle_get_os() -> Result<OsInfo> {
    return Ok(OsInfo { 
        arch:           whoami::arch().to_string(),
        desktop_env:    whoami::desktop_env().to_string(),
        distro:         whoami::distro().to_string(),
        platform:       whoami::platform().to_string(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sys_get_os() -> anyhow::Result<()>{

        let test_heap = Heap::new();
        let res = get_os(&test_heap)?;
        println!("{}", res.to_string());
        assert!(res.to_string().contains(r#""arch": "x86_64""#));
        Ok(())
    }
}