use super::*;
use eldritch_macros::eldritch_library_impl;
use crate::lang::ast::Value;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

#[derive(Default, Debug)]
#[eldritch_library_impl(HttpLibrary)]
pub struct HttpLibraryFake;

impl HttpLibrary for HttpLibraryFake {
    fn download(&self, _url: String, _path: String) -> Result<(), String> { Ok(()) }

    fn request(&self, method: String, url: String, _headers: Option<BTreeMap<String, String>>, _body: Option<Vec<u8>>) -> Result<BTreeMap<String, Value>, String> {
        let mut map = BTreeMap::new();
        map.insert("status_code".into(), Value::Int(200));
        map.insert("body".into(), Value::Bytes(format!("Mock response for {} {}", method, url).into_bytes()));
        map.insert("headers".into(), Value::None); // Simplified
        Ok(map)
    }

    fn upload(&self, _url: String, _path: String) -> Result<(), String> { Ok(()) }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::*;

    #[test]
    fn test_http_fake() {
        let http = HttpLibraryFake::default();
        let resp = http.request("GET".into(), "http://example.com".into(), None, None).unwrap();
        assert_eq!(resp.get("status_code").unwrap(), &Value::Int(200));
    }
}
