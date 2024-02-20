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
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    if headers.is_some() {
        for (k, v) in headers.unwrap() {
            let name = HeaderName::from_bytes(k.as_bytes())?;
            let value = HeaderValue::from_bytes(v.as_bytes())?;
            headers_map.append(name, value);
        }
    }

    if body.is_some() {
        return runtime.block_on(handle_post(uri, body, None, headers_map));
    }

    if form.is_some() {
        let mut form_map = HashMap::new();
        for (k, v) in form.unwrap() {
            form_map.insert(k, v);
        }

        return runtime.block_on(handle_post(uri, None, Some(form_map), headers_map));
    }

    runtime.block_on(handle_post(uri, None, None, headers_map))
}

async fn handle_post(
    uri: String,
    body: Option<String>,
    form: Option<HashMap<String, String>>,
    headers: HeaderMap,
) -> Result<String> {
    #[cfg(debug_assertions)]
    log::info!(
        "eldritch sending HTTP POST request to '{}' with headers '{:#?}'",
        uri,
        headers
    );

    let client = reqwest::Client::new().post(uri).headers(headers);
    if body.is_some() {
        let resp = client.body(body.unwrap()).send().await?.text().await?;
        return Ok(resp);
    }
    if form.is_some() {
        let resp = client.form(&form.unwrap()).send().await?.text().await?;
        return Ok(resp);
    }
    let resp = client.send().await?.text().await?;
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
