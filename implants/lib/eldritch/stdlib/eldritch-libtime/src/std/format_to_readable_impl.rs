use alloc::string::{String, ToString};
use anyhow::Result as AnyhowResult;
use chrono::{TimeZone, Utc};

pub fn format_to_readable(input: i64, format: String) -> Result<String, String> {
    format_to_readable_impl(input, format).map_err(|e| e.to_string())
}

fn format_to_readable_impl(input: i64, fmt: String) -> AnyhowResult<String> {
    let dt = Utc
        .timestamp_opt(input, 0)
        .single()
        .ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))?;
    Ok(dt.format(&fmt).to_string())
}

#[cfg(test)]
mod tests {
    use super::super::StdTimeLibrary;
    use super::super::TimeLibrary;
    use alloc::string::ToString;

    #[test]
    fn test_format_to_readable_invalid() {
        let lib = StdTimeLibrary;
        let res = lib.format_to_readable(i64::MAX, "%Y".to_string());
        assert!(res.is_err());
    }
}
