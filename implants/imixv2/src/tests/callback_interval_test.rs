use super::super::agent::ImixAgent;
use super::super::task::TaskRegistry;
use pb::config::Config;
use std::sync::Arc;
use transport::MockTransport;

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

#[tokio::test]
async fn test_imix_agent_get_callback_interval_success() {
    let mut config = Config::default();
    config.info = Some(pb::c2::Beacon {
        interval: 10,
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
