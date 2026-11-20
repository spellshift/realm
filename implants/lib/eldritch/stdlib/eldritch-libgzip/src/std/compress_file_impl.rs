use crate::std::{is_same_file, resolve_path};
use alloc::string::{String, ToString};
use anyhow::{Context, Result as AnyhowResult, anyhow};
use flate2::Compression;
use flate2::write::GzEncoder;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Write};
use std::path::Path;
use tar::{Builder, HeaderMode};

pub fn compress_file(src: String, dst: String) -> Result<(), String> {
    compress_impl(src, dst).map_err(|e| e.to_string())
}

fn compress_impl(src: String, dst: String) -> AnyhowResult<()> {
    let src_path = Path::new(&src);
    let dst_path = Path::new(&dst);

    let canonical_src = fs::canonicalize(src_path)?;
    if is_same_file(src_path, dst_path)? {
        return Err(anyhow!(
            "Source and destination refer to the same file: {src}"
        ));
    }

    let dst_existed = dst_path.symlink_metadata().is_ok();

    if canonical_src.is_dir() {
        let resolved_dst = resolve_path(dst_path)?;
        if resolved_dst.starts_with(&canonical_src) {
            return Err(anyhow!(
                "Destination path cannot be inside source directory: {dst}"
            ));
        }

        let src_name = canonical_src
            .file_name()
            .context("Failed to get directory name for source path")?;

        let res = (|| -> AnyhowResult<()> {
            let f_dst = BufWriter::new(File::create(dst_path)?);
            let deflater = GzEncoder::new(f_dst, Compression::fast());

            let mut tar_builder = Builder::new(deflater);
            tar_builder.mode(HeaderMode::Deterministic);
            tar_builder.follow_symlinks(false);
            tar_builder.append_dir_all(src_name, &canonical_src)?;

            let deflater = tar_builder.into_inner()?;
            deflater.finish()?.flush()?;
            Ok(())
        })();

        if let Err(err) = res {
            if !dst_existed {
                let _ = fs::remove_file(dst_path);
            }
            return Err(err);
        }
        return Ok(());
    }

    let mut reader = BufReader::new(File::open(&canonical_src)?);
    let res = (|| -> AnyhowResult<()> {
        let f_dst = BufWriter::new(File::create(dst_path)?);
        let mut deflater = GzEncoder::new(f_dst, Compression::fast());

        io::copy(&mut reader, &mut deflater)?;
        deflater.finish()?.flush()?;
        Ok(())
    })();

    if let Err(err) = res {
        if !dst_existed {
            let _ = fs::remove_file(dst_path);
        }
        return Err(err);
    }
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
    fn test_compress_empty_dir_and_relative_dot_path() {
        let dir = tempdir().unwrap();
        let empty_dir = dir.path().join("empty_dir");
        fs::create_dir_all(&empty_dir).unwrap();

        // Use a path ending in `/..` and `/.` to verify canonicalization resolves file_name
        let dot_path = empty_dir.join(".");
        let archive = dir.path().join("empty.tar.gz");
        compress_file(
            dot_path.to_string_lossy().to_string(),
            archive.to_string_lossy().to_string(),
        )
        .unwrap();

        let extracted = dir.path().join("extracted");
        decompress_file(
            archive.to_string_lossy().to_string(),
            extracted.to_string_lossy().to_string(),
        )
        .unwrap();
        assert!(extracted.join("empty_dir").is_dir());
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

    #[test]
    fn test_compress_src_equals_dst_rejected() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("data.txt");
        fs::write(&file, "do not destroy").unwrap();

        let path = file.to_string_lossy().to_string();
        assert!(compress_file(path.clone(), path).is_err());
        assert_eq!(fs::read_to_string(&file).unwrap(), "do not destroy");

        #[cfg(unix)]
        {
            let hardlink = dir.path().join("hardlink.txt");
            fs::hard_link(&file, &hardlink).unwrap();
            assert!(
                compress_file(
                    file.to_string_lossy().to_string(),
                    hardlink.to_string_lossy().to_string(),
                )
                .is_err()
            );
            assert_eq!(fs::read_to_string(&file).unwrap(), "do not destroy");
        }
    }

    #[test]
    fn test_compress_dst_inside_src_dir_rejected() {
        let dir = tempdir().unwrap();
        let src_dir = dir.path().join("my_dir");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("a.txt"), "hello").unwrap();

        let dst_inside = src_dir.join("out.tar.gz");
        assert!(
            compress_file(
                src_dir.to_string_lossy().to_string(),
                dst_inside.to_string_lossy().to_string(),
            )
            .is_err()
        );
        assert!(!dst_inside.exists());
    }

    #[test]
    fn test_compress_nonexistent_dst_parent_errors() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("a.txt");
        fs::write(&src, "hello").unwrap();
        let dst = dir.path().join("no_such_parent/out.gz");

        assert!(
            compress_file(
                src.to_string_lossy().to_string(),
                dst.to_string_lossy().to_string(),
            )
            .is_err()
        );
    }

    #[cfg(unix)]
    #[test]
    fn test_compress_dir_preserves_symlinks_and_handles_cycles() {
        use std::os::unix::fs::symlink;

        let dir = tempdir().unwrap();
        let src_dir = dir.path().join("sym_dir");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("real.txt"), "real").unwrap();
        symlink(src_dir.join("missing_target"), src_dir.join("broken_link")).unwrap();
        symlink(&src_dir, src_dir.join("self_loop")).unwrap();

        let archive = dir.path().join("sym.tar.gz");
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
        assert_eq!(
            fs::read_to_string(extracted.join("sym_dir/real.txt")).unwrap(),
            "real"
        );
        assert!(
            extracted
                .join("sym_dir/broken_link")
                .symlink_metadata()
                .unwrap()
                .file_type()
                .is_symlink()
        );
    }
}
