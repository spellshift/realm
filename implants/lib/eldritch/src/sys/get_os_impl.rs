use anyhow::Result;
use eldritch_types::operating_system_type::OperatingSystemType;

pub fn get_os() -> Result<OperatingSystemType> {
    return Ok(OperatingSystemType { 
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
        let res = get_os()?;
        println!("{}", res.to_string());
        assert!(res.to_string().contains(r#"arch: x86_64"#));
        Ok(())
    }
}