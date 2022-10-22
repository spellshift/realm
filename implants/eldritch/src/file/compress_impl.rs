use std::{path::Path, fs::{OpenOptions, File, self}};

use anyhow::Result;
use tar::{Builder, HeaderMode};
use tempfile::NamedTempFile;

fn tar_dir(src: String) -> Result<String> {
    let src_path = Path::new(&src);

    let tmp_tar_file_src = NamedTempFile::new()?;
    let tmp_tar_file_src_path = format!("{}.tar", String::from(tmp_tar_file_src.path().to_str().unwrap()));
    let _ = tmp_tar_file_src.close();

    // Create the tar bulider
    let tmp_tar_file = File::create(tmp_tar_file_src_path.clone()).unwrap();
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

    Ok(tmp_tar_file_src_path.clone())
}

pub fn compress(src: String, dst: String) -> Result<()> {
    // Setup src path strings.
    let mut tmp_src = src.clone();
    let src_path = Path::new(&tmp_src);

    println!("Start");
    // If our source is a dir create a tarball and update the src file to the tar ball.
    if src_path.clone().is_dir() {
        println!("IsDir");
        tmp_src = tar_dir(tmp_src).unwrap();
    }
    println!("Readers");
    // Check if src is a dir.
    // if dir tar the directry up.
    let mut f_src = std::io::BufReader::new(std::fs::File::open(tmp_src.clone()).unwrap());
    let mut f_dst = std::io::BufWriter::new(
            OpenOptions::new()
            .create(true) //Do we want to create the file if it doesn't exist? - Yes!
            .write(true)    
            .open(dst.clone())?);
    println!("Compress");
    lzma_rs::xz_compress(&mut f_src, &mut f_dst).unwrap();
    println!("Done");
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::prelude::*, fs};
    use sha256::digest_file;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn test_compress_basic() -> anyhow::Result<()>{
        // Create files
        let mut tmp_file_src = NamedTempFile::new()?;
        let path_src = String::from(tmp_file_src.path().to_str().unwrap());
        let tmp_file_dst = NamedTempFile::new()?;
        let path_dst = String::from(tmp_file_dst.path().to_str().unwrap());
        let _res = tmp_file_dst.close();

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
        let mut tmp_dir_src = tempdir()?;
        let path_src = String::from(tmp_dir_src.path().to_str().unwrap());
        // for (i,v) in ["Hello", "World", "Goodbye"].iter().enumerate() {
        //     let tmp_file = format!("{}/{}.txt", path_src.clone(), i);
        //     let _res = fs::write(tmp_file, v);
        // }

        let tmp_file_dst = NamedTempFile::new()?;
        let path_dst = format!("{}.tar.xz", String::from(tmp_file_dst.path().to_str().unwrap()));
        let _res = tmp_file_dst.close();

        // Run our code 
        // Test with trailing slash.
        compress(format!("{}/", path_src.clone()), path_dst.clone())?;

        // Hash
        let hash = digest_file(path_dst.clone())?;

        // Compare
        assert_eq!(hash, "-");

        Ok(())
    }

    #[test]
    fn test_compress_hashdir() -> anyhow::Result<()>{
        let path_dst = "/tmp/test-compress.tar.xz".to_string();
        compress("/tmp/test-compress-hashdir/".to_string(), path_dst.clone())?;
        let hash = digest_file(path_dst.clone())?;
        assert_eq!(hash, "-");
        Ok(())
    }

}
