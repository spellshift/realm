use anyhow::Result;
use chrono::NaiveDateTime;

pub fn format_to_epoch(s: &str, fmt: &str) -> Result<u64> {
    let naive = NaiveDateTime::parse_from_str(s, fmt)?;
    Ok(naive.and_utc().timestamp() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid() {
        let input = "2023-12-26 03:52:00";
        let format = "%Y-%m-%d %H:%M:%S";
        assert_eq!(format_to_epoch(input, format).unwrap(), 1703562720);
    }

    #[test]
    fn test_invalid() {
        let input = "2023-12-26";
        let format = "%Y-%m-%d";
        assert!(format_to_epoch(input, format).is_err());
    }
}
