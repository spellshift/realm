use anyhow::Result;

pub fn get_pid() -> Result<u32> {
    Ok(std::process::id())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sys_get_pid() {
        let res = get_pid().unwrap();
        assert_eq!(res, std::process::id());
    }
}
