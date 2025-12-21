use super::HttpLibrary;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use eldritch_core::Value;
use eldritch_macros::eldritch_library_impl;
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use spin::RwLock;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Default, Debug)]
#[eldritch_library_impl(HttpLibrary)]
pub struct StdHttpLibrary;

impl HttpLibrary for StdHttpLibrary {
    fn download(&self, url: String, path: String, insecure: Option<bool>) -> Result<(), String> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("Failed to create runtime: {e}"))?;

        runtime.block_on(async {
            // v1: download(uri, dst, allow_insecure)
            // v2: download(url, path) -> assumes insecure is false or handled by env?
            // The trait signature doesn't have allow_insecure. We'll default to false.
            let client = reqwest::Client::builder()
                .danger_accept_invalid_certs(insecure.unwrap_or(false))
                .build()
                .map_err(|e| format!("Failed to build client: {e}"))?;

            use futures::StreamExt;

            let resp = client
                .get(&url)
                .send()
                .await
                .map_err(|e| format!("Failed to send request: {e}"))?;

            if !resp.status().is_success() {
                return Err(format!("Download failed with status: {}", resp.status()));
            }

            let mut dest = File::create(PathBuf::from(path.clone()))
                .map_err(|e| format!("Failed to create file: {e}"))?;

            let mut stream = resp.bytes_stream();

            while let Some(chunk_result) = stream.next().await {
                let chunk = chunk_result.map_err(|e| format!("Error reading chunk: {e}"))?;
                dest.write_all(&chunk)
                    .map_err(|e| format!("Error writing to file: {e}"))?;
            }

            dest.flush()
                .map_err(|e| format!("Error flushing file: {e}"))?;
            Ok(())
        })
    }

    fn get(
        &self,
        url: String,
        headers: Option<BTreeMap<String, String>>,
    ) -> Result<BTreeMap<String, Value>, String> {
        let client = Client::builder()
            .build()
            .map_err(|e| format!("Failed to build client: {e}"))?;

        let mut req = client.get(&url);

        if let Some(h) = headers {
            let mut headers_map = HeaderMap::new();
            for (k, v) in h {
                let name = HeaderName::from_bytes(k.as_bytes())
                    .map_err(|e| format!("Invalid header name: {e}"))?;
                let value = HeaderValue::from_bytes(v.as_bytes())
                    .map_err(|e| format!("Invalid header value: {e}"))?;
                headers_map.append(name, value);
            }
            req = req.headers(headers_map);
        }

        let resp = req.send().map_err(|e| format!("Request failed: {e}"))?;

        let mut map = BTreeMap::new();
        map.insert(
            "status_code".into(),
            Value::Int(resp.status().as_u16() as i64),
        );

        let mut headers_map = BTreeMap::new();
        for (k, v) in resp.headers() {
            headers_map.insert(
                Value::String(k.to_string()),
                Value::String(v.to_str().unwrap_or("").to_string()),
            );
        }
        map.insert(
            "headers".into(),
            Value::Dictionary(Arc::new(RwLock::new(headers_map))),
        );

        // We use bytes for body to be safe, consistent with fake implementation returning bytes
        let bytes = resp
            .bytes()
            .map_err(|e| format!("Failed to read body: {e}"))?;
        map.insert("body".into(), Value::Bytes(bytes.to_vec()));

        Ok(map)
    }

    fn post(
        &self,
        url: String,
        body: Option<Vec<u8>>,
        headers: Option<BTreeMap<String, String>>,
    ) -> Result<BTreeMap<String, Value>, String> {
        let client = reqwest::blocking::Client::builder()
            .build()
            .map_err(|e| format!("Failed to build client: {e}"))?;

        let mut req = client.post(&url);

        if let Some(h) = headers {
            let mut headers_map = HeaderMap::new();
            for (k, v) in h {
                let name = HeaderName::from_bytes(k.as_bytes())
                    .map_err(|e| format!("Invalid header name: {e}"))?;
                let value = HeaderValue::from_bytes(v.as_bytes())
                    .map_err(|e| format!("Invalid header value: {e}"))?;
                headers_map.append(name, value);
            }
            req = req.headers(headers_map);
        }

        if let Some(b) = body {
            req = req.body(b);
        }

        let resp = req.send().map_err(|e| format!("Request failed: {e}"))?;

        let mut map = BTreeMap::new();
        map.insert(
            "status_code".into(),
            Value::Int(resp.status().as_u16() as i64),
        );

        let mut headers_map = BTreeMap::new();
        for (k, v) in resp.headers() {
            headers_map.insert(
                Value::String(k.to_string()),
                Value::String(v.to_str().unwrap_or("").to_string()),
            );
        }
        map.insert(
            "headers".into(),
            Value::Dictionary(Arc::new(RwLock::new(headers_map))),
        );

        let bytes = resp
            .bytes()
            .map_err(|e| format!("Failed to read body: {e}"))?;
        map.insert("body".into(), Value::Bytes(bytes.to_vec()));

        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httptest::{
        Expectation, Server,
        matchers::{all_of, contains, request},
        responders::status_code,
    };
    use std::fs::read_to_string;
    use tempfile::NamedTempFile;

    #[test]
    fn test_download() {
        let server = Server::run();
        server.expect(
            Expectation::matching(request::method_path("GET", "/foo"))
                .respond_with(status_code(200).body("test body")),
        );

        let tmp_file = NamedTempFile::new().unwrap();
        let path = String::from(tmp_file.path().to_str().unwrap());
        // Close file so download can overwrite/create it if needed

        let url = server.url("/foo").to_string();
        let lib = StdHttpLibrary;

        lib.download(url, path.clone(), None).unwrap();

        let content = read_to_string(&path).unwrap();
        assert_eq!(content, "test body");
    }

    #[test]
    fn test_download_404() {
        let server = Server::run();
        server.expect(
            Expectation::matching(request::method_path("GET", "/foo"))
                .respond_with(status_code(404)),
        );

        let tmp_file = NamedTempFile::new().unwrap();
        let path = String::from(tmp_file.path().to_str().unwrap());

        let url = server.url("/foo").to_string();
        let lib = StdHttpLibrary;

        let res = lib.download(url, path, None);
        assert!(res.is_err());
        assert!(
            res.unwrap_err()
                .contains("Download failed with status: 404")
        );
    }

    #[test]
    fn test_download_write_error() {
        let server = Server::run();
        server.expect(
            Expectation::matching(request::method_path("GET", "/foo"))
                .respond_with(status_code(200).body("test body")),
        );

        let url = server.url("/foo").to_string();
        let lib = StdHttpLibrary;

        // Try to download to a directory path, which should fail to open as a file
        let tmp_dir = tempfile::tempdir().unwrap();
        let path = tmp_dir.path().to_str().unwrap().to_string();

        let res = lib.download(url, path, None);
        assert!(res.is_err());
        // Exact error message depends on OS, but should be a file creation error
        let err = res.unwrap_err();
        assert!(
            err.contains("Failed to create file")
                || err.contains("Is a directory")
                || err.contains("Access is denied")
        );
    }

    #[test]
    fn test_get() {
        let server = Server::run();
        server.expect(
            Expectation::matching(request::method_path("GET", "/foo")).respond_with(
                status_code(200)
                    .body("test body")
                    .append_header("X-Test", "Value"),
            ),
        );

        let url = server.url("/foo").to_string();
        let lib = StdHttpLibrary;

        let res = lib.get(url, None).unwrap();

        assert_eq!(res.get("status_code").unwrap(), &Value::Int(200));

        if let Value::Bytes(b) = res.get("body").unwrap() {
            assert_eq!(b, b"test body");
        } else {
            panic!("Body should be bytes");
        }

        if let Value::Dictionary(d) = res.get("headers").unwrap() {
            let dict = d.read();
            assert_eq!(
                dict.get(&Value::String("x-test".to_string()))
                    .or(dict.get(&Value::String("X-Test".to_string())))
                    .unwrap(),
                &Value::String("Value".into())
            );
        } else {
            panic!("Headers should be dictionary");
        }
    }

    #[test]
    fn test_get_404() {
        let server = Server::run();
        server.expect(
            Expectation::matching(request::method_path("GET", "/foo"))
                .respond_with(status_code(404)),
        );

        let url = server.url("/foo").to_string();
        let lib = StdHttpLibrary;

        let res = lib.get(url, None).unwrap();
        assert_eq!(res.get("status_code").unwrap(), &Value::Int(404));
    }

    #[test]
    fn test_get_server_error() {
        let server = Server::run();
        server.expect(
            Expectation::matching(request::method_path("GET", "/foo"))
                .respond_with(status_code(500)),
        );

        let url = server.url("/foo").to_string();
        let lib = StdHttpLibrary;

        let res = lib.get(url, None).unwrap();
        assert_eq!(res.get("status_code").unwrap(), &Value::Int(500));
    }

    #[test]
    fn test_get_connection_error() {
        // Pick a port that is unlikely to be open
        let url = "http://127.0.0.1:54321/foo".to_string();
        let lib = StdHttpLibrary;

        let res = lib.get(url, None);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Request failed"));
    }

    #[test]
    fn test_get_with_headers() {
        let server = Server::run();
        // Lowercase the header key in expectation as reqwest/httptest might normalize it?
        // Actually, HTTP/2 normalizes to lowercase. But we are likely using HTTP/1.1 in httptest.
        // reqwest does preserve case for headers usually, but maybe it canonicalizes.
        // Let's try matching lowercase.
        server.expect(
            Expectation::matching(all_of![
                request::method_path("GET", "/foo"),
                request::headers(contains(("x-my-header", "MyValue")))
            ])
            .respond_with(status_code(200)),
        );

        let url = server.url("/foo").to_string();
        let lib = StdHttpLibrary;

        let mut headers = BTreeMap::new();
        headers.insert("X-My-Header".into(), "MyValue".into());

        let res = lib.get(url, Some(headers)).unwrap();
        assert_eq!(res.get("status_code").unwrap(), &Value::Int(200));
    }

    #[test]
    fn test_post() {
        let server = Server::run();
        server.expect(
            Expectation::matching(all_of![
                request::method_path("POST", "/foo"),
                request::body("request body")
            ])
            .respond_with(status_code(201).body("response body")),
        );

        let url = server.url("/foo").to_string();
        let lib = StdHttpLibrary;

        let res = lib.post(url, Some(b"request body".to_vec()), None).unwrap();

        assert_eq!(res.get("status_code").unwrap(), &Value::Int(201));
        if let Value::Bytes(b) = res.get("body").unwrap() {
            assert_eq!(b, b"response body");
        } else {
            panic!("Body should be bytes");
        }
    }

    #[test]
    fn test_post_with_headers() {
        let server = Server::run();
        server.expect(
            Expectation::matching(all_of![
                request::method_path("POST", "/foo"),
                request::headers(contains(("content-type", "application/json")))
            ])
            .respond_with(status_code(200)),
        );

        let url = server.url("/foo").to_string();
        let lib = StdHttpLibrary;

        let mut headers = BTreeMap::new();
        headers.insert("Content-Type".into(), "application/json".into());

        let res = lib.post(url, None, Some(headers)).unwrap();
        assert_eq!(res.get("status_code").unwrap(), &Value::Int(200));
    }

    #[test]
    fn test_post_error() {
        let url = "http://127.0.0.1:54321/foo".to_string();
        let lib = StdHttpLibrary;

        let res = lib.post(url, None, None);
        assert!(res.is_err());
    }
}
