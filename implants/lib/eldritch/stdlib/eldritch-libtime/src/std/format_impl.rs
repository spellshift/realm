use alloc::string::{String, ToString};
use anyhow::Result as AnyhowResult;

pub fn format(input: i64, format: String) -> Result<String, String> {
    format_impl(input, format).map_err(|e| e.to_string())
}

fn format_impl(input: i64, fmt: String) -> AnyhowResult<String> {
    let ts = jiff::Timestamp::from_second(input)?;
    Ok(ts.strftime(&fmt).to_string())
}

#[cfg(test)]
mod tests {
    use super::super::StdTimeLibrary;
    use super::super::TimeLibrary;
    use alloc::string::ToString;

    #[test]
    fn test_format_invalid() {
        let lib = StdTimeLibrary;
        let res = lib.format(i64::MAX, "%Y".to_string());
        assert!(res.is_err());
    }
}
