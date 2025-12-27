use alloc::format;
use alloc::string::String;
use ::std::fs;

pub fn copy(src: String, dst: String) -> Result<(), String> {
    fs::copy(&src, &dst).map_err(|e| format!("Failed to copy {src} to {dst}: {e}"))?;
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
