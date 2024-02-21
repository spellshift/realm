use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use starlark::collections::SmallMap;
use std::collections::HashMap;

pub fn post(
    uri: String,
    body: Option<String>,
    form: Option<SmallMap<String, String>>,
    headers: Option<SmallMap<String, String>>,
) -> Result<String> {
    let mut headers_map = HeaderMap::new();

    if let Some(h) = headers {
        for (k, v) in h {
            let name = HeaderName::from_bytes(k.as_bytes())?;
            let value = HeaderValue::from_bytes(v.as_bytes())?;
            headers_map.append(name, value);
        }
    }

    let client = reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;
    let req = client.post(uri.clone()).headers(headers_map.clone());

    if let Some(b) = body {
        #[cfg(debug_assertions)]
        log::info!(
            "eldritch sending HTTP POST request to '{}' with headers '{:#?}' and body '{}'",
            uri,
            headers_map,
            b.clone()
        );

        let resp = req.body(b).send()?.text()?;
        return Ok(resp);
    }

    if let Some(f) = form {
        let mut form_map = HashMap::new();
        for (k, v) in f {
            form_map.insert(k, v);
        }

        #[cfg(debug_assertions)]
        log::info!(
            "eldritch sending HTTP POST request to '{}' with headers '{:#?}' and form '{:#?}'",
            uri,
            headers_map,
            form_map.clone()
        );

        let resp = req.form(&form_map).send()?.text()?;
        return Ok(resp);
    }

    #[cfg(debug_assertions)]
    log::info!(
        "eldritch sending HTTP POST request to '{}' with headers '{:#?}'",
        uri,
        headers_map
    );

    let resp = req.send()?.text()?;
    Ok(resp)
}

#[cfg(test)]
mod tests {

    use super::*;
    use httptest::{matchers::*, responders::*, Expectation, Server};
    use starlark::collections::SmallMap;

    #[test]
    fn test_post_no_body_or_params_or_headers() -> anyhow::Result<()> {
        // running test http server
        let server = Server::run();
        server.expect(
            Expectation::matching(request::method_path("POST", "/foo"))
                .respond_with(status_code(200).body("test body")),
        );

        // reference test server uri
        let url = server.url("/foo").to_string();

        // run our code
        let contents = post(url, None, None, None)?;

        // check request returned correctly
        assert_eq!(contents, "test body");

        Ok(())
    }

    #[test]
    fn test_post_empty_params_and_headers() -> anyhow::Result<()> {
        // running test http server
        let server = Server::run();
        server.expect(
            Expectation::matching(request::method_path("POST", "/foo"))
                .respond_with(status_code(200).body("test body")),
        );

        // reference test server uri
        let url = server.url("/foo").to_string();

        // run our code
        let contents = post(url, None, Some(SmallMap::new()), Some(SmallMap::new()))?;

        // check request returned correctly
        assert_eq!(contents, "test body");

        Ok(())
    }

    #[test]
    fn test_post_with_params() -> anyhow::Result<()> {
        // running test http server
        let server = Server::run();
        let m = all_of![
            request::method_path("POST", "/foo"),
            request::body(url_decoded(contains(("a", "true")))),
            request::body(url_decoded(contains(("b", "bar")))),
            request::body(url_decoded(contains(("c", "3")))),
        ];
        server.expect(Expectation::matching(m).respond_with(status_code(200).body("test body")));

        // reference test server uri
        let url = server.url("/foo").to_string();

        // run our code
        let mut params = SmallMap::new();
        params.insert("a".to_string(), "true".to_string());
        params.insert("b".to_string(), "bar".to_string());
        params.insert("c".to_string(), "3".to_string());
        let contents = post(url, None, Some(params), None)?;

        // check request returned correctly
        assert_eq!(contents, "test body");

        Ok(())
    }

    #[test]
    fn test_post_with_headers() -> anyhow::Result<()> {
        // running test http server
        let server = Server::run();
        let m = all_of![
            request::method_path("POST", "/foo"),
            request::headers(contains(("a", "TRUE"))),
            request::headers(contains(("b", "bar"))),
        ];
        server.expect(Expectation::matching(m).respond_with(status_code(200).body("test body")));

        // reference test server uri
        let url = server.url("/foo").to_string();

        // run our code
        let mut headers = SmallMap::new();
        headers.insert("A".to_string(), "TRUE".to_string());
        headers.insert("b".to_string(), "bar".to_string());
        let contents = post(url, None, None, Some(headers))?;

        // check request returned correctly
        assert_eq!(contents, "test body");

        Ok(())
    }

    #[test]
    fn test_post_with_params_and_headers() -> anyhow::Result<()> {
        // running test http server
        let server = Server::run();
        let m = all_of![
            request::method_path("POST", "/foo"),
            request::headers(contains(("a", "TRUE"))),
            request::headers(contains(("b", "bar"))),
            request::body(url_decoded(contains(("c", "3")))),
        ];
        server.expect(Expectation::matching(m).respond_with(status_code(200).body("test body")));

        // reference test server uri
        let url = server.url("/foo").to_string();

        // run our code
        let mut headers = SmallMap::new();
        headers.insert("A".to_string(), "TRUE".to_string());
        headers.insert("b".to_string(), "bar".to_string());
        let mut params = SmallMap::new();
        params.insert("c".to_string(), "3".to_string());
        let contents = post(url, None, Some(params), Some(headers))?;

        // check request returned correctly
        assert_eq!(contents, "test body");

        Ok(())
    }

    #[test]
    fn test_post_with_body_and_header() -> anyhow::Result<()> {
        // running test http server
        let server = Server::run();
        let m = all_of![
            request::method_path("POST", "/foo"),
            request::headers(contains(("a", "TRUE"))),
            request::body("the quick brown fox jumps over the lazy dog"),
        ];
        server.expect(Expectation::matching(m).respond_with(status_code(200).body("test body")));

        // reference test server uri
        let url = server.url("/foo").to_string();

        // run our code
        let mut headers = SmallMap::new();
        headers.insert("A".to_string(), "TRUE".to_string());
        let contents = post(
            url,
            Some(String::from("the quick brown fox jumps over the lazy dog")),
            None,
            Some(headers),
        )?;

        // check request returned correctly
        assert_eq!(contents, "test body");

        Ok(())
    }
}
