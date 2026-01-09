use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use regex::Regex;

pub fn match_all(haystack: String, pattern: String) -> Result<Vec<String>, String> {
    let re = Regex::new(&pattern).map_err(|e| e.to_string())?;
    // - 1 because capture_locations includes the implicit whole-match group
    let num_capture_groups = re.capture_locations().len().saturating_sub(1);
    if num_capture_groups != 1 {
        return Err(format!(
            "only 1 capture group is supported but {num_capture_groups} given"
        ));
    }
    let mut matches = Vec::new();
    for captures in re.captures_iter(&haystack) {
        if let Some(m) = captures.get(1) {
            matches.push(String::from(m.as_str()));
        }
    }
    Ok(matches)
}

#[cfg(test)]
mod tests {
    use super::super::RegexLibrary;
    use super::super::StdRegexLibrary;
    use alloc::string::String;

    #[test]
    fn test_match_all_one_match() {
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
            Frozen with snow."#,
        );
        let test_pattern = String::from(r"(?m)^[ \t]*(.+\.)$");
        let matches = lib.match_all(test_haystack, test_pattern).unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches.first().unwrap(), "Frozen with snow.");
    }

    #[test]
    fn test_match_all_multi_match() {
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
        let matches = lib.match_all(test_haystack, test_pattern).unwrap();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches.first().unwrap(), "That cannot fly.");
        assert_eq!(matches.get(1).unwrap(), "Frozen with snow.");
    }

    #[test]
    fn test_match_all_no_match() {
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
        let matches = lib.match_all(test_haystack, test_pattern).unwrap();
        assert!(matches.is_empty());
    }

    #[test]
    fn test_invalid_capture_groups() {
        let lib = StdRegexLibrary;
        let test_pattern = String::from(r"(foo)(bar)");
        let res = lib.match_all("foobar".into(), test_pattern.clone());
        assert!(res.is_err());
        assert_eq!(
            res.err().unwrap(),
            "only 1 capture group is supported but 2 given"
        );

        let res = lib.r#match("foobar".into(), test_pattern);
        assert!(res.is_err());
        assert_eq!(
            res.err().unwrap(),
            "only 1 capture group is supported but 2 given"
        );
    }

    #[test]
    fn test_invalid_regex_match_all() {
        let lib = StdRegexLibrary;
        let res = lib.match_all("foo".into(), "[".into());
        assert!(res.is_err());
    }
}
