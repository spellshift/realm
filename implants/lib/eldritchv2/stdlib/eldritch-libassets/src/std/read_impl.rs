use crate::RustEmbed;
use alloc::string::String;
use alloc::sync::Arc;
use eldritch_agent::Agent;

use super::read_binary_impl;

pub fn read<A: RustEmbed>(
    agent: Arc<dyn Agent>,
    jwt: String,
    remote_assets: &[String],
    name: String,
) -> Result<String, String> {
    let bytes = read_binary_impl::read_binary::<A>(agent, jwt, remote_assets, name)?;
    String::from_utf8(bytes).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_embedded_success() {
        use read_binary_impl::tests::{MockAgent, TestAsset};
        let agent = Arc::new(MockAgent::new());
        let content = read::<TestAsset>(
            agent,
            "a jwt".to_string(),
            &Vec::new(),
            "print/main.eldritch".to_string(),
        );
        assert!(content.is_ok());
        assert_eq!(
            content.unwrap().trim(),
            "print(\"This script just prints\")"
        );
    }
}
