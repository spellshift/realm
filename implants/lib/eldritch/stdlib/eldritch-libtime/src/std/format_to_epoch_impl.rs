use alloc::string::{String, ToString};
use anyhow::Result as AnyhowResult;

pub fn format_to_epoch(input: String, fmt: String) -> Result<i64, String> {
    format_to_epoch_impl(input, fmt).map_err(|e| e.to_string())
}

fn format_to_epoch_impl(input: String, fmt: String) -> AnyhowResult<i64> {
    // Try to parse as Timestamp with timezone first
    if let Ok(ts) = jiff::Timestamp::strptime(&fmt, &input) {
        return Ok(ts.as_second());
    }
    // Handle %z vs %:z mismatch (original tolerated both, jiff is strict)
    if fmt.contains("%z") {
        let alt_fmt = fmt.replace("%z", "%:z");
        if alt_fmt != fmt {
            if let Ok(ts) = jiff::Timestamp::strptime(&alt_fmt, &input) {
                return Ok(ts.as_second());
            }
        }
    }
    if fmt.contains("%:z") {
        let alt_fmt = fmt.replace("%:z", "%z");
        if alt_fmt != fmt {
            if let Ok(ts) = jiff::Timestamp::strptime(&alt_fmt, &input) {
                return Ok(ts.as_second());
            }
        }
    }

    // Fallback to civil DateTime (assume UTC)
    // Original NaiveDateTime requires a time component; mimic that by rejecting
    // date-only formats for this API (test_date_only_fails expects error).
    // This keeps timestomp's date-only handling separate.
    let has_time = fmt.contains("%H")
        || fmt.contains("%I")
        || fmt.contains("%M")
        || fmt.contains("%S")
        || fmt.contains("%T")
        || fmt.contains("%R")
        || fmt.contains("%X");
    if !has_time {
        anyhow::bail!("format does not contain time component");
    }
    let dt = jiff::civil::DateTime::strptime(&fmt, &input)?;
    let ts = jiff::tz::TimeZone::UTC.to_timestamp(dt)?;
    Ok(ts.as_second())
}

#[cfg(test)]
mod tests {
    use super::super::StdTimeLibrary;
    use super::super::TimeLibrary;
    use alloc::string::ToString;

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

    #[test]
    fn test_format_to_epoch_formats() {
        let lib = StdTimeLibrary;
        // Test with different format
        let ts = 1609459200; // 2021-01-01 00:00:00 UTC
        let date_str = "2021/01/01 00:00:00";
        let fmt = "%Y/%m/%d %H:%M:%S";

        let epoch = lib
            .format_to_epoch(date_str.to_string(), fmt.to_string())
            .unwrap();
        assert_eq!(epoch, ts);
    }

    #[test]
    fn test_date_only_fails() {
        let lib = StdTimeLibrary;
        let date_str = "2021/01/01";
        let fmt = "%Y/%m/%d";
        let res = lib.format_to_epoch(date_str.to_string(), fmt.to_string());
        assert!(res.is_err());
    }

    #[test]
    fn test_format_to_epoch_invalid() {
        let lib = StdTimeLibrary;
        let res = lib.format_to_epoch("invalid".to_string(), "%Y".to_string());
        assert!(res.is_err());
    }

    #[test]
    fn test_format_with_timezone() {
        let lib = StdTimeLibrary;
        // RFC3339 format with timezone
        let input = "2021-01-01T00:00:00+00:00";
        let fmt = "%Y-%m-%dT%H:%M:%S%z";
        let ts = 1609459200;

        let epoch = lib
            .format_to_epoch(input.to_string(), fmt.to_string())
            .unwrap();
        assert_eq!(epoch, ts);
    }
}
