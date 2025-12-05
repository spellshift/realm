use super::super::agent::ImixAgent;
use eldritch_libagent::agent::Agent;
use pb::c2;
use pb::config::Config;
use transport::MockTransport;

#[tokio::test]
async fn test_new_agent() {
    let config = Config::default();
    let transport = MockTransport::default();
    let agent = ImixAgent::new(config, transport);
    assert_eq!(agent.get_callback_interval_u64(), 5); // Default is 5
}

#[tokio::test]
async fn test_fetch_tasks() {
    let config = Config::default();
    let mut transport = MockTransport::default();

    transport
        .expect_claim_tasks()
        .times(1)
        .returning(|_| Ok(c2::ClaimTasksResponse { tasks: vec![] }));

    let agent = ImixAgent::new(config, transport);
    let tasks = agent.fetch_tasks().await.unwrap();
    assert!(tasks.is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_report_task_output() {
    let config = Config::default();
    let mut transport = MockTransport::default();

    transport
        .expect_report_task_output()
        .times(1)
        .returning(|_| Ok(c2::ReportTaskOutputResponse {}));

    let agent = ImixAgent::new(config, transport);
    let req = c2::ReportTaskOutputRequest { output: None };

    // We need to run this on a separate thread to avoid "Cannot start a runtime from within a runtime"
    // because ImixAgent uses block_on.
    // However, we are in an async test which is already running a runtime.
    // The issue is ImixAgent::report_task_output creates a NEW runtime and blocks on it.
    // This is illegal inside an existing tokio runtime.

    // Solution: Run the blocking code in a spawn_blocking block or just a regular thread spawn.
    let handle = std::thread::spawn(move || {
         agent.report_task_output(req).unwrap();
    });
    handle.join().unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_report_process_list() {
    let config = Config::default();
    let mut transport = MockTransport::default();

    transport
        .expect_report_process_list()
        .times(1)
        .returning(|_| Ok(c2::ReportProcessListResponse {}));

    let agent = ImixAgent::new(config, transport);
    let req = c2::ReportProcessListRequest {
        task_id: 0,
        list: None,
    };

    let handle = std::thread::spawn(move || {
        agent.report_process_list(req).unwrap();
    });
    handle.join().unwrap();
}
