use crate::std::StdAssetsLibrary;
use alloc::string::String;

impl StdAssetsLibrary {
    pub fn list_impl(&self) -> Result<Vec<String>, String> {
        Ok(self.asset_names.iter().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::std::read_binary_impl::tests::{MockAgent, TestAsset};
    use crate::std::{AgentAssets, AssetsLibrary, EmbeddedAssets};
    use std::sync::Arc;

    #[test]
    fn test_list() -> anyhow::Result<()> {
        let agent = Arc::new(MockAgent::new());
        let remote_files = vec!["remote1.txt".to_string(), "remote2.txt".to_string()];
        let mut lib = StdAssetsLibrary::new();
        lib.add(Arc::new(AgentAssets::new(
            agent,
            String::new(),
            remote_files.clone(),
        )))?;
        lib.add(Arc::new(EmbeddedAssets::<TestAsset>::new()))?;
        let list = lib.list().unwrap();
        assert!(list.iter().any(|f| f.contains("print/main.eldritch")));
        assert!(list.contains(&"remote1.txt".to_string()));
        assert!(list.contains(&"remote2.txt".to_string()));
        Ok(())
    }
}
