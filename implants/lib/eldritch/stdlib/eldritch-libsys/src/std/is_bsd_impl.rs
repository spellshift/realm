use anyhow::Result;

pub fn is_bsd() -> Result<bool> {
    if cfg!(any(
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )) {
        return Ok(true);
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_bsd() {
        let res = is_bsd().unwrap();
        let expected = cfg!(any(
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ));
        assert_eq!(res, expected);
    }
}
