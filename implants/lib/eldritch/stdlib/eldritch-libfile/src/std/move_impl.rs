use ::std::fs;
use alloc::format;
use alloc::string::String;

pub fn move_(src: String, dst: String) -> Result<(), String> {
    let src_paths = crate::std::glob_util::resolve_paths(&src, false)?;
    let is_dst_dir = std::path::Path::new(&dst).is_dir();

    if src_paths.len() > 1 && !is_dst_dir {
        return Err(format!(
            "Destination {dst} must be a directory when moving multiple files"
        ));
    }

    for p in src_paths {
        let target = if is_dst_dir {
            std::path::Path::new(&dst).join(p.file_name().unwrap_or_default())
        } else {
            std::path::PathBuf::from(&dst)
        };
        fs::rename(&p, &target).map_err(|e| {
            format!(
                "Failed to move {} to {}: {e}",
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
    fn test_move() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let src = tmp_dir.path().join("src.txt");
        let dst = tmp_dir.path().join("dst.txt");

        fs::write(&src, "move me").unwrap();

        move_(
            src.to_string_lossy().to_string(),
            dst.to_string_lossy().to_string(),
        )
        .unwrap();

        assert!(!src.exists());
        assert!(dst.exists());
        assert_eq!(fs::read_to_string(dst).unwrap(), "move me");
    }
}
