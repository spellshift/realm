use super::super::insert_dict_kv;
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

pub fn get_os(starlark_heap: &Heap) -> Result<Dict<'_>> {
    let cmd_res = handle_get_os()?;

    let res = SmallMap::new();
    let mut dict_res = Dict::new(res);
    insert_dict_kv!(dict_res, starlark_heap, "arch", &cmd_res.arch, String);
    insert_dict_kv!(
        dict_res,
        starlark_heap,
        "desktop_env",
        &cmd_res.desktop_env,
        String
    );
    insert_dict_kv!(dict_res, starlark_heap, "distro", &cmd_res.distro, String);
    insert_dict_kv!(
        dict_res,
        starlark_heap,
        "platform",
        &cmd_res.platform,
        String
    );

    Ok(dict_res)
}

fn handle_get_os() -> Result<OsInfo> {
    let tmp_platform = whoami::platform().to_string();
    let platform = String::from(match tmp_platform.to_lowercase().as_str() {
        "linux" => "PLATFORM_LINUX",
        "windows" => "PLATFORM_WINDOWS",
        "mac os" => "PLATFORM_MACOS",
        "bsd" => "PLATFORM_BSD",
        _ => tmp_platform.as_str(),
    });

    Ok(OsInfo {
        arch: whoami::arch().to_string(),
        desktop_env: whoami::desktop_env().to_string(),
        distro: whoami::distro().to_string(),
        platform,
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
        #[cfg(target_arch = "x86_64")]
        assert!(res.to_string().contains(r#""arch": "x86_64""#));
        #[cfg(target_arch = "aarch64")]
        assert!(res.to_string().contains(r#""arch": "arm64""#));
        Ok(())
    }
}
