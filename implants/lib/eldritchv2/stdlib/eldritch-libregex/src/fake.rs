
use alloc::string::String;
use alloc::vec::Vec;
use eldritch_macros::eldritch_library_impl;
use super::RegexLibrary;

#[derive(Default, Debug)]
#[eldritch_library_impl(RegexLibrary)]
pub struct RegexLibraryFake;

impl RegexLibrary for RegexLibraryFake {
    fn match_all(&self, _haystack: String, _pattern: String) -> Result<Vec<String>, String> {
        Ok(Vec::new())
    }

    fn r#match(&self, _haystack: String, _pattern: String) -> Result<String, String> {
        Ok(String::new())
    }

    fn replace_all(
        &self,
        haystack: String,
        _pattern: String,
        _value: String,
    ) -> Result<String, String> {
        Ok(haystack) // No-op replacement
    }

    fn replace(
        &self,
        haystack: String,
        _pattern: String,
        _value: String,
    ) -> Result<String, String> {
        Ok(haystack)
    }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {


    #[test]
    fn test_regex_fake() {
        let regex = RegexLibraryFake::default();
        assert!(regex
            .match_all("foo".into(), "bar".into())
            .unwrap()
            .is_empty());
    }
}
