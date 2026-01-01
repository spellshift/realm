use std::io::Write;
use tempfile::NamedTempFile;

/// Creates a test file with realistic data pattern (not all zeros)
/// This ensures encryption and compression behave realistically
pub fn create_test_file(size: usize) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");

    // Use a repeating pattern (0..255) to simulate realistic data
    // Not completely random (which would be incompressible) nor all zeros
    let pattern: Vec<u8> = (0..256).map(|i| i as u8).collect();
    let mut written = 0;

    while written < size {
        let to_write = std::cmp::min(pattern.len(), size - written);
        file.write_all(&pattern[..to_write])
            .expect("Failed to write to temp file");
        written += to_write;
    }

    file.flush().expect("Failed to flush temp file");
    file
}

/// Human-readable file size formatting for benchmark names
pub fn human_readable(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = 1024 * KB;

    if bytes >= MB {
        format!("{}MB", bytes / MB)
    } else if bytes >= KB {
        format!("{}KB", bytes / KB)
    } else {
        format!("{}B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_create_test_file() {
        let temp_file = create_test_file(1024);
        let metadata = std::fs::metadata(temp_file.path()).unwrap();
        assert_eq!(metadata.len(), 1024);
    }

    #[test]
    fn test_file_has_pattern() {
        let temp_file = create_test_file(512);
        let mut contents = Vec::new();
        std::fs::File::open(temp_file.path())
            .unwrap()
            .read_to_end(&mut contents)
            .unwrap();

        // Should contain the repeating 0..255 pattern
        assert_eq!(contents.len(), 512);
        assert_eq!(contents[0], 0);
        assert_eq!(contents[255], 255);
        assert_eq!(contents[256], 0); // Pattern repeats
    }

    #[test]
    fn test_human_readable() {
        assert_eq!(human_readable(512), "512B");
        assert_eq!(human_readable(1024), "1KB");
        assert_eq!(human_readable(2048), "2KB");
        assert_eq!(human_readable(1024 * 1024), "1MB");
        assert_eq!(human_readable(5 * 1024 * 1024), "5MB");
    }
}
