use super::super::agent::ImixAgent;
use super::super::task::TaskRegistry;
use eldritch_libagent::agent::Agent;
use pb::config::Config;
use std::sync::Arc;
use transport::{MockTransport, SyncTransport};
use std::sync::mpsc;

#[tokio::test]
async fn test_start_reverse_shell() {
    let _ = pretty_env_logger::try_init();

    // Setup mocks
    let mut transport = MockTransport::default();

    // Expect clone to be called, and return a mock that expects reverse_shell
    transport.expect_clone().returning(|| {
        let mut t = MockTransport::default();
        // Expect reverse_shell stream initiation
        t.expect_reverse_shell().times(1).returning(|_, _| Ok(()));
        t.expect_is_active().returning(|| true);
        t
    });

    // Expect is_active to be called by get_usable_transport
    transport.expect_is_active().returning(|| true);

    // Handle required for ImixAgent spawning
    let handle = tokio::runtime::Handle::current();

    let task_registry = Arc::new(TaskRegistry::new());
    let agent = Arc::new(ImixAgent::new(
        Config::default(),
        transport,
        handle,
        task_registry,
    ));

    // Execution must happen in a separate thread to allow block_on
    let agent_clone = agent.clone();

    // Get sync transport
    let sync_transport = agent_clone.get_sync_transport();

    // Create channels for the stream
    let (tx_req, rx_req) = mpsc::channel();
    let (tx_resp, rx_resp) = mpsc::channel();

    let result = std::thread::spawn(move || {
        sync_transport.reverse_shell(rx_req, tx_resp)
    })
    .join()
    .unwrap();

    assert!(result.is_ok(), "reverse_shell stream should succeed");

    // We can't easily verify the async task spawned by sync_transport.reverse_shell actually ran
    // without interacting with channels or waiting.
    // But since the mock expects 1 call and we didn't panic, it likely worked.
    // The test might finish before the async task runs if we don't wait?
    // `sync_transport.reverse_shell` spawns a task and returns immediately.
    // The mock expectation is on `transport.reverse_shell`.
    // We should wait a bit to allow the spawned task to call the transport.
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}
