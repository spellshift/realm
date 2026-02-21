use super::super::agent::ImixAgent;
use super::super::task::TaskRegistry;
use eldritch::agent::agent::Agent;
use pb::c2::host::Platform;
use pb::c2::{self, Host};
use pb::config::Config;
use std::sync::Arc;
use transport::MockTransport;

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
        context: Some(c2::TaskContext {
            task_id: 1,
            jwt: "some jwt".to_string(),
        }),
        shell_task_output: None,
    };
    agent.report_task_output(req).unwrap();

    // 2. Flush outputs (should drain buffer and call transport)
    agent.flush_outputs().await;
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
        context: Some(c2::TaskContext {
            task_id: 0,
            jwt: "a jwt".to_string(),
        }),
    };

    let agent_clone = agent.clone();
    let result = std::thread::spawn(move || agent_clone.fetch_asset(req))
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

    let agent_clone = agent.clone();
    std::thread::spawn(move || {
        let _ = agent_clone.report_credential(c2::ReportCredentialRequest {
            credential: None,
            context: Some(c2::TaskContext {
                task_id: 1,
                jwt: "some jwt".to_string(),
            }),
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

    let agent_clone = agent.clone();
    std::thread::spawn(move || {
        let _ = agent_clone.report_process_list(c2::ReportProcessListRequest {
            list: None,
            context: Some(c2::TaskContext {
                task_id: 1,
                jwt: "some jwt".to_string(),
            }),
        });
    })
    .join()
    .unwrap();
}

#[tokio::test]
async fn test_imix_agent_claim_tasks() {
    let mut transport = MockTransport::default();
    transport.expect_is_active().returning(|| true);
    transport.expect_is_active().returning(|| true);
    transport.expect_claim_tasks().times(1).returning(|_| {
        Ok(c2::ClaimTasksResponse {
            tasks: vec![],
            shell_tasks: vec![],
        })
    });

    let handle = tokio::runtime::Handle::current();
    let registry = Arc::new(TaskRegistry::new());

    // Provide config with beacon info
    let config = Config::default();
    let agent = ImixAgent::new(config, transport, handle, registry);

    // let agent_clone = agent.clone();
    let _ = agent.claim_tasks().await.unwrap();
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

    let agent_clone = agent.clone();
    std::thread::spawn(move || {
        let _ = agent_clone.report_file(c2::ReportFileRequest {
            chunk: None,
            context: Some(c2::TaskContext {
                task_id: 1,
                jwt: "test jwt".to_string(),
            }),
        });
    })
    .join()
    .unwrap();
}

#[tokio::test]
#[allow(clippy::field_reassign_with_default)]
async fn test_imix_agent_config_access() {
    let mut config = Config::default();

    config.info = Some(pb::c2::Beacon {
        identifier: "agent1".to_string(),
        available_transports: Some(pb::c2::AvailableTransports {
            transports: vec![pb::c2::Transport {
                uri: "http://localhost:8080".to_string(),
                interval: 5,
                ..Default::default()
            }],
            active_index: 0,
        }),
        ..Default::default()
    });

    let mut transport = MockTransport::default();
    transport.expect_is_active().returning(|| true);

    let handle = tokio::runtime::Handle::current();
    let registry = Arc::new(TaskRegistry::new());
    let agent = ImixAgent::new(config, transport, handle, registry);

    // Run in thread for block_on
    let agent_clone = agent.clone();
    let result = std::thread::spawn(move || agent_clone.get_config())
        .join()
        .unwrap();

    assert!(result.is_ok());
    let map = result.unwrap();
    assert_eq!(map.get("callback_uri").unwrap(), "http://localhost:8080");
    assert_eq!(map.get("beacon_id").unwrap(), "agent1");
}

#[test]
fn test_agent_config_platform_as_enum_variant_name() {
    let config = Config {
        info: Some(pb::c2::Beacon {
            available_transports: Some(pb::c2::AvailableTransports {
                transports: vec![pb::c2::Transport {
                    uri: "http://localhost:8080".to_string(),
                    interval: 5,
                    ..Default::default()
                }],
                active_index: 0,
            }),
            host: Some(Host {
                platform: Platform::Linux as i32,
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut transport = MockTransport::default();
    transport.expect_is_active().returning(|| true);

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let agent = ImixAgent::new(
        config,
        transport,
        runtime.handle().clone(),
        Arc::new(TaskRegistry::new()),
    );

    let map = agent.get_config().unwrap();
    assert_eq!(map.get("platform").unwrap(), "PLATFORM_LINUX");
}

#[test]
fn test_agent_config_active_transport_type_as_enum_variant_name() {
    let config = Config {
        info: Some(pb::c2::Beacon {
            available_transports: Some(c2::AvailableTransports {
                transports: vec![pb::c2::Transport {
                    r#type: pb::c2::transport::Type::TransportGrpc as i32,
                    uri: "http://localhost:8000".to_string(),
                    interval: 5,
                    extra: "".to_string(),
                }],
                active_index: 0,
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut transport = MockTransport::default();
    transport.expect_is_active().returning(|| true);

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let agent = ImixAgent::new(
        config,
        transport,
        runtime.handle().clone(),
        Arc::new(TaskRegistry::new()),
    );

    let map = agent.get_config().unwrap();
    assert_eq!(map.get("type").unwrap(), "TRANSPORT_GRPC");
}
