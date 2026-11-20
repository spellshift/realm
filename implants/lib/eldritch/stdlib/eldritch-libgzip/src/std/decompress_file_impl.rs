use crate::std::is_same_file;
use alloc::string::{String, ToString};
use anyhow::{Result as AnyhowResult, anyhow};
use flate2::bufread::MultiGzDecoder;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::Path;

pub fn decompress_file(src: String, dst: String) -> Result<(), String> {
    decompress_impl(src, dst).map_err(|e| e.to_string())
}

/// Captures any underlying gzip / I/O error separately from tar parsing errors
/// so corrupt gzip streams fail immediately instead of falling back to raw writes.
struct GzipCheckReader<R> {
    inner: R,
    io_err: Option<io::Error>,
}

impl<R: Read> Read for GzipCheckReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.inner.read(buf) {
            Ok(n) => Ok(n),
            Err(e) => {
                let kind = e.kind();
                let msg = e.to_string();
                self.io_err = Some(e);
                Err(io::Error::new(kind, msg))
            }
        }
    }
}

/// Pass 1: streams the gzip file into `tar::Archive::entries()` in O(1) memory.
/// - Returns `Err` if the underlying gzip stream or I/O fails.
/// - Returns `Ok(true)` if the stream is a valid non-empty tar archive (including
///   GNU sparse entries, PAX/USTAR headers, and multi-member gzip streams).
/// - Returns `Ok(false)` for valid non-tar gzip files (including empty / all-zero).
fn inspect_gzip_for_tar(src_path: &Path) -> AnyhowResult<bool> {
    let f_src = BufReader::new(File::open(src_path)?);
    let decoder = MultiGzDecoder::new(f_src);
    let reader = GzipCheckReader {
        inner: decoder,
        io_err: None,
    };

    let mut archive = tar::Archive::new(reader);
    let mut has_entries = false;
    let mut is_valid_tar = true;

    match archive.entries() {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        if entry.path().is_err() {
                            is_valid_tar = false;
                            break;
                        }
                        has_entries = true;
                    }
                    Err(_) => {
                        is_valid_tar = false;
                        break;
                    }
                }
            }
        }
        Err(_) => {
            is_valid_tar = false;
        }
    }

    let mut reader = archive.into_inner();
    if let Some(err) = reader.io_err.take() {
        return Err(err.into());
    }

    if !is_valid_tar || !has_entries {
        return Ok(false);
    }

    // Drain any remaining bytes so MultiGzDecoder validates the gzip trailer/CRC32
    // and any concatenated members before touching dst.
    io::copy(&mut reader.inner, &mut io::sink())?;
    Ok(true)
}

