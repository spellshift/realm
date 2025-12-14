use super::super::agent::ImixAgent;
use super::super::task::TaskRegistry;
use eldritchv2::Agent;
use pb::c2;
use pb::config::Config;
use std::sync::Arc;
use transport::{MockTransport, SyncTransport};

#[tokio::test]
async fn test_imix_agent_buffer_and_flush() {
    let mut transport = MockTransport::default();

    // We expect report_task_output to be called exactly once
    transport
        .expect_report_task_output()
        .times(1)
        .returning(|_| Ok(c2::ReportTaskOutputResponse {}));

    transport.expect_is_active().returning(|| true);

    let handle = tokio::runtime::Handle::current();
    let registry = Arc::new(TaskRegistry::new());
    let agent = ImixAgent::new(Config::default(), transport, handle, registry);

    // 1. Report output (should buffer)
    let req = c2::ReportTaskOutputRequest {
        output: Some(c2::TaskOutput {
            id: 1,
            output: "test".to_string(),
            ..Default::default()
        }),
    };
    agent.buffer_task_output(req).unwrap();

    // Verify buffer
    {
        let buffer = agent.output_buffer.lock().unwrap();
        assert_eq!(buffer.len(), 1);
    }

    // 2. Flush outputs (should drain buffer and call transport)
    agent.flush_outputs().await;

    // Verify buffer empty
    {
        let buffer = agent.output_buffer.lock().unwrap();
        assert!(buffer.is_empty());
    }
}

#[tokio::test]
async fn test_imix_agent_fetch_asset() {
    let mut transport = MockTransport::default();

    transport.expect_is_active().returning(|| true);
    transport.expect_clone().returning(|| {
        let mut t = MockTransport::default();
        t.expect_is_active().returning(|| true);

        t.expect_fetch_asset().times(1).returning(|req, tx| {
            assert_eq!(req.name, "test_file");
            let chunk1 = c2::FetchAssetResponse {
                chunk: vec![1, 2, 3],
            };
            let chunk2 = c2::FetchAssetResponse { chunk: vec![4, 5] };
            let _ = tx.send(chunk1);
            let _ = tx.send(chunk2);
            Ok(())
        });
        t
    });

    let handle = tokio::runtime::Handle::current();
    let registry = Arc::new(TaskRegistry::new());
    let agent = ImixAgent::new(Config::default(), transport, handle, registry);

    let req = c2::FetchAssetRequest {
        name: "test_file".to_string(),
    };

    let agent_clone = Arc::new(agent); // Wrap in Arc for get_sync_transport
    let st = agent_clone.get_sync_transport();

    let result = std::thread::spawn(move || st.fetch_asset(req))
        .join()
        .unwrap();

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![1, 2, 3, 4, 5]);
}

#[tokio::test]
async fn test_imix_agent_report_credential() {
    let mut transport = MockTransport::default();
    transport.expect_is_active().returning(|| true);

    transport.expect_clone().returning(|| {
        let mut t = MockTransport::default();
        t.expect_is_active().returning(|| true);
        t.expect_report_credential()
            .times(1)
            .returning(|_| Ok(c2::ReportCredentialResponse {}));
        t
    });

    let handle = tokio::runtime::Handle::current();
    let registry = Arc::new(TaskRegistry::new());
    let agent = ImixAgent::new(Config::default(), transport, handle, registry);

    let agent_clone = Arc::new(agent);
    let st = agent_clone.get_sync_transport();

    std::thread::spawn(move || {
        let _ = st.report_credential(c2::ReportCredentialRequest {
            task_id: 1,
            credential: None,
        });
    })
    .join()
    .unwrap();
}

