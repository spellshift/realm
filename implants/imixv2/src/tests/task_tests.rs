use super::super::task::TaskRegistry;
use alloc::collections::BTreeMap;
use eldritch_libagent::agent::Agent;
use pb::c2;
use pb::eldritch::Tome;
use std::sync::Arc;
use std::time::Duration;
use crate::agent::ImixAgent;
use transport::MockTransport;
use pb::config::Config;

#[tokio::test]
async fn test_task_registry_spawn() {
    let mut transport = MockTransport::default();
    transport.expect_clone().returning(MockTransport::default);

    // Setup basic mock behavior if needed by task execution?
    // Tasks might fetch assets or report things.
    // The test task "print(...)" doesn't use transport, but `setup_interpreter` registers libraries which take transport.
    // But print just outputs to stdout which `StreamPrinter` handles and `ImixAgent` buffers.

    let config = Config::default();
    let runtime_handle = tokio::runtime::Handle::current();
    let registry = Arc::new(TaskRegistry::new());

    let agent = Arc::new(ImixAgent::new(
        config,
        transport,
        runtime_handle,
        registry.clone(),
    ));

    let task_id = 123;
    let task = c2::Task {
        id: task_id,
        tome: Some(Tome {
            eldritch: "print(\"Hello World\")".to_string(),
            ..Default::default()
        }),
        quest_name: "test_quest".to_string(),
    };

    registry.spawn(task, agent.clone());

    // Wait a bit more for execution
    tokio::time::sleep(Duration::from_secs(2)).await;

    let reports = agent.output_buffer.lock().unwrap();
    assert!(!reports.is_empty(), "Should have reported output");

    // Check for Hello World
    let has_output = reports.iter().any(|r| {
        r.output
            .as_ref()
            .map(|o| o.output.contains("Hello World"))
            .unwrap_or(false)
    });
    assert!(
        has_output,
        "Should have found report containing 'Hello World'"
    );

    // Check completion
    let has_finished = reports.iter().any(|r| {
        r.output
            .as_ref()
            .map(|o| o.exec_finished_at.is_some())
            .unwrap_or(false)
    });
    assert!(has_finished, "Should have marked task as finished");
}

#[tokio::test]
async fn test_task_streaming_output() {
    let mut transport = MockTransport::default();
    transport.expect_clone().returning(MockTransport::default);

    let config = Config::default();
    let runtime_handle = tokio::runtime::Handle::current();
    let registry = Arc::new(TaskRegistry::new());

    let agent = Arc::new(ImixAgent::new(
        config,
        transport,
        runtime_handle,
        registry.clone(),
    ));

    let task_id = 456;
    // Removed indentation and loops to avoid parser errors in string literal
    let code = "print(\"Chunk 1\")\nprint(\"Chunk 2\")";
    println!("Code: {:?}", code);

    let task = c2::Task {
        id: task_id,
        tome: Some(Tome {
            eldritch: code.to_string(),
            ..Default::default()
        }),
        quest_name: "streaming_test".to_string(),
    };

    registry.spawn(task, agent.clone());

    tokio::time::sleep(Duration::from_secs(3)).await;

    let reports = agent.output_buffer.lock().unwrap();

    // Debug output
    println!("Reports count: {}", reports.len());
    for r in reports.iter() {
        println!("Report: {:?}", r);
    }

    let outputs: Vec<String> = reports
        .iter()
        .filter_map(|r| r.output.as_ref().map(|o| o.output.clone()))
        .filter(|s| !s.is_empty())
        .collect();

    assert!(!outputs.is_empty(), "Should have at least one output.");

    let combined = outputs.join("");
    assert!(combined.contains("Chunk 1"), "Missing Chunk 1");
    assert!(combined.contains("Chunk 2"), "Missing Chunk 2");
}

#[tokio::test]
async fn test_task_streaming_error() {
    let mut transport = MockTransport::default();
    transport.expect_clone().returning(MockTransport::default);

    let config = Config::default();
    let runtime_handle = tokio::runtime::Handle::current();
    let registry = Arc::new(TaskRegistry::new());

    let agent = Arc::new(ImixAgent::new(
        config,
        transport,
        runtime_handle,
        registry.clone(),
    ));

    let task_id = 789;
    let code = "print(\"Before Error\")\nx = 1 / 0";
    println!("Code: {:?}", code);

    let task = c2::Task {
        id: task_id,
        tome: Some(Tome {
            eldritch: code.to_string(),
            ..Default::default()
        }),
        quest_name: "error_test".to_string(),
    };

    registry.spawn(task, agent.clone());

    tokio::time::sleep(Duration::from_secs(3)).await;

    let reports = agent.output_buffer.lock().unwrap();

    // Debug
    println!("Reports count: {}", reports.len());
    for r in reports.iter() {
        println!("Report: {:?}", r);
    }

    let outputs: Vec<String> = reports
        .iter()
        .filter_map(|r| r.output.as_ref().map(|o| o.output.clone()))
        .filter(|s| !s.is_empty())
        .collect();

    assert!(
        outputs.iter().any(|s| s.contains("Before Error")),
        "Should contain pre-error output"
    );

    // Check for error report
    let error_report = reports.iter().find(|r| {
        r.output
            .as_ref()
            .map(|o| o.error.is_some())
            .unwrap_or(false)
    });
    assert!(error_report.is_some(), "Should report error");
}

#[tokio::test]
async fn test_task_registry_list_and_stop() {
    let mut transport = MockTransport::default();
    transport.expect_clone().returning(MockTransport::default);

    let config = Config::default();
    let runtime_handle = tokio::runtime::Handle::current();
    let registry = Arc::new(TaskRegistry::new());

    let agent = Arc::new(ImixAgent::new(
        config,
        transport,
        runtime_handle,
        registry.clone(),
    ));

    let task_id = 999;
    let task = c2::Task {
        id: task_id,
        tome: Some(Tome {
            eldritch: "print(\"x=1\")".to_string(),
            ..Default::default()
        }),
        quest_name: "list_stop_quest".to_string(),
    };

    registry.spawn(task, agent.clone());

    // Check list immediately
    let _list = registry.list();

    registry.stop(task_id);
    let tasks_after = registry.list();
    assert!(
        !tasks_after.iter().any(|t| t.id == task_id),
        "Task should be removed from list"
    );
}
