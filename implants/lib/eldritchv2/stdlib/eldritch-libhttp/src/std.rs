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
    fn download(&self, url: String, path: String) -> Result<(), String> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("Failed to create runtime: {e}"))?;

        runtime.block_on(async {
            let mut dest = File::create(PathBuf::from(path.clone()))
                .map_err(|e| format!("Failed to create file: {e}"))?;

            // v1: download(uri, dst, allow_insecure)
            // v2: download(url, path) -> assumes insecure is false or handled by env?
            // The trait signature doesn't have allow_insecure. We'll default to false.
            let client = reqwest::Client::builder()
                .build()
                .map_err(|e| format!("Failed to build client: {e}"))?;

            use futures::StreamExt;

            let resp = client
                .get(&url)
                .send()
                .await
                .map_err(|e| format!("Failed to send request: {e}"))?;

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
        matchers::{all_of, request},
        responders::status_code,
        Expectation, Server,
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
        // Close file so download can overwrite/create it if needed, or just keep path
        // NamedTempFile deletes on drop. We keep it alive until end of test?
        // Actually, on Windows you can't write to an open file sometimes.
        // Let's just use the path.

        let url = server.url("/foo").to_string();
        let lib = StdHttpLibrary;

        lib.download(url, path.clone()).unwrap();

        let content = read_to_string(&path).unwrap();
        assert_eq!(content, "test body");
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
                dict.get(&Value::String("x-test".to_string())).or(dict.get(&Value::String("X-Test".to_string()))).unwrap(),
                &Value::String("Value".into())
            );
        } else {
            panic!("Headers should be dictionary");
        }
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
}
