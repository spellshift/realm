use crate::std::StdAssetsLibrary;
use alloc::vec::Vec;
use anyhow::Result;

impl StdAssetsLibrary {
    pub fn read_binary_impl(&self, name: &str) -> Result<Vec<u8>> {
        // Iterate through the boxed trait objects (maintaining precedence order)
        for backend in &self.backends {
            if let Ok(file) = backend.get(name) {
                // Return immediately upon the first match
                return Ok(file);
            }
        }
        Err(anyhow::anyhow!("asset not found: {}", name))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::std::{AgentAssets, AssetsLibrary, EmbeddedAssets};
    use alloc::string::String;
    use alloc::string::ToString;
    use eldritch_mockagent::MockAgent;
    use pb::c2::TaskContext;
    use std::sync::Arc;

    #[cfg(debug_assertions)]
    #[derive(rust_embed::Embed)]
    #[folder = "../../../../../bin/embedded_files_test"]
    pub struct TestAsset;

    #[test]
    fn test_read_binary_embedded_success() -> anyhow::Result<()> {
        let mut lib = StdAssetsLibrary::new();
        lib.add(Arc::new(EmbeddedAssets::<TestAsset>::new()))?;
        let content = lib.read_binary("print/main.eldritch".to_string());
        assert!(content.is_ok());
        let content = content.unwrap();
        let bytes = if let eldritch_core::Value::Bytes(b) = content {
            b
        } else {
            panic!("Expected Bytes")
        };
        assert!(!bytes.is_empty());
        assert_eq!(
            std::str::from_utf8(&bytes).unwrap().trim(),
            "print(\"This script just prints\")"
        );
        Ok(())
    }

    #[test]
    fn test_read_binary_embedded_fail() -> anyhow::Result<()> {
        let mut lib = StdAssetsLibrary::new();
        lib.add(Arc::new(EmbeddedAssets::<TestAsset>::new()))?;
        assert!(lib.read_binary("nonexistent_file".to_string()).is_err());
        Ok(())
    }

    #[test]
    fn test_read_binary_remote_success() -> anyhow::Result<()> {
        let agent = Arc::new(MockAgent::new().with_asset("remote_file.txt", b"remote content"));
        let mut lib = StdAssetsLibrary::new();
        lib.add(Arc::new(AgentAssets::new(
            agent,
            eldritch_agent::Context::Task(TaskContext {
                task_id: 0,
                jwt: String::new(),
            }),
            vec!["remote_file.txt".to_string()],
        )))?;
        let content = lib.read_binary("remote_file.txt".to_string());
        assert!(content.is_ok());
        let bytes = if let eldritch_core::Value::Bytes(b) = content.unwrap() {
            b
        } else {
            panic!("Expected Bytes")
        };
        assert_eq!(bytes, b"remote content");
        Ok(())
    }

    #[test]
    fn test_read_binary_remote_fail() -> anyhow::Result<()> {
        let agent = Arc::new(MockAgent::new().should_fail());
        let mut lib = StdAssetsLibrary::new();
        lib.add(Arc::new(AgentAssets::new(
            agent,
            eldritch_agent::Context::Task(TaskContext {
                task_id: 0,
                jwt: String::new(),
            }),
            vec!["remote_file.txt".to_string()],
        )))?;
        let result = lib.read_binary("remote_file.txt".to_string());
        assert!(result.is_err());
        Ok(())
    }
}
