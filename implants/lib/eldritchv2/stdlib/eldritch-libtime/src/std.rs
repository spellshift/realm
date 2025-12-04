use super::TimeLibrary;
use alloc::string::{String, ToString};
use anyhow::Result as AnyhowResult;
use eldritch_macros::eldritch_library_impl;

#[cfg(feature = "stdlib")]
use chrono::{NaiveDateTime, TimeZone, Utc};
#[cfg(feature = "stdlib")]
use std::{thread, time};

#[derive(Debug, Default)]
#[eldritch_library_impl(TimeLibrary)]
pub struct StdTimeLibrary;

impl TimeLibrary for StdTimeLibrary {
    fn format_to_epoch(&self, input: String, format: String) -> Result<i64, String> {
        format_to_epoch_impl(input, format).map_err(|e| e.to_string())
    }

    fn format_to_readable(&self, input: i64, format: String) -> Result<String, String> {
        format_to_readable_impl(input, format).map_err(|e| e.to_string())
    }

    fn now(&self) -> Result<i64, String> {
        Ok(Utc::now().timestamp())
    }

    fn sleep(&self, secs: i64) -> Result<(), String> {
        thread::sleep(time::Duration::from_secs(secs as u64));
        Ok(())
    }
}

// Implementations

fn format_to_epoch_impl(input: String, fmt: String) -> AnyhowResult<i64> {
    // Try to parse as DateTime with timezone first
    if let Ok(dt) = chrono::DateTime::parse_from_str(&input, &fmt) {
        return Ok(dt.timestamp());
    }

    // Fallback to NaiveDateTime (assume UTC)
    let dt = NaiveDateTime::parse_from_str(&input, &fmt)?;
    Ok(dt.and_utc().timestamp())
}

fn format_to_readable_impl(input: i64, fmt: String) -> AnyhowResult<String> {
    let dt = Utc.timestamp_opt(input, 0).single().ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))?;
    Ok(dt.format(&fmt).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_conversion() {
        let lib = StdTimeLibrary;
        let ts = 1609459200; // 2021-01-01 00:00:00 UTC
        let fmt = "%Y-%m-%d %H:%M:%S";

        let readable = lib.format_to_readable(ts, fmt.to_string()).unwrap();
        assert_eq!(readable, "2021-01-01 00:00:00");

        let epoch = lib.format_to_epoch(readable, fmt.to_string()).unwrap();
        assert_eq!(epoch, ts);
    }
}
