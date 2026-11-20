use alloc::string::{String, ToString};
use anyhow::{Context, Result as AnyhowResult};
use flate2::Compression;
use flate2::write::GzEncoder;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use tar::{Builder, HeaderMode};

pub fn compress_file(src: String, dst: String) -> Result<(), String> {
    compress_impl(src, dst).map_err(|e| e.to_string())
}

fn compress_impl(src: String, dst: String) -> AnyhowResult<()> {
    let src_path = Path::new(&src);

    if src_path.is_dir() {
        let src_name = src_path.file_name().context("Failed to get file name")?;
        let f_dst = BufWriter::new(File::create(&dst)?);
        let deflater = GzEncoder::new(f_dst, Compression::fast());

        let mut tar_builder = Builder::new(deflater);
        tar_builder.mode(HeaderMode::Deterministic);
        tar_builder.append_dir_all(src_name, src_path)?;

        let deflater = tar_builder.into_inner()?;
        deflater.finish()?.flush()?;
        return Ok(());
    }

    let mut reader = BufReader::new(File::open(&src)?);
    let f_dst = BufWriter::new(File::create(&dst)?);
    let mut deflater = GzEncoder::new(f_dst, Compression::fast());

    ::std::io::copy(&mut reader, &mut deflater)?;
    deflater.finish()?.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::std::decompress_file_impl::decompress_file;
    use std::fs;
    use tempfile::{NamedTempFile, tempdir};

    #[test]
    fn test_compress_file() {
        let content = "Compression Test";
        let tmp_src = NamedTempFile::new().unwrap();
        let src_path = tmp_src.path().to_string_lossy().to_string();
        fs::write(&src_path, content).unwrap();

        let tmp_dst = NamedTempFile::new().unwrap();
        let dst_path = tmp_dst.path().to_string_lossy().to_string();

        compress_file(src_path, dst_path.clone()).unwrap();

        let meta = fs::metadata(&dst_path).unwrap();
        assert!(meta.len() > 0);

        let out_file = NamedTempFile::new().unwrap();
        let out_path = out_file.path().to_string_lossy().to_string();
        decompress_file(dst_path, out_path.clone()).unwrap();
        assert_eq!(fs::read_to_string(&out_path).unwrap(), content);
    }

    #[test]
    fn test_compress_dir_roundtrip() {
        let dir = tempdir().unwrap();
        let src_dir = dir.path().join("sample_dir");
        fs::create_dir_all(src_dir.join("sub")).unwrap();
        fs::write(src_dir.join("a.txt"), "alpha").unwrap();
        fs::write(src_dir.join("sub/b.txt"), "beta").unwrap();

        let archive_path = dir.path().join("sample.tar.gz");
        compress_file(
            src_dir.to_string_lossy().to_string(),
            archive_path.to_string_lossy().to_string(),
        )
        .unwrap();

        let extracted_dir = dir.path().join("extracted");
        decompress_file(
            archive_path.to_string_lossy().to_string(),
            extracted_dir.to_string_lossy().to_string(),
        )
        .unwrap();

        assert_eq!(
            fs::read_to_string(extracted_dir.join("sample_dir/a.txt")).unwrap(),
            "alpha"
        );
        assert_eq!(
            fs::read_to_string(extracted_dir.join("sample_dir/sub/b.txt")).unwrap(),
            "beta"
        );
    }

    #[test]
    fn test_compress_file_truncates_existing_dst() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("small.txt");
        let dst = dir.path().join("out.gz");
        fs::write(&src, "hi").unwrap();
        fs::write(&dst, vec![b'X'; 4096]).unwrap();

        compress_file(
            src.to_string_lossy().to_string(),
            dst.to_string_lossy().to_string(),
        )
        .unwrap();

        assert!(fs::metadata(&dst).unwrap().len() < 4096);

        let out = dir.path().join("out.txt");
        decompress_file(
            dst.to_string_lossy().to_string(),
            out.to_string_lossy().to_string(),
        )
        .unwrap();
        assert_eq!(fs::read_to_string(&out).unwrap(), "hi");
    }

    #[test]
    fn test_compress_missing_src_does_not_create_dst() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("does_not_exist.txt");
        let dst = dir.path().join("out.gz");

        let res = compress_file(
            src.to_string_lossy().to_string(),
            dst.to_string_lossy().to_string(),
        );
        assert!(res.is_err());
        assert!(!dst.exists());
    }
}
