use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::Guardrail;

#[derive(Serialize, Deserialize)]
pub struct File {
    pub path: String,
}

impl Default for File {
    fn default() -> Self {
        File {
            path: "".to_string(),
        }
    }
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
        let file_path = dir.path().join("test_file.txt");
        fs::write(&file_path, "test").unwrap();

        let guardrail = File::new(file_path.to_str().unwrap());
        assert!(guardrail.check());
    }

    #[test]
    fn test_file_guardrail_not_exists() {
        let guardrail = File::new("/path/that/does/not/exist");
        assert!(!guardrail.check());
    }
}
