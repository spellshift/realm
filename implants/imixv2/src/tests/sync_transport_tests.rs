use super::super::agent::ImixAgent;
use super::super::task::TaskRegistry;
use pb::config::Config;
use std::sync::Arc;
use transport::{MockTransport, SyncTransport};

#[tokio::test]
async fn test_imix_sync_transport_reverse_shell_recovery() {
    // 1. Initialize agent with MockTransport (default/empty state)
    let handle = tokio::runtime::Handle::current();
    let registry = Arc::new(TaskRegistry::new());

    let mut config = Config::default();
    config.callback_uri = "http://localhost:8080".to_string();

    let mut initial_transport = MockTransport::default();
    initial_transport.expect_is_active().returning(|| false);
    initial_transport.expect_clone().returning(|| {
        let mut t = MockTransport::default();
        t.expect_is_active().returning(|| false);
        t
    });

    // 2. Setup expectation for creating a NEW transport
    let ctx = MockTransport::new_context();
    ctx.expect()
        .with(mockall::predicate::eq("http://localhost:8080".to_string()), mockall::predicate::eq(None))
        .returning(|_, _| {
             let mut t = MockTransport::default();
             t.expect_reverse_shell()
                .times(1)
                .returning(|_, _| Ok(()));
             Ok(t)
        });

    let agent = ImixAgent::new(config, initial_transport, handle, registry);
    let agent = Arc::new(agent);

    // 3. Get SyncTransport
    let st = agent.get_sync_transport();

    // 4. Run reverse_shell
    let (_tx_req, rx_req) = std::sync::mpsc::channel();
    let (tx_resp, _rx_resp) = std::sync::mpsc::channel();

    let result = std::thread::spawn(move || {
        st.reverse_shell(rx_req, tx_resp)
    }).join().unwrap();

    assert!(result.is_ok(), "Reverse shell failed: {:?}", result.err());
}
