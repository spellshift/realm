use anyhow::Result;
use tokio::{
    io::{ AsyncWriteExt },
    fs::{ File },
};
use tokio_stream::StreamExt;
use std::path::PathBuf;

async fn handle_download(uri: String, dst: String) -> Result<()> {
    // Create our file 
    let mut dest = {
        let fname = PathBuf::from(dst);
        File::create(fname).await?
    };

    // Download as a stream of bytes.
    // there's no checking at all happening here, for anything
    let mut stream = reqwest::get(uri)
        .await?
        .bytes_stream();
    
    // Write the stream of bytes to the file in chunks
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        dest.write_all(&chunk).await?;
    }

    // Flush file writer
    dest.flush().await?;
    Ok(())
}

pub fn download(uri: String, dst: String) -> Result<()> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let response = runtime.block_on(
        handle_download(uri, dst)
    );

    match response {
        Ok(_) => Ok(()),
        Err(_) => return response,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httptest::{Server, Expectation, matchers::*, responders::*};
    use tempfile::NamedTempFile;
    use std::fs::{remove_file, read_to_string};

    #[test]
    fn test_download_file() -> anyhow::Result<()> {
        // running test http server
        let server = Server::run();
        server.expect(
            Expectation::matching(request::method_path("GET", "/foo"))
            .respond_with(status_code(200)
            .body("test body")),
        );

        // just using a temp file for its path
        let tmp_file = NamedTempFile::new()?;
        let path = String::from(tmp_file.path().to_str().unwrap()).clone();
        tmp_file.close()?;

        // reference test server uri
        let url = server.url("/foo").to_string();

        // run our code
        download(url, path.clone())?;

        // Read the file
        let contents = read_to_string(path.clone())
            .expect("Something went wrong reading the file");

        // check file written correctly
        assert_eq!(contents, "test body");

        // cleanup
        remove_file(path)?;

        Ok(())
    }
}
