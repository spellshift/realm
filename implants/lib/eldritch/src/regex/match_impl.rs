use anyhow::Result;
use regex::Regex;

pub fn r#match(haystack: String, pattern: String) -> Result<String> {
    let re = Regex::new(pattern.as_str())?;
    let captures = re.captures_iter(haystack.as_str());
    if let Some((_, [m])) = captures.map(|c| c.extract()).next() {
        return Ok(String::from(m));
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
