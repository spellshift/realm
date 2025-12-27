#[cfg(feature = "stdlib")]
use anyhow::Result as AnyhowResult;
#[cfg(feature = "stdlib")]
use alloc::string::ToString;
#[cfg(feature = "stdlib")]
use alloc::string::String;
#[cfg(feature = "stdlib")]
use alloc::vec::Vec;
#[cfg(feature = "stdlib")]
use std::fs::{self, File};
#[cfg(feature = "stdlib")]
use std::path::Path;
#[cfg(feature = "stdlib")]
use std::io::Read;

#[cfg(feature = "stdlib")]
pub fn decompress(src: String, dst: String) -> Result<(), String> {
    decompress_impl(src, dst).map_err(|e| e.to_string())
}

#[cfg(not(feature = "stdlib"))]
pub fn decompress(_src: alloc::string::String, _dst: alloc::string::String) -> Result<(), alloc::string::String> {
    Err("decompress requires stdlib feature".into())
}

#[cfg(feature = "stdlib")]
fn decompress_impl(src: String, dst: String) -> AnyhowResult<()> {
    use tar::Archive;

    let f_src = ::std::io::BufReader::new(File::open(&src)?);
    let mut decoder = flate2::read::GzDecoder::new(f_src);

    let mut decoded_data = Vec::new();
    decoder.read_to_end(&mut decoded_data)?;

    // Try as tar
    // Create a temp dir to verify if it is a tar
    if Archive::new(decoded_data.as_slice()).entries().is_ok() {
        // It's likely a tar

        let dst_path = Path::new(&dst);
        if !dst_path.exists() {
            fs::create_dir_all(dst_path)?;
        }

        let mut archive = Archive::new(decoded_data.as_slice());

        let tmp_dir = tempfile::tempdir()?;
        match archive.unpack(tmp_dir.path()) {
            Ok(_) => {
                if dst_path.exists() {
                    fs::remove_dir_all(dst_path).ok(); // ignore fail
                }

                // Keep the temp dir content by moving it
                let path = tmp_dir.keep();
                fs::rename(&path, &dst)?;
                Ok(())
            }
            Err(_) => {
                // Not a tar or unpack failed. Write raw bytes.
                if dst_path.exists() && dst_path.is_dir() {
                    fs::remove_dir_all(dst_path)?;
                }
                fs::write(&dst, decoded_data)?;
                Ok(())
            }
        }
    } else {
        // Not a tar
        fs::write(&dst, decoded_data)?;
        Ok(())
    }
}

#[cfg(test)]
#[cfg(feature = "stdlib")]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    use flate2::write::GzEncoder;
    use flate2::Compression;

    #[test]
    fn test_decompress() {
        let content = "Decompression Test";
        let tmp_src = NamedTempFile::new().unwrap();
        let src_path = tmp_src.path().to_string_lossy().to_string();

        // Create a gzip file manually
        let file = File::create(&src_path).unwrap();
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(content.as_bytes()).unwrap();
        encoder.finish().unwrap();

        let tmp_dst = NamedTempFile::new().unwrap();
        let dst_path = tmp_dst.path().to_string_lossy().to_string();

        decompress(src_path, dst_path.clone()).unwrap();

        let res = fs::read_to_string(&dst_path).unwrap();
        assert_eq!(res, content);
    }
}
