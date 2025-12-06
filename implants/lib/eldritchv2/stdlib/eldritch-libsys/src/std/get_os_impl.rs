use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use anyhow::Result;

#[derive(Debug)]
struct OsInfo {
    arch: String,
    desktop_env: String,
    distro: String,
    platform: String,
}

pub fn get_os() -> Result<BTreeMap<String, String>> {
    let cmd_res = handle_get_os()?;

    let mut dict_res = BTreeMap::new();
    dict_res.insert("arch".to_string(), cmd_res.arch);
    dict_res.insert("desktop_env".to_string(), cmd_res.desktop_env);
    dict_res.insert("distro".to_string(), cmd_res.distro);
    dict_res.insert("platform".to_string(), cmd_res.platform);

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
        let res = get_os()?;
        let res_str = format!("{res:?}");
        println!("{res_str}");
        #[cfg(target_arch = "x86_64")]
        assert!(res_str.contains(r#""arch": "x86_64""#));
        #[cfg(target_arch = "aarch64")]
        assert!(res_str.contains(r#""arch": "arm64""#));
        Ok(())
    }
}
