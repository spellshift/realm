use std::io::Write;

use crate::std::StdAssetsLibrary;
use alloc::string::String;

impl StdAssetsLibrary {
    pub fn copy_impl(&self, src: String, dest: String) -> Result<(), String> {
        let bytes = self.read_binary_impl(&src).map_err(|e| e.to_string())?;
        let mut file = std::fs::File::create(dest).map_err(|e| e.to_string())?;
        file.write_all(&bytes).map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::std::read_binary_impl::tests::{MockAgent, TestAsset};
    use crate::std::{AgentAssets, AssetsLibrary, EmbeddedAssets};
    use std::sync::Arc;

    #[test]
    fn test_copy_success() -> anyhow::Result<()> {
        let agent = Arc::new(MockAgent::new());
        let mut lib = StdAssetsLibrary::new();
        lib.add(Arc::new(AgentAssets::new(agent, String::new(), Vec::new())))?;
        lib.add(Arc::new(EmbeddedAssets::<TestAsset>::new()))?;
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_path = temp_dir.path().join("copied_main.eldritch");
        let dest_str = dest_path.to_str().unwrap().to_string();
        let result = lib.copy("print/main.eldritch".to_string(), dest_str.clone());
        assert!(result.is_ok());
        let content = std::fs::read_to_string(dest_path).unwrap();
        assert_eq!(content.trim(), "print(\"This script just prints\")");
        Ok(())
    }

    #[test]
    fn test_copy_fail_read() -> anyhow::Result<()> {
        let agent = Arc::new(MockAgent::new());
        let mut lib = StdAssetsLibrary::new();
        lib.add(Arc::new(AgentAssets::new(agent, String::new(), Vec::new())))?;
        lib.add(Arc::new(EmbeddedAssets::<TestAsset>::new()))?;
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_path = temp_dir.path().join("should_not_exist");
        let result = lib.copy(
            "nonexistent".to_string(),
            dest_path.to_str().unwrap().to_string(),
        );
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_copy_fail_write() -> anyhow::Result<()> {
        let agent = Arc::new(MockAgent::new());
        let mut lib = StdAssetsLibrary::new();
        lib.add(Arc::new(AgentAssets::new(agent, String::new(), Vec::new())))?;
        lib.add(Arc::new(EmbeddedAssets::<TestAsset>::new()))?;
        let temp_dir = tempfile::tempdir().unwrap();
        let _dest_str = temp_dir.path().to_str().unwrap().to_string();
        let invalid_dest = temp_dir
            .path()
            .join("nonexistent_dir")
            .join("file.txt")
            .to_str()
            .unwrap()
            .to_string();
        let result = lib.copy("print/main.eldritch".to_string(), invalid_dest);
        assert!(result.is_err());
        Ok(())
    }
}
