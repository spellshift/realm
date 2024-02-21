use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use starlark::collections::SmallMap;
use std::collections::HashMap;

pub fn get(
    uri: String,
    query_params: Option<SmallMap<String, String>>,
    headers: Option<SmallMap<String, String>>,
) -> Result<String> {
    let mut query_map = HashMap::new();
    let mut headers_map = HeaderMap::new();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    if let Some(q) = query_params {
        for (k, v) in q {
            query_map.insert(k, v);
        }
    }

    if let Some(h) = headers {
        for (k, v) in h {
            let name = HeaderName::from_bytes(k.as_bytes())?;
            let value = HeaderValue::from_bytes(v.as_bytes())?;
            headers_map.append(name, value);
        }
    }

    runtime.block_on(handle_get(uri, query_map, headers_map))
}

async fn handle_get(
    uri: String,
    query_params: HashMap<String, String>,
    headers: HeaderMap,
) -> Result<String> {
    #[cfg(debug_assertions)]
    log::info!(
        "eldritch sending HTTP GET request to '{}' with headers '{:#?}'",
        uri,
        headers
    );

    let client = reqwest::Client::new()
        .get(uri)
        .headers(headers)
        .query(&query_params);
    let resp = client.send().await?.text().await?;
    Ok(resp)
}

#[cfg(test)]
mod tests {

    use super::*;
    use httptest::{matchers::*, responders::*, Expectation, Server};
    use starlark::collections::SmallMap;

    #[test]
    fn test_get_no_params_or_headers() -> anyhow::Result<()> {
        // running test http server
        let server = Server::run();
        server.expect(
            Expectation::matching(request::method_path("GET", "/foo"))
                .respond_with(status_code(200).body("test body")),
        );

        // reference test server uri
        let url = server.url("/foo").to_string();

        // run our code
        let contents = get(url, None, None)?;

        // check request returned correctly
        assert_eq!(contents, "test body");

        Ok(())
    }

    #[test]
    fn test_get_empty_params_and_headers() -> anyhow::Result<()> {
        // running test http server
        let server = Server::run();
        server.expect(
            Expectation::matching(request::method_path("GET", "/foo"))
                .respond_with(status_code(200).body("test body")),
        );

        // reference test server uri
        let url = server.url("/foo").to_string();

        // run our code
        let contents = get(url, Some(SmallMap::new()), Some(SmallMap::new()))?;

        // check request returned correctly
        assert_eq!(contents, "test body");

        Ok(())
    }

    #[test]
    fn test_get_with_params() -> anyhow::Result<()> {
        // running test http server
        let server = Server::run();
        let m = all_of![
            request::method_path("GET", "/foo"),
            request::query(url_decoded(contains(("a", "true")))),
            request::query(url_decoded(contains(("b", "bar")))),
            request::query(url_decoded(contains(("c", "3")))),
        ];
        server.expect(Expectation::matching(m).respond_with(status_code(200).body("test body")));

        // reference test server uri
        let url = server.url("/foo").to_string();

        // run our code
        let mut params = SmallMap::new();
        params.insert("a".to_string(), "true".to_string());
        params.insert("b".to_string(), "bar".to_string());
        params.insert("c".to_string(), "3".to_string());
        let contents = get(url, Some(params), None)?;

        // check request returned correctly
        assert_eq!(contents, "test body");

        Ok(())
    }

    #[test]
    fn test_get_with_hybrid_params() -> anyhow::Result<()> {
        // running test http server
        let server = Server::run();
        let m = all_of![
            request::method_path("GET", "/foo"),
            request::query(url_decoded(contains(("a", "true")))),
            request::query(url_decoded(contains(("b", "bar")))),
            request::query(url_decoded(contains(("c", "3")))),
        ];
        server.expect(Expectation::matching(m).respond_with(status_code(200).body("test body")));

        // reference test server uri
        let url = server.url("/foo?a=true").to_string();

        // run our code
        let mut params = SmallMap::new();
        params.insert("b".to_string(), "bar".to_string());
        params.insert("c".to_string(), "3".to_string());
        let contents = get(url, Some(params), None)?;

        // check request returned correctly
        assert_eq!(contents, "test body");

        Ok(())
    }

    #[test]
    fn test_get_with_headers() -> anyhow::Result<()> {
        // running test http server
        let server = Server::run();
        let m = all_of![
            request::method_path("GET", "/foo"),
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
        let contents = get(url, None, Some(headers))?;

        // check request returned correctly
        assert_eq!(contents, "test body");

        Ok(())
    }

    #[test]
    fn test_get_with_params_and_headers() -> anyhow::Result<()> {
        // running test http server
        let server = Server::run();
        let m = all_of![
            request::method_path("GET", "/foo"),
            request::headers(contains(("a", "TRUE"))),
            request::headers(contains(("b", "bar"))),
            request::query(url_decoded(contains(("c", "3")))),
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
        let contents = get(url, Some(params), Some(headers))?;

        // check request returned correctly
        assert_eq!(contents, "test body");

        Ok(())
    }
}
