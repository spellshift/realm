use super::HttpLibrary;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
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
    fn download(
        &self,
        uri: String,
        dst: String,
        allow_insecure: Option<bool>,
    ) -> Result<(), String> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("Failed to create runtime: {e}"))?;

        runtime.block_on(async {
            let client = reqwest::Client::builder()
                .danger_accept_invalid_certs(allow_insecure.unwrap_or(false))
                .build()
                .map_err(|e| format!("Failed to build client: {e}"))?;

            use futures::StreamExt;

            let resp = client
                .get(&uri)
                .send()
                .await
                .map_err(|e| format!("Failed to send request: {e}"))?;

            if !resp.status().is_success() {
                return Err(format!("Download failed with status: {}", resp.status()));
            }

            let mut dest = File::create(PathBuf::from(dst.clone()))
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
        uri: String,
        query_params: Option<BTreeMap<String, String>>,
        headers: Option<BTreeMap<String, String>>,
        allow_insecure: Option<bool>,
    ) -> Result<BTreeMap<String, Value>, String> {
        let client = Client::builder()
            .danger_accept_invalid_certs(allow_insecure.unwrap_or(false))
            .build()
            .map_err(|e| format!("Failed to build client: {e}"))?;

        let mut req = client.get(&uri);

        if let Some(params) = query_params {
            req = req.query(&params);
        }

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

        let bytes = resp
            .bytes()
            .map_err(|e| format!("Failed to read body: {e}"))?;
        map.insert("body".into(), Value::Bytes(bytes.to_vec()));

        Ok(map)
    }

    fn post(
        &self,
        uri: String,
        body: Option<String>,
        form: Option<BTreeMap<String, String>>,
        headers: Option<BTreeMap<String, String>>,
        allow_insecure: Option<bool>,
    ) -> Result<BTreeMap<String, Value>, String> {
        let client = reqwest::blocking::Client::builder()
            .danger_accept_invalid_certs(allow_insecure.unwrap_or(false))
            .build()
            .map_err(|e| format!("Failed to build client: {e}"))?;

        let mut req = client.post(&uri);

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
        } else if let Some(f) = form {
            req = req.form(&f);
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
        matchers::{all_of, contains, request, url_decoded},
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

        let url = server.url("/foo").to_string();
        let lib = StdHttpLibrary;

        lib.download(url, path.clone(), None).unwrap();

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

        let res = lib.get(url, None, None, None).unwrap();

        assert_eq!(res.get("status_code").unwrap(), &Value::Int(200));

        if let Value::Bytes(b) = res.get("body").unwrap() {
            assert_eq!(b, b"test body");
        } else {
            panic!("Body should be bytes");
        }
    }

    #[test]
    fn test_get_with_params() {
        let server = Server::run();
        server.expect(
            Expectation::matching(all_of![
                request::method_path("GET", "/foo"),
                request::query(url_decoded(contains(("q", "search"))))
            ])
            .respond_with(status_code(200)),
        );

        let url = server.url("/foo").to_string();
        let lib = StdHttpLibrary;

        let mut params = BTreeMap::new();
        params.insert("q".into(), "search".into());

        let res = lib.get(url, Some(params), None, None).unwrap();
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

        let res = lib
            .post(url, Some("request body".into()), None, None, None)
            .unwrap();

        assert_eq!(res.get("status_code").unwrap(), &Value::Int(201));
    }

    #[test]
    fn test_post_with_form() {
        let server = Server::run();
        server.expect(
            Expectation::matching(all_of![
                request::method_path("POST", "/foo"),
                request::body(url_decoded(contains(("user", "test"))))
            ])
            .respond_with(status_code(200)),
        );

        let url = server.url("/foo").to_string();
        let lib = StdHttpLibrary;

        let mut form = BTreeMap::new();
        form.insert("user".into(), "test".into());

        let res = lib.post(url, None, Some(form), None, None).unwrap();
        assert_eq!(res.get("status_code").unwrap(), &Value::Int(200));
    }
}
