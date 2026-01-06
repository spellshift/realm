use alloc::format;
use alloc::string::String;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub fn download(
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

#[cfg(test)]
mod tests {
    use super::*;
    use httptest::{
        Expectation, Server,
        matchers::request,
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

        download(url, path.clone(), None).unwrap();

        let content = read_to_string(&path).unwrap();
        assert_eq!(content, "test body");
    }
}
