use std::io::Read;

use anyhow::Result;
use tar::Archive;
use tempfile::tempdir;

pub fn decompress(src: String, dst: String) -> Result<()> {
    let f_src = std::io::BufReader::new(std::fs::File::open(src.clone())?);
    let mut inflater = flate2::bufread::GzDecoder::new(f_src);

    let mut decoded_data: Vec<u8> = Vec::new();
    inflater.read_to_end(&mut decoded_data)?;

    std::fs::write(dst.clone() + "_debug.tar", &decoded_data)?;

    let tmp_tar_dst = tempdir()?;

    // Attempt to read as a tarball, indicating this is a directory
    match Archive::new(decoded_data.as_slice()).unpack(&tmp_tar_dst) {
        Ok(_entries) => {
            // Move the temp dir to the dst
            std::fs::rename(tmp_tar_dst.path(), std::path::Path::new(&dst))?;
            Ok(())
        }
        // Is a single file, write it to dst
        Err(_) => {
            std::fs::write(dst, decoded_data)?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::file::compress_impl::compress;

    use super::*;
    use std::{fs, io::prelude::*, path::Path};
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn test_decompress_basic() -> anyhow::Result<()> {
        let content = "Hello, world!";

        // Create files
        let mut tmp_file_src = NamedTempFile::new()?;
        let path_src = String::from(tmp_file_src.path().to_str().unwrap());
        let path_dst = path_src.clone() + "_compressed.gz";

        // Write to file
        tmp_file_src.write_all(content.as_bytes())?;

        // Run our code
        compress(path_src.clone(), path_dst.clone())?;

        decompress(path_dst.clone(), path_src.clone() + "_decompressed")?;

        // Read decompressed file
        let decompressed_content = fs::read_to_string(path_src.clone() + "_decompressed")?;
        assert_eq!(decompressed_content, content);

        Ok(())
    }

    #[test]
    fn test_decompress_tar_dir() -> anyhow::Result<()> {
        // Create files
        let tmp_dir_src = tempdir()?;
        let src_path = String::from(tmp_dir_src.path().to_str().unwrap());
        let src_name = tmp_dir_src.path().file_name().unwrap().to_str().unwrap();
        let dst_path = src_path.clone() + "_compressed.tar.gz";
        let decompressed_path = src_path.clone() + "_decompressed";
        let inner_decompresed_path = Path::new(&decompressed_path.clone())
            .join(src_name)
            .to_str()
            .unwrap()
            .to_string();

        let test_data = ["Hello", "World", "Goodbye"];

        for (i, v) in test_data.iter().enumerate() {
            let tmp_file_path = tmp_dir_src.path().join(format!("{}.txt", i));
            let _res = fs::write(tmp_file_path, v);
        }

        compress(src_path.clone(), dst_path.clone())?;

        decompress(dst_path, decompressed_path.clone())?;

        // Read decompressed files
        for (i, v) in test_data.iter().enumerate() {
            let decompressed_file = Path::new(&inner_decompresed_path)
                .join(format!("{}.txt", i))
                .to_str()
                .unwrap()
                .to_string();
            let decompressed_content = fs::read_to_string(decompressed_file)?;
            assert_eq!(decompressed_content, *v);
        }

        Ok(())
    }
}