fn remove_symlink(path: &Path) -> io::Result<()> {
    #[cfg(windows)]
    {
        if fs::remove_file(path).is_err() {
            fs::remove_dir(path)?;
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        fs::remove_file(path)
    }
}

fn prepare_tar_dst(dst_path: &Path) -> AnyhowResult<()> {
    if let Ok(meta) = fs::symlink_metadata(dst_path) {
        if meta.file_type().is_symlink() {
            remove_symlink(dst_path)?;
        } else if !meta.is_dir() {
            fs::remove_file(dst_path)?;
        }
    }
    fs::create_dir_all(dst_path)?;
    Ok(())
}

fn decompress_impl(src: String, dst: String) -> AnyhowResult<()> {
    let src_path = Path::new(&src);
    let dst_path = Path::new(&dst);

    if is_same_file(src_path, dst_path)? {
        return Err(anyhow!(
            "Source and destination refer to the same file: {src}"
        ));
    }

    let is_tar = inspect_gzip_for_tar(src_path)?;

    if !is_tar {
        let dst_existed = dst_path.symlink_metadata().is_ok();
        let copy_res = (|| -> AnyhowResult<()> {
            let f_src = BufReader::new(File::open(src_path)?);
            let mut decoder = MultiGzDecoder::new(f_src);
            let mut writer = BufWriter::new(File::create(dst_path)?);
            io::copy(&mut decoder, &mut writer)?;
            writer.flush()?;
            Ok(())
        })();

        if let Err(err) = copy_res {
            if !dst_existed {
                let _ = fs::remove_file(dst_path);
            }
            return Err(err);
        }
        return Ok(());
    }

    prepare_tar_dst(dst_path)?;

    let f_src = BufReader::new(File::open(src_path)?);
    let decoder = MultiGzDecoder::new(f_src);
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(dst_path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::std::compress_file_impl::compress_file;
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
    fn test_decompress_tar_gz_merges_into_existing_dir() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("archive.tar.gz");
        let dst = dir.path().join("existing_dir");
        fs::create_dir_all(&dst).unwrap();
        fs::write(dst.join("keep.txt"), "preexisting").unwrap();

        create_tar_gz(&src, &[("file.txt", b"payload")]);

        decompress_file(
            src.to_string_lossy().to_string(),
            dst.to_string_lossy().to_string(),
        )
        .unwrap();

        assert_eq!(fs::read_to_string(dst.join("file.txt")).unwrap(), "payload");
        assert_eq!(
            fs::read_to_string(dst.join("keep.txt")).unwrap(),
            "preexisting"
        );
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

    #[test]
    fn test_decompress_sparse_tar_gz_roundtrip() {
        let dir = tempdir().unwrap();
        let src_dir = dir.path().join("sparse_dir");
        fs::create_dir_all(&src_dir).unwrap();

        // Create a sparse file with a large hole (1 MiB) and non-zero tail.
        let sparse_file_path = src_dir.join("sparse.bin");
        {
            use std::io::{Seek, SeekFrom};
            let mut f = File::create(&sparse_file_path).unwrap();
            f.write_all(b"HEAD").unwrap();
            f.seek(SeekFrom::Start(1024 * 1024)).unwrap();
            f.write_all(b"TAIL").unwrap();
        }

        let archive = dir.path().join("sparse.tar.gz");
        compress_file(
            src_dir.to_string_lossy().to_string(),
            archive.to_string_lossy().to_string(),
        )
        .unwrap();

        let extracted = dir.path().join("extracted");
        decompress_file(
            archive.to_string_lossy().to_string(),
            extracted.to_string_lossy().to_string(),
        )
        .unwrap();

        assert!(extracted.is_dir());
        let unpacked = fs::read(extracted.join("sparse_dir/sparse.bin")).unwrap();
        assert_eq!(unpacked.len(), 1024 * 1024 + 4);
        assert_eq!(&unpacked[..4], b"HEAD");
        assert_eq!(&unpacked[1024 * 1024..], b"TAIL");
    }

    #[test]
    fn test_decompress_multi_member_gzip() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("multi.gz");
        let dst = dir.path().join("multi.out");

        let mut combined = Vec::new();
        for part in [b"hello ".as_slice(), b"multi-member ".as_slice(), b"world!"] {
            let mut enc = GzEncoder::new(Vec::new(), Compression::default());
            enc.write_all(part).unwrap();
            combined.extend_from_slice(&enc.finish().unwrap());
        }
        fs::write(&src, combined).unwrap();

        decompress_file(
            src.to_string_lossy().to_string(),
            dst.to_string_lossy().to_string(),
        )
        .unwrap();

        assert_eq!(
            fs::read_to_string(&dst).unwrap(),
            "hello multi-member world!"
        );
    }

    #[test]
    fn test_decompress_corrupt_gzip_and_truncated_tar() {
        let dir = tempdir().unwrap();

        // Corrupt gzip bytes must return Err and not create dst
        let bad_gz = dir.path().join("bad.gz");
        let bad_dst = dir.path().join("bad.out");
        fs::write(&bad_gz, b"not valid gzip data").unwrap();
        assert!(
            decompress_file(
                bad_gz.to_string_lossy().to_string(),
                bad_dst.to_string_lossy().to_string(),
            )
            .is_err()
        );
        assert!(!bad_dst.exists());

        // Truncated tar (mid-payload) inside valid gzip should not unpack as tar
        let mut tar_bytes = Vec::new();
        {
            let mut builder = tar::Builder::new(&mut tar_bytes);
            let payload = vec![b'Z'; 1024];
            let mut header = tar::Header::new_gnu();
            header.set_size(payload.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append_data(&mut header, "cut.bin", &payload[..])
                .unwrap();
            builder.finish().unwrap();
        }
        let truncated_tar = &tar_bytes[..700]; // 512 header + partial 188 bytes of 1024 payload
        let trunc_src = dir.path().join("trunc.tar.gz");
        let trunc_dst = dir.path().join("trunc.out");
        write_gzip_bytes(&trunc_src, truncated_tar);
        decompress_file(
            trunc_src.to_string_lossy().to_string(),
            trunc_dst.to_string_lossy().to_string(),
        )
        .unwrap();
        assert!(trunc_dst.is_file());
        assert_eq!(fs::read(&trunc_dst).unwrap(), truncated_tar);
    }

    #[test]
    fn test_decompress_src_equals_dst_rejected() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("same.gz");
        write_gzip_bytes(&path, b"important");

        let p = path.to_string_lossy().to_string();
        assert!(decompress_file(p.clone(), p).is_err());

        // Verify original gzip file is still intact
        let out = dir.path().join("verified.out");
        decompress_file(
            path.to_string_lossy().to_string(),
            out.to_string_lossy().to_string(),
        )
        .unwrap();
        assert_eq!(fs::read_to_string(&out).unwrap(), "important");
    }

    #[test]
    fn test_decompress_non_tar_into_existing_dir_errors() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("plain.gz");
        let dst = dir.path().join("existing_dir");
        fs::create_dir_all(&dst).unwrap();
        fs::write(dst.join("keep.txt"), "safe").unwrap();
        write_gzip_bytes(&src, b"plain data");

        assert!(
            decompress_file(
                src.to_string_lossy().to_string(),
                dst.to_string_lossy().to_string(),
            )
            .is_err()
        );
        assert_eq!(fs::read_to_string(dst.join("keep.txt")).unwrap(), "safe");
    }

    #[test]
    fn test_decompress_tar_path_traversal_does_not_escape_dst() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("traversal.tar.gz");
        let dst = dir.path().join("sandbox");

        // Manually build a raw tar header with a `../escaped.txt` path
        let mut tar_bytes = Vec::new();
        let mut header = tar::Header::new_gnu();
        let payload = b"evil";
        header.set_size(payload.len() as u64);
        header.set_mode(0o644);
        header.set_entry_type(tar::EntryType::Regular);
        let name = b"../escaped.txt";
        header.as_gnu_mut().unwrap().name[..name.len()].copy_from_slice(name);
        header.set_cksum();
        tar_bytes.extend_from_slice(header.as_bytes());
        tar_bytes.extend_from_slice(payload);
        tar_bytes.resize(512 + 512 + 1024, 0);

        write_gzip_bytes(&src, &tar_bytes);
        decompress_file(
            src.to_string_lossy().to_string(),
            dst.to_string_lossy().to_string(),
        )
        .unwrap();

        assert!(!dir.path().join("escaped.txt").exists());
    }

    #[cfg(unix)]
    #[test]
    fn test_decompress_tar_gz_replaces_symlinks_at_dst() {
        use std::os::unix::fs::symlink;

        let dir = tempdir().unwrap();
        let src = dir.path().join("archive.tar.gz");
        create_tar_gz(&src, &[("ok.txt", b"extracted")]);

        // 1. Broken symlink at dst
        let broken_dst = dir.path().join("broken_link");
        symlink(dir.path().join("nonexistent_target"), &broken_dst).unwrap();
        decompress_file(
            src.to_string_lossy().to_string(),
            broken_dst.to_string_lossy().to_string(),
        )
        .unwrap();
        assert!(
            !broken_dst
                .symlink_metadata()
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(
            fs::read_to_string(broken_dst.join("ok.txt")).unwrap(),
            "extracted"
        );

        // 2. Symlink to another directory at dst should be replaced, not followed
        let other_dir = dir.path().join("other_dir");
        fs::create_dir_all(&other_dir).unwrap();
        let dir_link = dir.path().join("dir_link");
        symlink(&other_dir, &dir_link).unwrap();
        decompress_file(
            src.to_string_lossy().to_string(),
            dir_link.to_string_lossy().to_string(),
        )
        .unwrap();
        assert!(
            !dir_link
                .symlink_metadata()
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(
            fs::read_to_string(dir_link.join("ok.txt")).unwrap(),
            "extracted"
        );
        assert!(!other_dir.join("ok.txt").exists());
    }
}
