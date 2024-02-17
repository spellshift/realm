use anyhow::Result;
use regex::Regex;

pub fn match_all(haystack: String, pattern: String) -> Result<Vec<String>> {
    let mut matches = Vec::new();
    let re = Regex::new(pattern.as_str())?;
    let captures = re.captures_iter(haystack.as_str());
    for (_, [m]) in captures.map(|c| c.extract()) {
        matches.push(String::from(m))
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
