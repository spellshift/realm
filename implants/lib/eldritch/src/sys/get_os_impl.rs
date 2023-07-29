use anyhow::Result;
use starlark::collections::SmallMap;
use starlark::const_frozen_string;
use starlark::values::dict::Dict;
use starlark::values::Heap;

#[derive(Debug)]
struct OsInfo {
    arch: String,
    desktop_env: String,
    distro: String,
    platform: String,
}

pub fn get_os(starlark_heap: &Heap) -> Result<Dict> {
    let cmd_res = handle_get_os()?;

    let res = SmallMap::new();
    let mut dict_res = Dict::new(res);
    let arch_value = starlark_heap.alloc_str(&cmd_res.arch);
    dict_res.insert_hashed(
        const_frozen_string!("arch")
            .to_value()
            .get_hashed()
            .unwrap(),
        arch_value.to_value(),
    );

    let desktop_env_value = starlark_heap.alloc_str(&cmd_res.desktop_env);
    dict_res.insert_hashed(
        const_frozen_string!("desktop_env")
            .to_value()
            .get_hashed()
            .unwrap(),
        desktop_env_value.to_value(),
    );

    let distro = starlark_heap.alloc_str(&cmd_res.distro);
    dict_res.insert_hashed(
        const_frozen_string!("distro")
            .to_value()
            .get_hashed()
            .unwrap(),
        distro.to_value(),
    );

    let platform = starlark_heap.alloc_str(&cmd_res.platform);
    dict_res.insert_hashed(
        const_frozen_string!("platform")
            .to_value()
            .get_hashed()
            .unwrap(),
        platform.to_value(),
    );

    Ok(dict_res)
}

fn handle_get_os() -> Result<OsInfo> {
    Ok(OsInfo {
        arch: whoami::arch().to_string(),
        desktop_env: whoami::desktop_env().to_string(),
        distro: whoami::distro(),
        platform: whoami::platform().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sys_get_os() -> anyhow::Result<()> {
        let test_heap = Heap::new();
        let res = get_os(&test_heap)?;
        println!("{}", res);
        assert!(res.to_string().contains(r#""arch": "x86_64""#));
        Ok(())
    }
}
