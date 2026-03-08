use ::std::fs;
use alloc::format;
use alloc::string::String;

pub fn copy(src: String, dst: String) -> Result<(), String> {
    let src_paths = crate::std::glob_util::resolve_paths(&src)?;

    // If copying multiple files, dst should probably be a directory
    // But for simplicity, if there's multiple, let's copy to dst joined with filename
    // if dst is a dir, or fail if dst is not a dir and multiple files matched.
    // If single file, behave as before.
    let is_dst_dir = std::path::Path::new(&dst).is_dir();

    if src_paths.len() > 1 && !is_dst_dir {
        return Err(format!(
            "Destination {dst} must be a directory when copying multiple files"
        ));
    }

    for p in src_paths {
        let target = if is_dst_dir {
            std::path::Path::new(&dst).join(p.file_name().unwrap_or_default())
        } else {
            std::path::PathBuf::from(&dst)
        };
        fs::copy(&p, &target).map_err(|e| {
            format!(
                "Failed to copy {} to {}: {e}",
                p.display(),
                target.display()
            )
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_copy() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let src = tmp_dir.path().join("src.txt");
        let dst = tmp_dir.path().join("dst.txt");

        fs::write(&src, "copy me").unwrap();

        copy(
            src.to_string_lossy().to_string(),
            dst.to_string_lossy().to_string(),
        )
        .unwrap();

        assert!(src.exists());
        assert!(dst.exists());
        assert_eq!(fs::read_to_string(dst).unwrap(), "copy me");
    }
}
