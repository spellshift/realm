use super::super::agent::ImixAgent;
use super::super::task::TaskRegistry;
use pb::config::Config;
use std::sync::Arc;
use transport::MockTransport;

#[allow(clippy::field_reassign_with_default)]
#[tokio::test]
async fn test_imix_agent_get_callback_interval_error() {
    let mut config = Config::default();
    config.info = None; // Ensure no beacon info to trigger error

    let transport = MockTransport::default();
    let handle = tokio::runtime::Handle::current();
    let registry = Arc::new(TaskRegistry::new());
    let agent = ImixAgent::new(config, transport, handle, registry);

    let result = agent.get_callback_interval_u64();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No beacon info"));
}

#[allow(clippy::field_reassign_with_default)]
#[tokio::test]
async fn test_imix_agent_get_callback_interval_success() {
    let mut config = Config::default();
    config.info = Some(pb::c2::Beacon {
        active_transport: Some(pb::c2::ActiveTransport {
            uri: "http://example.com/callback".to_string(),
            interval: 10,
            ..Default::default()
        }),
        ..Default::default()
    });

    let transport = MockTransport::default();
    let handle = tokio::runtime::Handle::current();
    let registry = Arc::new(TaskRegistry::new());
    let agent = ImixAgent::new(config, transport, handle, registry);

    let result = agent.get_callback_interval_u64();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 10);
}
