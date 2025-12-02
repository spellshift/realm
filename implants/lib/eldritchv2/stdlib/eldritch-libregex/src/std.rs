use super::RegexLibrary;
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::eldritch_library_impl;
use regex::{NoExpand, Regex};

#[derive(Default, Debug)]
#[eldritch_library_impl(RegexLibrary)]
pub struct StdRegexLibrary;

impl RegexLibrary for StdRegexLibrary {
    fn match_all(&self, haystack: String, pattern: String) -> Result<Vec<String>, String> {
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

    fn r#match(&self, haystack: String, pattern: String) -> Result<String, String> {
        let re = Regex::new(&pattern).map_err(|e| e.to_string())?;
        let num_capture_groups = re.capture_locations().len().saturating_sub(1);
        if num_capture_groups != 1 {
            return Err(format!(
                "only 1 capture group is supported but {num_capture_groups} given",
            ));
        }
        if let Some(captures) = re.captures(&haystack) {
            if let Some(m) = captures.get(1) {
                return Ok(String::from(m.as_str()));
            }
        }
        Ok(String::new())
    }

    fn replace_all(
        &self,
        haystack: String,
        pattern: String,
        value: String,
    ) -> Result<String, String> {
        let re = Regex::new(&pattern).map_err(|e| e.to_string())?;
        let result = re.replace_all(&haystack, NoExpand(&value));
        Ok(String::from(result))
    }

    fn replace(&self, haystack: String, pattern: String, value: String) -> Result<String, String> {
        let re = Regex::new(&pattern).map_err(|e| e.to_string())?;
        let result = re.replace(&haystack, NoExpand(&value));
        Ok(String::from(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let test_pattern = String::from(r"(?m)^\s*(.+\.)$");
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
        let test_pattern = String::from(r"(?m)^\s*(.+\.)$");
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
        let test_pattern = String::from(r"(?m)^\s*(.+\.)$");
        let matches = lib.match_all(test_haystack, test_pattern).unwrap();
        assert!(matches.is_empty());
    }

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
        let test_pattern = String::from(r"(?m)^\s*(.+\.)$");
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
        let test_pattern = String::from(r"(?m)^\s*(.+\.)$");
        let m = lib.r#match(test_haystack, test_pattern).unwrap();
        assert_eq!(m, "");
    }

    #[test]
    fn test_replace_all() {
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
        let test_pattern = String::from(r"(?m)^\s*(.+\.)$");
        let test_value = String::from("That cannot soar.");
        let m = lib
            .replace_all(test_haystack, test_pattern, test_value)
            .unwrap();
        assert!(!m.contains("That cannot fly."));
        assert!(!m.contains("Frozen with snow."));
        assert!(m.contains("That cannot soar."));
    }

    #[test]
    fn test_replace_all_not_found() {
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
        let test_pattern = String::from(r"(?m)^\s*(That we may believe)$");
        let test_value = String::from("That cannot soar.");
        let m = lib
            .replace_all(test_haystack.clone(), test_pattern, test_value)
            .unwrap();
        assert_eq!(test_haystack, m);
    }

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
        let test_pattern = String::from(r"(?m)^\s*(.+\.)$");
        let test_value = String::from("That cannot soar.");
        let m = lib
            .replace(test_haystack, test_pattern, test_value)
            .unwrap();
        assert!(!m.contains("That cannot fly."));
        assert!(m.contains("Frozen with snow."));
        assert!(m.contains("That cannot soar."));
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
}
