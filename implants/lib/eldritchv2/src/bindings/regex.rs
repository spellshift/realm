use eldritch_macros::{eldritch_library, eldritch_library_impl, eldritch_method};
use alloc::string::String;
use alloc::vec::Vec;

#[eldritch_library("regex")]
pub trait RegexLibrary {
    #[eldritch_method]
    fn match_all(&self, haystack: String, pattern: String) -> Result<Vec<String>, String>;

    #[eldritch_method]
    fn r#match(&self, haystack: String, pattern: String) -> Result<String, String>;

    #[eldritch_method]
    fn replace_all(&self, haystack: String, pattern: String, value: String) -> Result<String, String>;

    #[eldritch_method]
    fn replace(&self, haystack: String, pattern: String, value: String) -> Result<String, String>;
}

#[cfg(feature = "fake_bindings")]
#[derive(Default, Debug)]
#[eldritch_library_impl(RegexLibrary)]
pub struct RegexLibraryFake;

#[cfg(feature = "fake_bindings")]
impl RegexLibrary for RegexLibraryFake {
    fn match_all(&self, _haystack: String, _pattern: String) -> Result<Vec<String>, String> {
        Ok(Vec::new())
    }

    fn r#match(&self, _haystack: String, _pattern: String) -> Result<String, String> {
        Ok(String::new())
    }

    fn replace_all(&self, haystack: String, _pattern: String, _value: String) -> Result<String, String> {
        Ok(haystack) // No-op replacement
    }

    fn replace(&self, haystack: String, _pattern: String, _value: String) -> Result<String, String> {
        Ok(haystack)
    }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::*;

    #[test]
    fn test_regex_fake() {
        let regex = RegexLibraryFake::default();
        assert!(regex.match_all("foo".into(), "bar".into()).unwrap().is_empty());
        assert_eq!(regex.replace("foo".into(), "o".into(), "a".into()).unwrap(), "foo");
    }
}
