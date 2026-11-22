use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::Guardrail;

#[derive(Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct File {
    pub path: String,
}

impl File {
    pub fn new(path: &str) -> Self {
        File {
            path: path.to_string(),
        }
    }
}

impl Guardrail for File {
    fn get_name(&self) -> String {
        "file".to_string()
    }

    fn check(&self) -> bool {
        if self.path.is_empty() {
            return false;
        }

        // Preserve original casing so host OS filesystem semantics apply directly
        // (case-insensitive on default Windows/macOS, case-sensitive on Linux and
        // Windows per-directory case-sensitive folders).
        Path::new(&self.path).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_file_guardrail_exists() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("MixedCase_Test_File.txt");
        fs::write(&file_path, "test").unwrap();

        let guardrail = File::new(file_path.to_str().unwrap());
        assert!(guardrail.check());
    }

    #[test]
    fn test_file_guardrail_not_exists() {
        let guardrail = File::new("/path/that/does/not/exist");
        assert!(!guardrail.check());
    }

    #[test]
    fn test_file_guardrail_empty_path_fails_closed() {
        let guardrail = File::new("");
        assert!(!guardrail.check());
    }
}
