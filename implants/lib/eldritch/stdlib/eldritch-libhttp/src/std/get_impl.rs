use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use eldritch_core::Value;
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use spin::RwLock;

pub fn get(
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

#[cfg(test)]
mod tests {
    use super::*;
    use httptest::{
        Expectation, Server,
        matchers::{all_of, contains, request, url_decoded},
        responders::status_code,
    };

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

        let res = get(url, None, None, None).unwrap();

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

        let mut params = BTreeMap::new();
        params.insert("q".into(), "search".into());

        let res = get(url, Some(params), None, None).unwrap();
        assert_eq!(res.get("status_code").unwrap(), &Value::Int(200));
    }
}
