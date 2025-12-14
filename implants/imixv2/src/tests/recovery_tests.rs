use crate::agent::ImixAgent;
use crate::task::TaskRegistry;
use pb::config::Config;
use std::sync::Arc;
use tokio::runtime::Handle;
use transport::{MockTransport, Transport};
use anyhow::Result;
use std::sync::mpsc;
use pb::c2::{ReverseShellRequest, ReverseShellResponse};

#[test]
fn test_sync_transport_recovery_reverse_shell() -> Result<()> {
    // We need a runtime for the agent and async tasks
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let runtime_handle = rt.handle().clone();

    // Helper to create a transport that reports active=true
    let create_active_transport = || {
        let mut t = MockTransport::default();
        t.expect_clone().returning(move || {
            let mut t = MockTransport::default();
            t.expect_is_active().returning(|| true);
            t.expect_clone().returning(|| {
                 let mut t = MockTransport::default();
                 t.expect_is_active().returning(|| true);
                 t
            });
            t
        });
        t.expect_is_active().returning(|| true);
        t.expect_name().returning(|| "mock");
        t.expect_list_available().returning(|| vec!["mock".to_string()]);
        t
    };

    // 1. Setup MockTransport (Initial)
    let mock_transport = create_active_transport();

    // Setup static method mocks for the recovery transport
    let ctx = MockTransport::new_context();
    ctx.expect()
        .returning(|_, _| {
             let mut t = MockTransport::default();
             // The recovered transport must be cloneable and active
             t.expect_clone().returning(move || {
                let mut t = MockTransport::default();
                t.expect_is_active().returning(|| true);
                t.expect_reverse_shell().returning(|_, _| Ok(()));
                t
             });
             t.expect_is_active().returning(|| true);
             t.expect_name().returning(|| "mock");
             t.expect_list_available().returning(|| vec!["mock".to_string()]);

             // When recovered, reverse_shell should be called and succeed
             t.expect_reverse_shell().returning(|_, _| Ok(()));
             Ok(t)
        });

    // 2. Create Agent
    let mut config = Config::default();
    config.callback_uri = "mock://callback".to_string();
    config.info = Some(pb::c2::Beacon::default());

    let registry = Arc::new(TaskRegistry::new());

    let agent = ImixAgent::new(config, mock_transport, runtime_handle.clone(), registry);

    // 3. Get SyncTransport
    let sync = agent.get_sync_transport();

    // 4. Simulate Disconnect (update transport to inactive)
    let mut inactive_transport = MockTransport::default();
    // When cloned, it should return another inactive transport (because SyncTransportAdapter clones it to check)
    inactive_transport.expect_clone().returning(|| {
        let mut t = MockTransport::default();
        t.expect_is_active().returning(|| false);
        t
    });
    inactive_transport.expect_is_active().returning(|| false);

    // We need to call async methods on agent.
    rt.block_on(async {
        agent.update_transport(inactive_transport).await;
    });

    // 5. Try to use reverse_shell (which uses factory to recover)
    let (tx_req, rx_req) = mpsc::channel::<ReverseShellRequest>();
    let (tx_resp, rx_resp) = mpsc::channel::<ReverseShellResponse>();

    drop(tx_req);
    drop(rx_resp);

    // Call reverse_shell
    let res = sync.reverse_shell(rx_req, tx_resp);

    assert!(res.is_ok(), "SyncTransport failed to recover and run reverse_shell");

    Ok(())
}
