use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::Result;

pub fn now() -> Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::prelude::*;

    #[test]
    fn test_now() {
        let now_out = now().unwrap() as i64;
        let now_test = Local::now().timestamp();
        assert!((now_out - now_test).abs() <= 250);
    }
}
