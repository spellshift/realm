use alloc::collections::BTreeMap;
use alloc::string::String;
use anyhow::Result;
use std::env;

pub fn get_env() -> Result<BTreeMap<String, String>> {
    let mut dict_res = BTreeMap::new();

    for (key, val) in env::vars() {
        dict_res.insert(key, val);
    }

    Ok(dict_res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::env;

    #[test]
    fn test_get_env() -> Result<()> {
        unsafe {
            env::set_var("FOO", "BAR");
        }
        let res = get_env()?;
        let val = res.get("FOO").unwrap();
        assert_eq!(val, "BAR");
        Ok(())
    }
}
