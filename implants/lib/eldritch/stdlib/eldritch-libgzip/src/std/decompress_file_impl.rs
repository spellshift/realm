use alloc::string::{String, ToString};
use alloc::vec::Vec;
use anyhow::Result as AnyhowResult;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

pub fn decompress_file(src: String, dst: String) -> Result<(), String> {
    decompress_impl(src, dst).map_err(|e| e.to_string())
}

fn is_tar_archive(data: &[u8]) -> bool {
    let mut archive = tar::Archive::new(data);
    let Ok(entries) = archive.entries() else {
        return false;
    };

    let mut has_entries = false;
    for entry in entries {
        let Ok(entry) = entry else {
            return false;
        };
        if entry.raw_file_position().saturating_add(entry.size()) > data.len() as u64 {
            return false;
        }
        has_entries = true;
    }

    has_entries
}

fn decompress_impl(src: String, dst: String) -> AnyhowResult<()> {
    let f_src = ::std::io::BufReader::new(File::open(&src)?);
    let mut decoder = flate2::read::GzDecoder::new(f_src);

    let mut decoded_data = Vec::new();
    decoder.read_to_end(&mut decoded_data)?;

    if !is_tar_archive(&decoded_data) {
        fs::write(&dst, decoded_data)?;
        return Ok(());
    }

    let dst_path = Path::new(&dst);
    if dst_path.is_file() {
        fs::remove_file(dst_path)?;
    }
    fs::create_dir_all(dst_path)?;

    let mut archive = tar::Archive::new(decoded_data.as_slice());
    archive.unpack(dst_path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::io::Write;
    use tempfile::{NamedTempFile, tempdir};

    fn write_gzip_bytes(path: &Path, data: &[u8]) {
        let file = File::create(path).unwrap();
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(data).unwrap();
        encoder.finish().unwrap();
    }

    fn create_tar_gz(path: &Path, files: &[(&str, &[u8])]) {
        let file = File::create(path).unwrap();
        let encoder = GzEncoder::new(file, Compression::default());
        let mut tar_builder = tar::Builder::new(encoder);
        for (name, content) in files {
            let mut header = tar::Header::new_gnu();
            header.set_size(content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar_builder
                .append_data(&mut header, name, &content[..])
                .unwrap();
        }
        let encoder = tar_builder.into_inner().unwrap();
        encoder.finish().unwrap();
    }

    #[test]
    fn test_decompress_file() {
        let content = "Decompression Test";
        let tmp_src = NamedTempFile::new().unwrap();
        let src_path = tmp_src.path().to_string_lossy().to_string();

        write_gzip_bytes(tmp_src.path(), content.as_bytes());

        let tmp_dst = NamedTempFile::new().unwrap();
        let dst_path = tmp_dst.path().to_string_lossy().to_string();

        decompress_file(src_path, dst_path.clone()).unwrap();

        let res = fs::read_to_string(&dst_path).unwrap();
        assert_eq!(res, content);
    }

    #[test]
    fn test_decompress_large_non_tar_and_zeros() {
        let dir = tempdir().unwrap();

        // Non-tar > 512 bytes
        let text_content = "A".repeat(2048);
        let text_src = dir.path().join("text.gz");
        let text_dst = dir.path().join("text.out");
        write_gzip_bytes(&text_src, text_content.as_bytes());
        decompress_file(
            text_src.to_string_lossy().to_string(),
            text_dst.to_string_lossy().to_string(),
        )
        .unwrap();
        assert!(text_dst.is_file());
        assert_eq!(fs::read_to_string(&text_dst).unwrap(), text_content);

        // All-zero bytes (should not be treated as an empty tar)
        let zero_content = vec![0u8; 2048];
        let zero_src = dir.path().join("zeros.gz");
        let zero_dst = dir.path().join("zeros.out");
        write_gzip_bytes(&zero_src, &zero_content);
        decompress_file(
            zero_src.to_string_lossy().to_string(),
            zero_dst.to_string_lossy().to_string(),
        )
        .unwrap();
        assert!(zero_dst.is_file());
        assert_eq!(fs::read(&zero_dst).unwrap(), zero_content);
    }

    #[test]
    fn test_decompress_tar_gz() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("archive.tar.gz");
        let dst = dir.path().join("extracted");

        create_tar_gz(
            &src,
            &[
                ("hello.txt", b"hello world"),
                ("nested/sub.txt", b"nested content"),
            ],
        );

        decompress_file(
            src.to_string_lossy().to_string(),
            dst.to_string_lossy().to_string(),
        )
        .unwrap();

        assert!(dst.is_dir());
        assert_eq!(
            fs::read_to_string(dst.join("hello.txt")).unwrap(),
            "hello world"
        );
        assert_eq!(
            fs::read_to_string(dst.join("nested/sub.txt")).unwrap(),
            "nested content"
        );
    }

    #[test]
    fn test_decompress_tar_gz_into_existing_dir() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("archive.tar.gz");
        let dst = dir.path().join("existing_dir");
        fs::create_dir_all(&dst).unwrap();

        create_tar_gz(&src, &[("file.txt", b"payload")]);

        decompress_file(
            src.to_string_lossy().to_string(),
            dst.to_string_lossy().to_string(),
        )
        .unwrap();

        assert_eq!(fs::read_to_string(dst.join("file.txt")).unwrap(), "payload");
    }

    #[test]
    fn test_decompress_tar_gz_long_filename_and_replaces_file() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("long.tar.gz");
        let dst = dir.path().join("dst_was_file");
        fs::write(&dst, b"old file").unwrap();

        let long_name = format!("dir/{}.txt", "a".repeat(160));
        {
            let file = File::create(&src).unwrap();
            let encoder = GzEncoder::new(file, Compression::default());
            let mut tar_builder = tar::Builder::new(encoder);
            let payload = b"long path payload";
            let mut header = tar::Header::new_gnu();
            header.set_size(payload.len() as u64);
            header.set_mode(0o644);
            tar_builder
                .append_data(&mut header, &long_name, &payload[..])
                .unwrap();
            let encoder = tar_builder.into_inner().unwrap();
            encoder.finish().unwrap();
        }

        decompress_file(
            src.to_string_lossy().to_string(),
            dst.to_string_lossy().to_string(),
        )
        .unwrap();

        assert!(dst.is_dir());
        assert_eq!(
            fs::read_to_string(dst.join(&long_name)).unwrap(),
            "long path payload"
        );
    }

    #[test]
    fn test_decompress_empty_gzip() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("empty.gz");
        let dst = dir.path().join("empty.out");
        write_gzip_bytes(&src, b"");

        decompress_file(
            src.to_string_lossy().to_string(),
            dst.to_string_lossy().to_string(),
        )
        .unwrap();

        assert!(dst.is_file());
        assert!(fs::read(&dst).unwrap().is_empty());
    }
}
