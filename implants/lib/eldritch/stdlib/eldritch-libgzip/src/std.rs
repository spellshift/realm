use super::GzipLibrary;
use alloc::string::String;
use alloc::vec::Vec;
use anyhow::Result as AnyhowResult;
use eldritch_macros::eldritch_library_impl;
use std::fs;
use std::path::{Path, PathBuf};

pub mod compress_file_impl;
pub mod compress_impl;
pub mod decompress_file_impl;
pub mod decompress_impl;

/// Resolve a path to a canonical form even when the final component (or several
/// trailing components) does not exist yet.
pub(crate) fn resolve_path(path: &Path) -> AnyhowResult<PathBuf> {
    if let Ok(canonical) = fs::canonicalize(path) {
        return Ok(canonical);
    }

    let mut suffix_components = Vec::new();
    let mut current = path;

    while let Some(name) = current.file_name() {
        suffix_components.push(name.to_os_string());
        current = current.parent().unwrap_or_else(|| Path::new(""));
        let probe = if current.as_os_str().is_empty() {
            Path::new(".")
        } else {
            current
        };
        if let Ok(mut canonical) = fs::canonicalize(probe) {
            for component in suffix_components.into_iter().rev() {
                canonical.push(component);
            }
            return Ok(canonical);
        }
    }

    Ok(path.to_path_buf())
}

/// Returns `true` if `src` and `dst` resolve to the same path or underlying file
/// (including symlinks and Unix hardlinks).
pub(crate) fn is_same_file(src: &Path, dst: &Path) -> AnyhowResult<bool> {
    let canonical_src = fs::canonicalize(src)?;
    let resolved_dst = resolve_path(dst)?;
    if canonical_src == resolved_dst {
        return Ok(true);
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if let (Ok(src_meta), Ok(dst_meta)) = (fs::metadata(&canonical_src), fs::metadata(dst))
            && src_meta.dev() == dst_meta.dev()
            && src_meta.ino() == dst_meta.ino()
        {
            return Ok(true);
        }
    }

    Ok(false)
}

#[derive(Default, Debug)]
#[eldritch_library_impl(GzipLibrary)]
pub struct StdGzipLibrary;

impl GzipLibrary for StdGzipLibrary {
    fn compress(&self, data: Vec<u8>) -> Result<Vec<u8>, String> {
        compress_impl::compress_bytes(data)
    }

    fn compress_file(&self, src: String, dst: String) -> Result<(), String> {
        compress_file_impl::compress_file(src, dst)
    }

    fn decompress(&self, data: Vec<u8>) -> Result<Vec<u8>, String> {
        decompress_impl::decompress_bytes(data)
    }

    fn decompress_file(&self, src: String, dst: String) -> Result<(), String> {
        decompress_file_impl::decompress_file(src, dst)
    }
}
