use alloc::string::{String, ToString};
use anyhow::Result as AnyhowResult;

pub fn format_to_readable(input: i64, format: String) -> Result<String, String> {
    format_to_readable_impl(input, format).map_err(|e| e.to_string())
}

fn format_to_readable_impl(input: i64, fmt: String) -> AnyhowResult<String> {
    let ts = jiff::Timestamp::from_second(input)?;
    Ok(ts.strftime(&fmt).to_string())
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
