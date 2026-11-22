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

fn decompress_raw_file(src_path: &Path, dst_path: &Path) -> AnyhowResult<()> {
    let f_src = BufReader::new(File::open(src_path)?);
    let mut decoder = MultiGzDecoder::new(f_src);
    let mut writer = BufWriter::new(File::create(dst_path)?);
    io::copy(&mut decoder, &mut writer)?;
    writer.flush()?;
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
        if let Err(err) = decompress_raw_file(src_path, dst_path) {
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
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_decompress_file() {
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

        decompress_file(src_path, dst_path.clone()).unwrap();

        let res = fs::read_to_string(&dst_path).unwrap();
        assert_eq!(res, content);
    }
}
