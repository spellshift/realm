use crate::std::StdAssetsLibrary;
use alloc::string::String;

impl StdAssetsLibrary {
    pub fn read_impl(&self, name: String) -> Result<String, String> {
        let bytes = self.read_binary_impl(&name).map_err(|e| e.to_string())?;
        String::from_utf8(bytes).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::std::read_binary_impl::tests::{MockAgent, TestAsset};
    use crate::std::{AgentAssets, AssetsLibrary, EmbeddedAssets};
    use pb::c2::TaskContext;
    use std::sync::Arc;

    #[test]
    fn test_read_embedded_success() -> anyhow::Result<()> {
        let agent = Arc::new(MockAgent::new());
        let mut lib = StdAssetsLibrary::new();
        lib.add(Arc::new(AgentAssets::new(
            agent,
            TaskContext {
                task_id: 0,
                jwt: String::new(),
            },
            vec!["remote_file.txt".to_string()],
        )))?;
        lib.add(Arc::new(EmbeddedAssets::<TestAsset>::new()))?;
        let content = lib.read("print/main.eldritch".to_string());
        assert!(content.is_ok());
        assert_eq!(
            content.unwrap().trim(),
            "print(\"This script just prints\")"
        );
        Ok(())
    }
}
