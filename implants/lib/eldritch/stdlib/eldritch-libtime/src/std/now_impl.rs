use chrono::Utc;

pub fn now() -> Result<i64, String> {
    Ok(Utc::now().timestamp())
}

#[cfg(test)]
mod tests {
    use super::super::StdTimeLibrary;
    use super::super::TimeLibrary;

    #[test]
    fn test_now() {
        let lib = StdTimeLibrary;
        let ts = lib.now().unwrap();
        assert!(ts > 1600000000);
    }
}