#[tokio::test]
async fn test_imix_agent_report_process_list() {
    let mut transport = MockTransport::default();
    transport.expect_is_active().returning(|| true);

    transport.expect_clone().returning(|| {
        let mut t = MockTransport::default();
        t.expect_is_active().returning(|| true);
        t.expect_report_process_list()
            .times(1)
            .returning(|_| Ok(c2::ReportProcessListResponse {}));
        t
    });

    let handle = tokio::runtime::Handle::current();
    let registry = Arc::new(TaskRegistry::new());
    let agent = ImixAgent::new(Config::default(), transport, handle, registry);

    let agent_clone = Arc::new(agent);
    let st = agent_clone.get_sync_transport();
    std::thread::spawn(move || {
        let _ = st.report_process_list(c2::ReportProcessListRequest {
            task_id: 1,
            list: None,
        });
    })
    .join()
    .unwrap();
}

#[tokio::test]
async fn test_imix_agent_claim_tasks() {
    let mut transport = MockTransport::default();
    transport.expect_is_active().returning(|| true);

    // Expect on main transport because claim_tasks uses it directly
    transport
        .expect_claim_tasks()
        .times(1)
        .returning(|_| Ok(c2::ClaimTasksResponse { tasks: vec![] }));

    let handle = tokio::runtime::Handle::current();
    let registry = Arc::new(TaskRegistry::new());

    // Provide config with beacon info
    let config = Config::default();
    let agent = ImixAgent::new(config, transport, handle, registry);

    let agent_clone = agent.clone();
    let _ = agent_clone.claim_tasks().await.unwrap();
}

#[tokio::test]
async fn test_imix_agent_report_file() {
    let mut transport = MockTransport::default();
    transport.expect_is_active().returning(|| true);

    transport.expect_clone().returning(|| {
        let mut t = MockTransport::default();
        t.expect_is_active().returning(|| true);
        t.expect_report_file()
            .times(1)
            .returning(|_| Ok(c2::ReportFileResponse {}));
        t
    });

    let handle = tokio::runtime::Handle::current();
    let registry = Arc::new(TaskRegistry::new());
    let agent = ImixAgent::new(Config::default(), transport, handle, registry);

    let agent_clone = Arc::new(agent);
    let st = agent_clone.get_sync_transport();
    std::thread::spawn(move || {
        let _ = st.report_file(c2::ReportFileRequest {
            task_id: 1,
            chunk: None,
        });
    })
    .join()
    .unwrap();
}

#[tokio::test]
#[allow(clippy::field_reassign_with_default)]
async fn test_imix_agent_config_access() {
    let mut config = Config::default();
    config.callback_uri = "http://localhost:8080".to_string();
    config.info = Some(pb::c2::Beacon {
        identifier: "agent1".to_string(),
        ..Default::default()
    });

    let mut transport = MockTransport::default();
    transport.expect_is_active().returning(|| true);

    let handle = tokio::runtime::Handle::current();
    let registry = Arc::new(TaskRegistry::new());
    let agent = ImixAgent::new(config, transport, handle, registry);

    // Run in thread for block_on
    let agent_clone = Arc::new(agent);
    let result = std::thread::spawn(move || agent_clone.get_config())
        .join()
        .unwrap();

    assert!(result.is_ok());
    let map = result.unwrap();
    assert_eq!(map.get("callback_uri").unwrap(), "http://localhost:8080");
    assert_eq!(map.get("beacon_id").unwrap(), "agent1");
}

#[tokio::test]
async fn test_imix_agent_transport_management() {
    let mut transport = MockTransport::default();
    transport.expect_name().returning(|| "mock_proto");
    transport
        .expect_list_available()
        .returning(|| vec!["mock_proto".to_string(), "http".to_string()]);
    transport.expect_is_active().returning(|| true);

    let handle = tokio::runtime::Handle::current();
    let registry = Arc::new(TaskRegistry::new());
    let agent = ImixAgent::new(Config::default(), transport, handle, registry);

    let agent_clone = Arc::new(agent);

    let ac1 = agent_clone.clone();
    let name = std::thread::spawn(move || ac1.get_transport())
        .join()
        .unwrap()
        .unwrap();
    assert_eq!(name, "mock_proto");

    let ac2 = agent_clone.clone();
    let list = std::thread::spawn(move || ac2.list_transports())
        .join()
        .unwrap()
        .unwrap();
    assert!(list.contains(&"mock_proto".to_string()));
}
