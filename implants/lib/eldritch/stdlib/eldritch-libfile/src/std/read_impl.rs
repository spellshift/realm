use ::std::fs;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use glob::glob;

pub fn impl_read_utf8_utf16(path: String) -> Result<String, String> {
    let bytes = fs::read(&path).map_err(|e| format!("Failed to read file {path}: {e}"))?;

    if bytes.starts_with(&[0xFF, 0xFE]) {
        let u16s: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();
        String::from_utf16(&u16s)
            .map_err(|e| format!("Failed to decode UTF-16 in file {path}: {e}"))
    } else {
        String::from_utf8(bytes).map_err(|e| format!("Failed to decode UTF-8 in file {path}: {e}"))
    }
}

pub fn read(path: String) -> Result<String, String> {
    if path.contains('*') || path.contains('?') || path.contains('[') {
        let paths: Vec<_> = glob(&path)
            .map_err(|e| format!("Invalid glob pattern {path}: {e}"))?
            .collect();
        if paths.is_empty() {
            return Err(format!("No files found matching pattern {path}"));
        }
        let mut result = String::new();
        for entry in paths {
            let matched_path = entry.map_err(|e| format!("Glob error: {e}"))?;
            let content = impl_read_utf8_utf16(matched_path.to_string_lossy().into_owned())?;
            result.push_str(&content);
        }
        Ok(result)
    } else {
        impl_read_utf8_utf16(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();

        fs::write(&path, "hello").unwrap();

        assert_eq!(read(path).unwrap(), "hello");
    }

    #[test]
    fn test_read_utf16le() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();

        // UTF-16LE for "hello\r\nworld" with BOM
        let bytes: [u8; 26] = [
            0xFF, 0xFE, // BOM
            0x68, 0x00, // h
            0x65, 0x00, // e
            0x6C, 0x00, // l
            0x6C, 0x00, // l
            0x6F, 0x00, // o
            0x0D, 0x00, // \r
            0x0A, 0x00, // \n
            0x77, 0x00, // w
            0x6F, 0x00, // o
            0x72, 0x00, // r
            0x6C, 0x00, // l
            0x64, 0x00, // d
        ];

        fs::write(&path, &bytes).unwrap();

        assert_eq!(read(path).unwrap(), "hello\r\nworld");
    }
}
