use std::{path::Path, fs::{OpenOptions, File}};

use anyhow::Result;
use tar::{Builder, HeaderMode};
use tempfile::NamedTempFile;

fn tar_dir(src: String, dst: String) -> Result<String> {
    let src_path = Path::new(&src);

    // Create the tar bulider
    let tmp_tar_file = File::create(dst.clone()).unwrap();
    let mut tar_builder = Builder::new(tmp_tar_file);
    tar_builder.mode(
        HeaderMode::Deterministic
    );

    // Add all files from source dir with the name of the dir.
    let _ = tar_builder.append_dir_all(
        src_path.clone().file_name().unwrap().to_str().unwrap(), 
        src_path.clone()
    );
    let _ = tar_builder.finish();

    Ok(dst.clone())
}

pub fn compress(src: String, dst: String) -> Result<()> {
    // Setup src path strings.
    let mut tmp_src = src.clone();
    let src_path = Path::new(&tmp_src);

    let tmp_tar_file_src = NamedTempFile::new()?;
    let tmp_tar_file_src_path = String::from(tmp_tar_file_src.path().to_str().unwrap());

    // If our source is a dir create a tarball and update the src file to the tar ball.
    if src_path.clone().is_dir() {
        tmp_src = tar_dir(tmp_src, tmp_tar_file_src_path).unwrap();
    } else {
        let _ = tmp_tar_file_src.close();
    }
    // Check if src is a dir.
    // if dir tar the directry up.
    let mut f_src = std::io::BufReader::new(std::fs::File::open(tmp_src.clone()).unwrap());
    let mut f_dst = std::io::BufWriter::new(
            OpenOptions::new()
            .create(true) //Do we want to create the file if it doesn't exist? - Yes!
            .write(true)    
            .open(dst.clone())?);

    lzma_rs::xz_compress(&mut f_src, &mut f_dst).unwrap();

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::{prelude::*, BufReader}, fs};
    use sha256::digest_file;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn test_compress_basic() -> anyhow::Result<()>{
        // Create files
        let mut tmp_file_src = NamedTempFile::new()?;
        let path_src = String::from(tmp_file_src.path().to_str().unwrap());
        let tmp_file_dst = NamedTempFile::new()?;
        let path_dst = String::from(tmp_file_dst.path().to_str().unwrap());
        // let _res = tmp_file_dst.close();

        // Write to file
        tmp_file_src.write_all(b"Hello, world!")?;

        // Run our code
        compress(path_src, path_dst.clone())?;

        // Hash
        let hash = digest_file(path_dst.clone())?;

        // Compare
        assert_eq!(hash, "e2f5ed829643c7670c5b429b57abc2c6209c46502fb21911f7db1d561ac16424");

        Ok(())
    }

    #[test]
    fn test_compress_dir() -> anyhow::Result<()>{
        // Create files
        let tmp_dir_src = tempdir()?;
        let path_src = String::from(tmp_dir_src.path().to_str().unwrap());
        for (i,v) in ["Hello", "World", "Goodbye"].iter().enumerate() {
            let tmp_file = format!("{}/{}.txt", path_src.clone(), i);
            let _res = fs::write(tmp_file, v);
        }

        let tmp_file_dst = NamedTempFile::new()?;
        let path_dst = String::from(tmp_file_dst.path().to_str().unwrap());

        // Run our code 
        // Test with trailing slash.
        compress(format!("{}/", path_src.clone()), path_dst.clone())?;

        // Test that no errors were raised.
        assert_eq!(true, true);

        Ok(())
    }

    #[test]
    fn test_compress_tar_dir() -> anyhow::Result<()>{
        // Create files
        let tmp_dir_src = tempdir()?;
        let path_src = String::from(tmp_dir_src.path().to_str().unwrap());
        for (i,v) in ["Hello", "World", "Goodbye"].iter().enumerate() {
            let tmp_file = format!("{}/{}.txt", path_src.clone(), i);
            let _res = fs::write(tmp_file, v);
        }

        let tmp_file_dst = NamedTempFile::new()?;
        let path_dst = String::from(tmp_file_dst.path().to_str().unwrap());

        // Run our code 
        // Test with trailing slash.
        tar_dir(format!("{}/", path_src.clone()), path_dst.clone())?;

        // Test for known strings.
        let mut res = 0;
        let searchstrings = ["Hello", "World", "Goodbye", "/0.txt", "/1.txt", "/2.txt", "ustar"];
        for cur_string in searchstrings {
            let tar_file = File::open(path_dst.clone())?;
            let reader = BufReader::new(tar_file);    
            for line in reader.lines(){
                let line = line.unwrap();
                if line.contains(cur_string){
                    res += 1;
                }
            }
        }
        
        assert_eq!(res, searchstrings.len());
        Ok(())
    }
}
