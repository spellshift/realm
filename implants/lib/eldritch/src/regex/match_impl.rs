use anyhow::{anyhow, Result};
use regex::Regex;

pub fn r#match(haystack: String, pattern: String) -> Result<String> {
    let re = Regex::new(pattern.as_str())?;
    // `- 1` is due to how Rust tracks the groups (https://docs.rs/regex/latest/regex/struct.CaptureLocations.html#method.len)
    let num_capture_groups = re.capture_locations().len() - 1;
    if num_capture_groups != 1 {
        return Err(anyhow!("only 1 capture group is supported but {} given", num_capture_groups))
    }
    if let Some(captures) = re.captures(haystack.as_str()) {
        // `get(1)` due to how Rust tracks the captures (https://docs.rs/regex/latest/regex/struct.Captures.html#method.get)
        if let Some(m) = captures.get(1) {
            return Ok(String::from(m.as_str()))
        }
    }
    Ok(String::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match() -> anyhow::Result<()> {
        let test_haystack = String::from(
            r#"
            Hold fast to dreams
            For if dreams die
            Life is a broken-winged bird
            That cannot fly.

            Hold fast to dreams
            For when dreams go
            Life is a barren field
            Frozen with snow."#,
        );
        let test_pattern = String::from(r"(?m)^\s*(.+\.)$");
        let m = r#match(test_haystack, test_pattern)?;
        assert_eq!(m, "That cannot fly.");
        Ok(())
    }

    #[test]
    fn test_no_match() -> anyhow::Result<()> {
        let test_haystack = String::from(
            r#"
            Hold fast to dreams
            For if dreams die
            Life is a broken-winged bird
            That cannot fly

            Hold fast to dreams
            For when dreams go
            Life is a barren field
            Frozen with snow"#,
        );
        let test_pattern = String::from(r"(?m)^\s*(.+\.)$");
        let m = r#match(test_haystack, test_pattern)?;
        assert_eq!(m, "");
        Ok(())
    }
}
