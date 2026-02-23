use crate::agent::ImixAgent;
use crate::task::TaskRegistry;
use pb::c2::{
    report_output_request, ReportOutputRequest, ReportShellTaskOutputMessage,
    ReportTaskOutputMessage, ShellTaskContext, ShellTaskOutput, TaskContext, TaskOutput,
};
use pb::config::Config;
use std::sync::{Arc, Mutex};
use transport::MockTransport;

#[tokio::test]
async fn test_agent_output_aggregation() {
    let _ = pretty_env_logger::try_init();

    // 1. Setup Mock Transport
    let mut transport = MockTransport::default();
    let actual_requests = Arc::new(Mutex::new(Vec::new()));
    let requests_clone = actual_requests.clone();

    // We expect 3 calls:
    // 1. Task 100
    // 2. Shell Task 500
    // 3. Shell Task 600
    transport
        .expect_report_output()
        .times(3)
        .returning(move |req| {
            requests_clone.lock().unwrap().push(req);
            Ok(pb::c2::ReportOutputResponse {})
        });

    transport.expect_is_active().returning(|| true);

    // 2. Setup Agent
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

    // 3. Send outputs

    // Task Output (Task ID 100)
    let task_out_1 = ReportOutputRequest {
        message: Some(report_output_request::Message::TaskOutput(
            ReportTaskOutputMessage {
                context: Some(TaskContext {
                    task_id: 100,
                    jwt: "jwt".into(),
                }),
                output: Some(TaskOutput {
                    id: 100,
                    output: "Part 1".into(),
                    error: None,
                    exec_started_at: None,
                    exec_finished_at: None,
                }),
            },
        )),
    };
    agent.output_tx.send(task_out_1).unwrap();

    let task_out_2 = ReportOutputRequest {
        message: Some(report_output_request::Message::TaskOutput(
            ReportTaskOutputMessage {
                context: Some(TaskContext {
                    task_id: 100,
                    jwt: "jwt".into(),
                }),
                output: Some(TaskOutput {
                    id: 100,
                    output: " Part 2".into(),
                    error: None,
                    exec_started_at: None,
                    exec_finished_at: None,
                }),
            },
        )),
    };
    agent.output_tx.send(task_out_2).unwrap();

    // Shell Task Output (Shell Task ID 500)
    let shell_out_1 = ReportOutputRequest {
        message: Some(report_output_request::Message::ShellTaskOutput(
            ReportShellTaskOutputMessage {
                context: Some(ShellTaskContext {
                    shell_task_id: 500,
                    jwt: "jwt".into(),
                }),
                output: Some(ShellTaskOutput {
                    id: 500,
                    output: "Shell 1".into(),
                    error: None,
                    exec_started_at: None,
                    exec_finished_at: None,
                }),
            },
        )),
    };
    agent.output_tx.send(shell_out_1).unwrap();

    let shell_out_2 = ReportOutputRequest {
        message: Some(report_output_request::Message::ShellTaskOutput(
            ReportShellTaskOutputMessage {
                context: Some(ShellTaskContext {
                    shell_task_id: 500,
                    jwt: "jwt".into(),
                }),
                output: Some(ShellTaskOutput {
                    id: 500,
                    output: " continued".into(),
                    error: None,
                    exec_started_at: None,
                    exec_finished_at: None,
                }),
            },
        )),
    };
    agent.output_tx.send(shell_out_2).unwrap();

    // Another Shell Task Output (Shell Task ID 600)
    let shell_out_3 = ReportOutputRequest {
        message: Some(report_output_request::Message::ShellTaskOutput(
            ReportShellTaskOutputMessage {
                context: Some(ShellTaskContext {
                    shell_task_id: 600,
                    jwt: "jwt".into(),
                }),
                output: Some(ShellTaskOutput {
                    id: 600,
                    output: "Shell 2".into(),
                    error: None,
                    exec_started_at: None,
                    exec_finished_at: None,
                }),
            },
        )),
    };
    agent.output_tx.send(shell_out_3).unwrap();

    // 4. Flush outputs
    agent.flush_outputs().await;

    // 5. Verify
    let reqs = actual_requests.lock().unwrap();
    assert_eq!(reqs.len(), 3, "Should have 3 aggregated requests");

    // Check Task 100
    let task_100 = reqs
        .iter()
        .find(|r| match &r.message {
            Some(report_output_request::Message::TaskOutput(m)) => {
                m.context.as_ref().map(|c| c.task_id) == Some(100)
            }
            _ => false,
        })
        .expect("Task 100 output missing");

    match &task_100.message {
        Some(report_output_request::Message::TaskOutput(m)) => {
            assert_eq!(m.output.as_ref().unwrap().output, "Part 1 Part 2");
        }
        _ => panic!("Expected TaskOutput"),
    }

    // Check Shell 500
    let shell_500 = reqs
        .iter()
        .find(|r| match &r.message {
            Some(report_output_request::Message::ShellTaskOutput(m)) => {
                m.context.as_ref().map(|c| c.shell_task_id) == Some(500)
            }
            _ => false,
        })
        .expect("Shell 500 output missing");

    match &shell_500.message {
        Some(report_output_request::Message::ShellTaskOutput(m)) => {
            assert_eq!(m.output.as_ref().unwrap().output, "Shell 1 continued");
        }
        _ => panic!("Expected ShellTaskOutput"),
    }

    // Check Shell 600
    let shell_600 = reqs
        .iter()
        .find(|r| match &r.message {
            Some(report_output_request::Message::ShellTaskOutput(m)) => {
                m.context.as_ref().map(|c| c.shell_task_id) == Some(600)
            }
            _ => false,
        })
        .expect("Shell 600 output missing");

    match &shell_600.message {
        Some(report_output_request::Message::ShellTaskOutput(m)) => {
            assert_eq!(m.output.as_ref().unwrap().output, "Shell 2");
        }
        _ => panic!("Expected ShellTaskOutput"),
    }
}
