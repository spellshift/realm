use anyhow::Result;
use std::io::prelude::*;
use curl::easy::Easy;

// Will follow redirects
// Will replace dst file if it exists
pub fn download(uri: String, dst: String) -> Result<()> {
    //TODO: Configure proxy.
    //TODO: Configure header settings.

    // Setup vars.
    let mut response = Vec::new();
    let mut easy = Easy::new();
    // Setup requset.
    easy.url(&uri).unwrap();
    let _redirect = easy.follow_location(true);

    {
        // Download data to response vector.
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            response.extend_from_slice(data);
            Ok(data.len())
        }).unwrap();
        transfer.perform().unwrap();
    }
    {
        // Write response vector to file dst.
        let mut file = std::fs::File::create(dst)?;
        file.write_all(response.as_slice())?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::fs::remove_file;
    use std::path::Path;
    use sha256::digest_file;

    #[test]
    fn test_download_new() -> anyhow::Result<()>{
        // Remove test files just in case.
        let win_file = "/tmp/win_test_download_new";
        let _remove_res = remove_file(String::from(win_file));
    
        // Run our code.
        // :shrug: seems like a mostly static file to test downloading.
        download(String::from("https://www.dundeecity.gov.uk/sites/default/files/publications/civic_renewal_forms.zip"), String::from(win_file))?;

        // Make sure the hash matches.
        let input = Path::new(win_file);
        let val = digest_file(input)?;
        assert_eq!(val, "c9e07f4ca083a99dda2bc9062400f6b54afceba4c6ce355914fef66c9f018dd0");

        // Cleanup
        remove_file(String::from(win_file))?;
        Ok(())
    }    
    #[test]
    fn test_download_existing_dst() -> anyhow::Result<()>{
        // Remove test files just in case.
        let win_file = "/tmp/win_test_download_existing_dst";
        let _ = remove_file(String::from(win_file));

        // Create file to exist.
        let mut file = File::create(win_file).unwrap();
        file.write_all(b"Hello, world!\n")?;

        // Run our code.
        // :shrug: seems like a mostly static file to test downloading.
        download(String::from("https://www.dundeecity.gov.uk/sites/default/files/publications/civic_renewal_forms.zip"), String::from(win_file))?;

        // Make sure the hash matches.
        let input = Path::new(win_file);
        let val = digest_file(input).unwrap();
        assert_eq!(val, "c9e07f4ca083a99dda2bc9062400f6b54afceba4c6ce355914fef66c9f018dd0");

        // Cleanup
        remove_file(String::from(win_file))?;

        Ok(())
    }
    #[test]
    fn test_download_ftp_existing() -> anyhow::Result<()>{
        // Remove test files just in case.
        let win_file = "/tmp/win_test_download_ftp_existing";
        let _remove_res = remove_file(String::from(win_file));

        // Create file to exist.
        let mut file = File::create(win_file).unwrap();
        file.write_all(b"Hello, world!\n")?;

        // Run our code.
        // :shrug: seems like a mostly static file to test downloading.
        download(String::from("ftp://speedtest:speedtest@ftp.otenet.gr/test1Mb.db"), String::from(win_file))?;

        // Make sure the hash matches.
        let input = Path::new(win_file);
        let val = digest_file(input).unwrap();
        assert_eq!(val, "30e14955ebf1352266dc2ff8067e68104607e750abb9d3b36582b8af909fcb58");

        // Cleanup
        remove_file(String::from(win_file))?;

        Ok(())
    }
}
