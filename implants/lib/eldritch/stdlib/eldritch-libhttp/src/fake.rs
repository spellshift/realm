use super::HttpLibrary;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;
use spin::RwLock;

#[derive(Default, Debug)]
#[eldritch_library_impl(HttpLibrary)]
pub struct HttpLibraryFake;

impl HttpLibrary for HttpLibraryFake {
    fn download(
        &self,
        mut uri: String,
        _dst: String,
        _allow_insecure: Option<bool>,
    ) -> Result<(), String> {
        if !uri.starts_with("http://") && !uri.starts_with("https://") {
            uri = format!("https://{}", uri);
        }
        let _ = uri; // Used for suppressing warning
        Ok(())
    }

    fn get(
        &self,
        mut uri: String,
        _query_params: Option<BTreeMap<String, String>>,
        _headers: Option<BTreeMap<String, String>>,
        _allow_insecure: Option<bool>,
    ) -> Result<BTreeMap<String, Value>, String> {
        if !uri.starts_with("http://") && !uri.starts_with("https://") {
            uri = format!("https://{}", uri);
        }

        let mut map = BTreeMap::new();
        map.insert("status_code".into(), Value::Int(200));
        map.insert(
            "body".into(),
            Value::Bytes(format!("Mock GET response from {}", uri).into_bytes()),
        );

        // Mock headers
        let mut headers_map = BTreeMap::new();
        headers_map.insert(
            Value::String("Content-Type".into()),
            Value::String("text/plain".into()),
        );
        map.insert(
            "headers".into(),
            Value::Dictionary(Arc::new(RwLock::new(headers_map))),
        );

        Ok(map)
    }

    fn post(
        &self,
        mut uri: String,
        body: Option<String>,
        _form: Option<BTreeMap<String, String>>,
        _headers: Option<BTreeMap<String, String>>,
        _allow_insecure: Option<bool>,
    ) -> Result<BTreeMap<String, Value>, String> {
        if !uri.starts_with("http://") && !uri.starts_with("https://") {
            uri = format!("https://{}", uri);
        }

        let mut map = BTreeMap::new();
        map.insert("status_code".into(), Value::Int(201));
        let body_len = body.map(|b| b.len()).unwrap_or(0);
        map.insert(
            "body".into(),
            Value::Bytes(
                format!(
                    "Mock POST response from {}, received {} bytes",
                    uri, body_len
                )
                .into_bytes(),
            ),
        );

        // Mock headers
        let mut headers_map = BTreeMap::new();
        headers_map.insert(
            Value::String("Content-Type".into()),
            Value::String("application/json".into()),
        );
        map.insert(
            "headers".into(),
            Value::Dictionary(Arc::new(RwLock::new(headers_map))),
        );

        Ok(map)
    }
}

#[cfg(all(test, feature = "fake_bindings"))]
mod tests {
    use super::*;

    #[test]
    fn test_http_fake_get() {
        let http = HttpLibraryFake;
        let resp = http
            .get("http://example.com".into(), None, None, None)
            .unwrap();
        assert_eq!(resp.get("status_code").unwrap(), &Value::Int(200));
        if let Value::Bytes(b) = resp.get("body").unwrap() {
            assert_eq!(
                String::from_utf8(b.clone()).unwrap(),
                "Mock GET response from http://example.com"
            );
        } else {
            panic!("Body should be bytes");
        }

        let resp2 = http.get("example.com".into(), None, None, None).unwrap();
        if let Value::Bytes(b) = resp2.get("body").unwrap() {
            assert_eq!(
                String::from_utf8(b.clone()).unwrap(),
                "Mock GET response from https://example.com"
            );
        } else {
            panic!("Body should be bytes");
        }
    }

    #[test]
    fn test_http_fake_post() {
        let http = HttpLibraryFake;
        let resp = http
            .post(
                "http://example.com".into(),
                Some("abc".into()),
                None,
                None,
                None,
            )
            .unwrap();
        assert_eq!(resp.get("status_code").unwrap(), &Value::Int(201));
        if let Value::Bytes(b) = resp.get("body").unwrap() {
            let s = String::from_utf8(b.clone()).unwrap();
            assert!(s.contains("received 3 bytes"));
            assert!(s.contains("http://example.com"));
        } else {
            panic!("Body should be bytes");
        }

        let resp2 = http
            .post("example.com".into(), Some("abc".into()), None, None, None)
            .unwrap();
        if let Value::Bytes(b) = resp2.get("body").unwrap() {
            let s = String::from_utf8(b.clone()).unwrap();
            assert!(s.contains("https://example.com"));
        } else {
            panic!("Body should be bytes");
        }
    }
}
