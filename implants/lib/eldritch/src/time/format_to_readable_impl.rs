use anyhow::{anyhow, Result};
use chrono::DateTime;

pub fn format_to_readable(t: i64, fmt: &str) -> Result<String> {
    let naive = DateTime::from_timestamp(t, 0)
        .ok_or(anyhow!("Failed to get timestamp from epoch value."))?;
    Ok(naive.format(fmt).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid() {
        assert_eq!(
            format_to_readable(1703563343, "%Y-%m-%d %H:%M:%S").unwrap(),
            String::from("2023-12-26 04:02:23")
        );
    }
}
