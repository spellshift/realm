use eldritch_macros::{eldritch_library, eldritch_library_impl, eldritch_method};
use alloc::string::String;
use alloc::collections::BTreeMap;

#[eldritch_library("http")]
pub trait HttpLibrary {
    #[eldritch_method]
    fn download(&self, uri: String, dst: String, allow_insecure: Option<bool>) -> Result<(), String>;

    #[eldritch_method]
    fn get(&self, uri: String, query_params: Option<BTreeMap<String, String>>, headers: Option<BTreeMap<String, String>>, allow_insecure: Option<bool>) -> Result<String, String>;

    #[eldritch_method]
    fn post(&self, uri: String, body: Option<String>, form: Option<BTreeMap<String, String>>, headers: Option<BTreeMap<String, String>>, allow_insecure: Option<bool>) -> Result<String, String>;
}

#[cfg(feature = "fake_bindings")]
#[derive(Default, Debug)]
#[eldritch_library_impl(HttpLibrary)]
pub struct HttpLibraryFake;

#[cfg(feature = "fake_bindings")]
impl HttpLibrary for HttpLibraryFake {
    fn download(&self, _uri: String, _dst: String, _allow_insecure: Option<bool>) -> Result<(), String> { Ok(()) }

    fn get(&self, _uri: String, _query_params: Option<BTreeMap<String, String>>, _headers: Option<BTreeMap<String, String>>, _allow_insecure: Option<bool>) -> Result<String, String> {
        Ok(String::from("GET Response"))
    }

    fn post(&self, _uri: String, _body: Option<String>, _form: Option<BTreeMap<String, String>>, _headers: Option<BTreeMap<String, String>>, _allow_insecure: Option<bool>) -> Result<String, String> {
        Ok(String::from("POST Response"))
    }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::*;

    #[test]
    fn test_http_fake() {
        let http = HttpLibraryFake::default();
        http.download("http://example.com".into(), "file".into(), None).unwrap();
        assert_eq!(http.get("http://example.com".into(), None, None, None).unwrap(), "GET Response");
        assert_eq!(http.post("http://example.com".into(), None, None, None, None).unwrap(), "POST Response");
    }
}
