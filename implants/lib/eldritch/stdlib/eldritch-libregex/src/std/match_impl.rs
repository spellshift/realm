use alloc::format;
use alloc::string::{String, ToString};
use regex::Regex;

pub fn r#match(haystack: String, pattern: String) -> Result<String, String> {
    let re = Regex::new(&pattern).map_err(|e| e.to_string())?;
    let num_capture_groups = re.capture_locations().len().saturating_sub(1);
    if num_capture_groups != 1 {
        return Err(format!(
            "only 1 capture group is supported but {num_capture_groups} given",
        ));
    }
    if let Some(captures) = re.captures(&haystack)
        && let Some(m) = captures.get(1)
    {
        return Ok(String::from(m.as_str()));
    }
    Ok(String::new())
}

#[cfg(test)]
mod tests {
    use super::super::RegexLibrary;
    use super::super::StdRegexLibrary;
    use alloc::string::String;

    #[test]
    fn test_match_found() {
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
        let m = lib.r#match(test_haystack, test_pattern).unwrap();
        assert_eq!(m, "That cannot fly.");
    }

    #[test]
    fn test_match_not_found() {
        let lib = StdRegexLibrary;
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
        let test_pattern = String::from(r"(?m)^[ \t]*(.+\.)$");
        let m = lib.r#match(test_haystack, test_pattern).unwrap();
        assert_eq!(m, "");
    }

    #[test]
    fn test_invalid_regex_match() {
        let lib = StdRegexLibrary;
        let res = lib.r#match("foo".into(), "[".into());
        assert!(res.is_err());
    }
}
