use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use eldritch_core::Value;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use spin::RwLock;

pub fn post(
    mut uri: String,
    body: Option<String>,
    form: Option<BTreeMap<String, String>>,
    headers: Option<BTreeMap<String, String>>,
    allow_insecure: Option<bool>,
    proxy: Option<String>,
) -> Result<BTreeMap<String, Value>, String> {
    if !uri.starts_with("http://") && !uri.starts_with("https://") {
        uri = format!("https://{}", uri);
    }

    let mut builder = reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(allow_insecure.unwrap_or(false));
    if let Some(proxy_url) = proxy {
        let p = reqwest::Proxy::all(&proxy_url)
            .map_err(|e| format!("Invalid proxy URL {proxy_url}: {e}"))?;
        builder = builder.proxy(p);
    }
    let client = builder
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

#[cfg(test)]
mod tests {
    use super::*;
    use httptest::{
        Expectation, Server,
        matchers::{all_of, contains, request, url_decoded},
        responders::status_code,
    };

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

        let res = post(url, Some("request body".into()), None, None, None, None).unwrap();

        assert_eq!(res.get("status_code").unwrap(), &Value::Int(201));
    }

    #[test]
    fn test_post_default_https() {
        // If we provide an unroutable domain, it will fail to connect but should try https.
        let res = post(
            "this-domain-will-not-exist-ever-123.com".to_string(),
            None,
            None,
            None,
            None,
            None,
        );
        let err = res.unwrap_err();
        assert!(
            err.contains("https://this-domain-will-not-exist-ever-123.com"),
            "Error message should indicate it tried https://this-domain-will-not-exist-ever-123.com, but was: {}",
            err
        );
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

        let mut form = BTreeMap::new();
        form.insert("user".into(), "test".into());

        let res = post(url, None, Some(form), None, None, None).unwrap();
        assert_eq!(res.get("status_code").unwrap(), &Value::Int(200));
    }
}
