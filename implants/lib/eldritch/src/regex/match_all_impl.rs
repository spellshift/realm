use anyhow::{anyhow, Result};
use regex::Regex;

pub fn match_all(haystack: String, pattern: String) -> Result<Vec<String>> {
    let mut matches = Vec::new();
    let re = Regex::new(pattern.as_str())?;
    // `- 1` is due to how Rust tracks the groups (https://docs.rs/regex/latest/regex/struct.CaptureLocations.html#method.len)
    let num_capture_groups = re.capture_locations().len() - 1;
    if num_capture_groups != 1 {
        return Err(anyhow!("only 1 capture group is supported but {} given", num_capture_groups))
    }
    for captures in re.captures_iter(haystack.as_str()) {
        // `get(1)` due to how Rust tracks the captures (https://docs.rs/regex/latest/regex/struct.Captures.html#method.get)
        if let Some(m) = captures.get(1) {
            matches.push(String::from(m.as_str()))
        }
    }
    Ok(matches)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_match() -> anyhow::Result<()> {
        let test_haystack = String::from(
            r#"
            Hold fast to dreams
            For if dreams die
            Life is a broken-winged bird
            That cannot fly

            Hold fast to dreams
            For when dreams go
            Life is a barren field
            Frozen with snow."#,
        );
        let test_pattern = String::from(r"(?m)^\s*(.+\.)$");
        let matches = match_all(test_haystack, test_pattern)?;
        assert_eq!(matches.len(), 1);
        assert_eq!(matches.first().unwrap(), "Frozen with snow.");
        Ok(())
    }

    #[test]
    fn test_multi_match() -> anyhow::Result<()> {
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
        let matches = match_all(test_haystack, test_pattern)?;
        assert_eq!(matches.len(), 2);
        assert_eq!(matches.first().unwrap(), "That cannot fly.");
        assert_eq!(matches.get(1).unwrap(), "Frozen with snow.");
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
        let matches = match_all(test_haystack, test_pattern)?;
        assert!(matches.is_empty());
        Ok(())
    }
}
