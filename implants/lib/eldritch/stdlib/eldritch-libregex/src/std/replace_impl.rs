use alloc::string::{String, ToString};
use regex::{NoExpand, Regex};

pub fn replace(haystack: String, pattern: String, value: String) -> Result<String, String> {
    let re = Regex::new(&pattern).map_err(|e| e.to_string())?;
    let result = re.replace(&haystack, NoExpand(&value));
    Ok(String::from(result))
}

#[cfg(test)]
mod tests {
    use super::super::RegexLibrary;
    use super::super::StdRegexLibrary;
    use alloc::string::String;

    #[test]
    fn test_replace() {
        let lib = StdRegexLibrary;
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
        let test_pattern = String::from(r"(?m)^[ \t]*(.+\.)$");
        let test_value = String::from("That cannot soar.");
        let m = lib
            .replace(test_haystack, test_pattern, test_value)
            .unwrap();
        assert!(!m.contains("That cannot fly."));
        assert!(m.contains("Frozen with snow."));
        assert!(m.contains("That cannot soar."));
    }

    #[test]
    fn test_invalid_regex_replace() {
        let lib = StdRegexLibrary;
        let res = lib.replace("foo".into(), "[".into(), "bar".into());
        assert!(res.is_err());
    }

    #[test]
    fn test_replace_multiple_occurrences() {
        let lib = StdRegexLibrary;
        let haystack = String::from("foo bar foo bar");
        let pattern = String::from("foo");
        let value = String::from("baz");
        let res = lib.replace(haystack, pattern, value).unwrap();
        assert_eq!(res, "baz bar foo bar");
    }
}
