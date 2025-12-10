use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
};

use anyhow::{Context, Result};
use flate2::Compression;
use tar::{Builder, HeaderMode};
use tempfile::NamedTempFile;

fn tar_dir(src: String, dst: String) -> Result<String> {
    let src_path = Path::new(&src);

    // Create the tar bulider
    let tmp_tar_file = match File::create(dst.clone()) {
        Ok(file) => file,
        Err(error) => {
            return Err(anyhow::anyhow!(
                "File {} failed to create.\n{:?}",
                dst.clone(),
                error
            ))
        }
    };
    let mut tar_builder = Builder::new(tmp_tar_file);
    tar_builder.mode(HeaderMode::Deterministic);

    let src_path_obj = src_path
        .file_name()
        .context(format!(
            "Failed to get path file_name {}",
            src_path.display()
        ))?
        .to_str()
        .context(format!(
            "Failed to convert osStr to str {}",
            src_path.display()
        ))?;

    // Add all files from source dir with the name of the dir.
    match tar_builder.append_dir_all(src_path_obj, src_path) {
        Ok(_) => {}
        Err(error) => {
            return Err(anyhow::anyhow!(
                "Appending dir {} failed.\n{:?}",
                src_path_obj,
                error
            ))
        }
    }

    match tar_builder.finish() {
        Ok(_) => {}
        Err(error) => {
            return Err(anyhow::anyhow!(
                "Error creating tar ball failed\n{:?}",
                error
            ))
        }
    }

    Ok(dst.clone())
}

pub fn compress(src: String, dst: String) -> Result<()> {
    // Setup src path strings.
    let mut tmp_src = src.clone();
    let src_path = Path::new(&tmp_src);

    let tmp_tar_file_src = NamedTempFile::new()?;
    let tmp_tar_file_src_path = String::from(
        tmp_tar_file_src
            .path()
            .to_str()
            .context(format!("Faild to get path str: {}", src))?,
    );

    // If our source is a dir create a tarball and update the src file to the tar ball.
    if src_path.is_dir() {
        tmp_src = tar_dir(tmp_src, tmp_tar_file_src_path)?
    } else {
        let _ = tmp_tar_file_src.close();
    }

    // Setup buffered reader writer.
    let f_src = std::io::BufReader::new(std::fs::File::open(tmp_src.clone())?);
    let mut f_dst = std::io::BufWriter::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(dst.clone())?,
    );

    let mut deflater = flate2::bufread::GzEncoder::new(f_src, Compression::fast());

    // Write
    let read_buffer_size = 1024 * 50;
    let mut bytes_read = read_buffer_size;
    while bytes_read != 0 {
        let mut buffer: Vec<u8> = vec![0; read_buffer_size];
        bytes_read = deflater.read(&mut buffer)?;
        let _ = f_dst.write_all(&buffer[0..bytes_read]);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha256::try_digest;
    use std::{
        fs,
        io::{prelude::*, BufReader},
    };
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn test_compress_basic() -> anyhow::Result<()> {
        // Create files
        let mut tmp_file_src = NamedTempFile::new()?;
        let path_src = String::from(tmp_file_src.path().to_str().unwrap());
        let tmp_file_dst = NamedTempFile::new()?;
        let path_dst = String::from(tmp_file_dst.path().to_str().unwrap());

        // Write to file
        tmp_file_src.write_all(b"Hello, world!")?;

        // Run our code
        compress(path_src, path_dst.clone())?;

        // Hash
        let hash = try_digest(tmp_file_dst.path())?;

        // Compare
        assert_eq!(
            hash,
            "a4a62449deb847be376f523c527ddb8e37eda2a4a71dd293d3ddcd4c4a81941f"
        );

        Ok(())
    }

    #[test]
    fn test_compress_tar_dir() -> anyhow::Result<()> {
        // Create files
        let tmp_dir_src = tempdir()?;
        let path_src = String::from(tmp_dir_src.path().to_str().unwrap());
        for (i, v) in ["Hello", "World", "Goodbye"].iter().enumerate() {
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
        let searchstrings = [
            "Hello", "World", "Goodbye", "/0.txt", "/1.txt", "/2.txt", "ustar",
        ];
        for cur_string in searchstrings {
            let tar_file = File::open(path_dst.clone())?;
            let reader = BufReader::new(tar_file);
            for line in reader.lines() {
                let line = line.unwrap();
                if line.contains(cur_string) {
                    res += 1;
                }
            }
        }

        assert_eq!(res, searchstrings.len());
        Ok(())
    }
}
