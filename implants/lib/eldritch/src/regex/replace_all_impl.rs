use anyhow::Result;
use regex::{NoExpand, Regex};

pub fn replace_all(haystack: String, pattern: String, value: String) -> Result<String> {
    let re = Regex::new(&pattern)?;
    let result = re.replace_all(&haystack, NoExpand(&value));
    Ok(String::from(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_all() -> anyhow::Result<()> {
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
        let test_value = String::from("That cannot soar.");
        let m = replace_all(test_haystack, test_pattern, test_value)?;
        assert!(!m.contains("That cannot fly."));
        assert!(!m.contains("Frozen with snow."));
        Ok(())
    }

    #[test]
    fn test_replace_not_found() -> anyhow::Result<()> {
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
        let test_pattern = String::from(r"(?m)^\s*(That we may believe)$");
        let test_value = String::from("That cannot soar.");
        let m = replace_all(test_haystack.clone(), test_pattern, test_value)?;
        assert_eq!(test_haystack, m);
        Ok(())
    }
}
