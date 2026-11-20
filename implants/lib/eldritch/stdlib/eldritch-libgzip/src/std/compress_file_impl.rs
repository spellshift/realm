use crate::std::{is_same_file, resolve_path};
use alloc::string::{String, ToString};
use anyhow::{Context, Result as AnyhowResult, anyhow};
use flate2::Compression;
use flate2::write::GzEncoder;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Write};
use std::path::Path;
use tar::{Builder, HeaderMode};

pub fn compress_file(src: String, dst: String) -> Result<(), String> {
    compress_impl(src, dst).map_err(|e| e.to_string())
}

fn compress_dir(src_name: &OsStr, src_dir: &Path, dst_path: &Path) -> AnyhowResult<()> {
    let f_dst = BufWriter::new(File::create(dst_path)?);
    let deflater = GzEncoder::new(f_dst, Compression::fast());

    let mut tar_builder = Builder::new(deflater);
    tar_builder.mode(HeaderMode::Deterministic);
    tar_builder.follow_symlinks(false);
    tar_builder.append_dir_all(src_name, src_dir)?;

    let deflater = tar_builder.into_inner()?;
    deflater.finish()?.flush()?;
    Ok(())
}

fn compress_single_file(src_file: &Path, dst_path: &Path) -> AnyhowResult<()> {
    let mut reader = BufReader::new(File::open(src_file)?);
    let f_dst = BufWriter::new(File::create(dst_path)?);
    let mut deflater = GzEncoder::new(f_dst, Compression::fast());

    io::copy(&mut reader, &mut deflater)?;
    deflater.finish()?.flush()?;
    Ok(())
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

    let res = if canonical_src.is_dir() {
        let resolved_dst = resolve_path(dst_path)?;
        if resolved_dst.starts_with(&canonical_src) {
            return Err(anyhow!(
                "Destination path cannot be inside source directory: {dst}"
            ));
        }

        let src_name = canonical_src
            .file_name()
            .context("Failed to get directory name for source path")?;
        compress_dir(src_name, &canonical_src, dst_path)
    } else {
        compress_single_file(&canonical_src, dst_path)
    };

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
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_compress_file() {
        let content = "Compression Test";
        let tmp_src = NamedTempFile::new().unwrap();
        let src_path = tmp_src.path().to_string_lossy().to_string();
        fs::write(&src_path, content).unwrap();

        let tmp_dst = NamedTempFile::new().unwrap();
        let dst_path = tmp_dst.path().to_string_lossy().to_string();

        compress_file(src_path.clone(), dst_path.clone()).unwrap();

        let meta = fs::metadata(&dst_path).unwrap();
        assert!(meta.len() > 0);
    }
}
