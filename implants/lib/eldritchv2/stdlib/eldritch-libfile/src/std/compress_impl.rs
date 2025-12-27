#[cfg(feature = "stdlib")]
use anyhow::{Context, Result as AnyhowResult};
#[cfg(feature = "stdlib")]
use alloc::string::ToString;
#[cfg(feature = "stdlib")]
use alloc::string::String;
#[cfg(feature = "stdlib")]
use std::path::Path;
#[cfg(feature = "stdlib")]
use std::fs::File;

#[cfg(feature = "stdlib")]
pub fn compress(src: String, dst: String) -> Result<(), String> {
    compress_impl(src, dst).map_err(|e| e.to_string())
}

#[cfg(not(feature = "stdlib"))]
pub fn compress(_src: alloc::string::String, _dst: alloc::string::String) -> Result<(), alloc::string::String> {
    Err("compress requires stdlib feature".into())
}

#[cfg(feature = "stdlib")]
fn compress_impl(src: String, dst: String) -> AnyhowResult<()> {
    use flate2::Compression;
    use tempfile::NamedTempFile;
    use std::fs::OpenOptions;

    let src_path = Path::new(&src);

    // Determine if we need to tar
    let tmp_tar_file_src = NamedTempFile::new()?;
    let tmp_src = if src_path.is_dir() {
        let tmp_path = tmp_tar_file_src.path().to_str().unwrap().to_string();
        tar_dir(&src, &tmp_path)?;
        tmp_path
    } else {
        src.clone()
    };

    let f_src = ::std::io::BufReader::new(File::open(&tmp_src)?);
    let f_dst = ::std::io::BufWriter::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(&dst)?,
    );

    let mut deflater = flate2::write::GzEncoder::new(f_dst, Compression::fast());
    let mut reader = f_src;
    ::std::io::copy(&mut reader, &mut deflater)?;
    deflater.finish()?;

    Ok(())
}

#[cfg(feature = "stdlib")]
fn tar_dir(src: &str, dst: &str) -> AnyhowResult<()> {
    use tar::{Builder, HeaderMode};

    let src_path = Path::new(src);
    let file = File::create(dst)?;
    let mut tar_builder = Builder::new(file);
    tar_builder.mode(HeaderMode::Deterministic);

    let src_name = src_path.file_name().context("Failed to get file name")?;

    tar_builder.append_dir_all(src_name, src_path)?;
    tar_builder.finish()?;
    Ok(())
}

#[cfg(test)]
#[cfg(feature = "stdlib")]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::fs;

    #[test]
    fn test_compress() {
        let content = "Compression Test";
        let tmp_src = NamedTempFile::new().unwrap();
        let src_path = tmp_src.path().to_string_lossy().to_string();
        fs::write(&src_path, content).unwrap();

        let tmp_dst = NamedTempFile::new().unwrap();
        let dst_path = tmp_dst.path().to_string_lossy().to_string();

        compress(src_path.clone(), dst_path.clone()).unwrap();

        let meta = fs::metadata(&dst_path).unwrap();
        assert!(meta.len() > 0);
    }
}
