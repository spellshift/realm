use super::super::agent::ImixAgent;
use super::super::task::TaskRegistry;
use eldritch::agent::agent::Agent;
use pb::config::Config;
use std::sync::Arc;
use transport::MockTransport;

#[tokio::test]
async fn test_start_reverse_shell() {
    let _ = pretty_env_logger::try_init();

    // Setup mocks
    let mut transport = MockTransport::default();

    // Expect clone to be called, and return a mock that expects reverse_shell
    transport.expect_clone().returning(|| {
        let mut t = MockTransport::default();
        t.expect_reverse_shell().times(1).returning(|_, _| Ok(()));
        t.expect_is_active().returning(|| true);
        t
    });

    // Expect is_active to be called by get_usable_transport
    transport.expect_is_active().returning(|| true);

    // Handle required for ImixAgent spawning
    let handle = tokio::runtime::Handle::current();

    let task_registry = Arc::new(TaskRegistry::new());
    let (tx, _rx) = tokio::sync::mpsc::channel(1);
    let agent = Arc::new(ImixAgent::new(
        Config::default(),
        transport,
        handle,
        task_registry,
        tx,
    ));

    // Execution must happen in a separate thread to allow block_on
    let agent_clone = agent.clone();
    let result = std::thread::spawn(move || {
        agent_clone.start_reverse_shell(
            eldritch_agent::Context::Task(pb::c2::TaskContext {
                task_id: 12345,
                jwt: "some jwt".to_string(),
            }),
            Some("echo test".to_string()),
        )
    })
    .join()
    .unwrap();

    assert!(result.is_ok(), "start_reverse_shell should succeed");

    // Verify subtask is registered
    {
        let subtasks = agent.subtasks.lock().unwrap();
        assert!(
            subtasks.contains_key(&12345),
            "Subtask should be registered"
        );
    }

    // Test stop_task stops the subtask
    let stop_result = agent.stop_task(12345);
    assert!(stop_result.is_ok());

    {
        let subtasks = agent.subtasks.lock().unwrap();
        assert!(
            !subtasks.contains_key(&12345),
            "Subtask should be removed after stop"
        );
    }
}
