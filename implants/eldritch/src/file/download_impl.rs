use anyhow::Result;
use std::io::{copy, Write};
use std::fs::File;
use std::path::PathBuf;

pub fn download(uri: String, dst: String) -> Result<()> {
    println!("Here");
    let mut dest = {
        let fname = PathBuf::from(dst);
        File::create(fname)?
    };
    println!("file");
    // there's no checking at all happening here, for anything
    // let resp = reqwest::blocking::get(uri)?;
    let mut stream = reqwest::get(uri)
        .await?
        .bytes_stream();
    println!("got");
    // let content = resp.text()?;
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        dest.write_all(&chunk).await?;
    }
    println!("Text");

    // copy(&mut content.as_bytes(), &mut dest)?;
    dest.flush().await?;
    println!("Written");
    Ok(())
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
        download(url, path.clone());

        // Read the file
        let contents = read_to_string(path.clone())
            .expect("Something went wrong reading the file");

        // check file written correctly
        assert_eq!(contents, "test body");

        // cleanup
        remove_file(path)?;

        Ok(())
    }
    #[test]
    fn test_download_big() -> anyhow::Result<()> {
        // running test http server
        // just using a temp file for its path
        let tmp_file = NamedTempFile::new()?;
        let path = "/tmp/bigfile.bin".to_string(); //String::from(tmp_file.path().to_str().unwrap()).clone();
        tmp_file.close()?;

        // reference test server uri
        let url = "https://speed.hetzner.de/1GB.bin".to_string();

        println!("Starting download");
        // run our code
        download(url, path.clone());

        println!("Finished download");

        remove_file(path)?;

        Ok(())
    }
}
