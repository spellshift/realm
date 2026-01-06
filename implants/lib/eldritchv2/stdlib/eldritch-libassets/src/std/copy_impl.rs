use crate::RustEmbed;
use alloc::string::String;
use alloc::sync::Arc;
use eldritch_agent::Agent;
use std::io::Write;

use super::read_binary_impl;

pub fn copy<A: RustEmbed>(
    agent: Arc<dyn Agent>,
    remote_assets: &[String],
    src: String,
    dest: String,
) -> Result<(), String> {
    let bytes = read_binary_impl::read_binary::<A>(agent, remote_assets, src)?;
    let mut file = std::fs::File::create(dest).map_err(|e| e.to_string())?;
    file.write_all(&bytes).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_success() {
        use read_binary_impl::tests::{MockAgent, TestAsset};
        let agent = Arc::new(MockAgent::new());
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_path = temp_dir.path().join("copied_main.eldritch");
        let dest_str = dest_path.to_str().unwrap().to_string();
        let result = copy::<TestAsset>(
            agent,
            &Vec::new(),
            "print/main.eldritch".to_string(),
            dest_str.clone(),
        );
        assert!(result.is_ok());
        let content = std::fs::read_to_string(dest_path).unwrap();
        assert_eq!(content.trim(), "print(\"This script just prints\")");
    }

    #[test]
    fn test_copy_fail_read() {
        use read_binary_impl::tests::{MockAgent, TestAsset};
        let agent = Arc::new(MockAgent::new());
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_path = temp_dir.path().join("should_not_exist");
        let result = copy::<TestAsset>(
            agent,
            &Vec::new(),
            "nonexistent".to_string(),
            dest_path.to_str().unwrap().to_string(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_copy_fail_write() {
        use read_binary_impl::tests::{MockAgent, TestAsset};
        let agent = Arc::new(MockAgent::new());
        let temp_dir = tempfile::tempdir().unwrap();
        let _dest_str = temp_dir.path().to_str().unwrap().to_string();
        let invalid_dest = temp_dir
            .path()
            .join("nonexistent_dir")
            .join("file.txt")
            .to_str()
            .unwrap()
            .to_string();
        let result = copy::<TestAsset>(
            agent,
            &Vec::new(),
            "print/main.eldritch".to_string(),
            invalid_dest,
        );
        assert!(result.is_err());
    }
}
