use anyhow::Result;
use std::io::copy;
use std::fs::File;
use std::path::PathBuf;

pub fn download(_uri: String, _dst: String) -> Result<()> {
    // there's no checking at all happening here, for anything
    let resp = reqwest::blocking::get(_uri)?;
    let content = resp.text()?;

    let mut dest = {
        let fname = PathBuf::from(_dst);
        File::create(fname)?
    };

    copy(&mut content.as_bytes(), &mut dest)?;

    Ok(())
}

// TODO: tests
#[cfg(test)]
mod tests {
    use super::*;
    use httptest::{Server, Expectation, matchers::*, responders::*};
    use tempfile::NamedTempFile;
    use std::fs::{remove_file, read_to_string};

    #[test]
    fn test_download_file() -> anyhow::Result<()> {
        println!("First bit");
        // running test http server
        let server = Server::run();
        server.expect(
            Expectation::matching(request::method_path("GET", "/foo"))
            .respond_with(status_code(200)
            .body("test body")),
        );
        println!("Second bit");

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
}
