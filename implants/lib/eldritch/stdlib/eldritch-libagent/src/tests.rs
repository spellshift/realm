use crate::AgentLibrary;
use crate::agent::Agent;
use crate::std::StdAgentLibrary;
use alloc::sync::Arc;
use eldritch_core::Value;
use eldritch_mockagent::MockAgent;
use std::thread;

#[test]
fn test_get_config() {
    let agent = Arc::new(MockAgent::new());
    let lib = StdAgentLibrary::new(
        agent,
        eldritch_agent::Context::Task(pb::c2::TaskContext {
            task_id: 1,
            jwt: "testjwt".to_string(),
        }),
    );

    let config = lib.get_config().unwrap();
    assert_eq!(config.get("key"), Some(&Value::String("value".to_string())));
    assert_eq!(config.get("interval"), Some(&Value::Int(5)));
}

#[test]
fn test_concurrent_access() {
    let agent = Arc::new(MockAgent::new());
    let lib = StdAgentLibrary::new(
        agent.clone(),
        eldritch_agent::Context::Task(pb::c2::TaskContext {
            task_id: 1,
            jwt: "testjwt".to_string(),
        }),
    );
    let lib = Arc::new(lib);

    let mut handles = vec![];

    // Reader threads
    for _ in 0..10 {
        let lib_clone = lib.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                let config = lib_clone.get_config().unwrap();
                assert!(config.contains_key("key"));
                assert!(config.contains_key("interval"));
            }
        }));
    }

    // Writer thread
    let agent_clone = agent.clone();
    handles.push(thread::spawn(move || {
        for i in 0..100 {
            let _ = agent_clone.set_callback_interval(i as u64);
        }
    }));

    for h in handles {
        h.join().unwrap();
    }

    // Verify final state
    let config = lib.get_config().unwrap();
    // The last written value depends on thread scheduling, but it should be valid int
    if let Some(Value::Int(val)) = config.get("interval") {
        assert!(*val >= 0 && *val <= 100);
    } else {
        panic!("Interval should be an int");
    }
}

#[test]
fn test_agent_library_methods() {
    let mock = MockAgent::new().with_asset("test_asset", b"test content");
    mock.tasks.lock().unwrap().push(pb::c2::Task {
        id: 42,
        tome: None,
        quest_name: "quest".to_string(),
        jwt: "jwt".to_string(),
    });

    let agent = Arc::new(mock);
    let lib = StdAgentLibrary::new(
        agent.clone(),
        eldritch_agent::Context::Task(pb::c2::TaskContext {
            task_id: 1,
            jwt: "testjwt".to_string(),
        }),
    );

    // Test fetch_asset
    let asset = lib.fetch_asset("test_asset".to_string()).unwrap();
    assert_eq!(asset, b"test content");

    // Test set_callback_interval
    lib.set_callback_interval(10).unwrap();
    let config = agent.get_config().unwrap();
    assert_eq!(config.get("interval"), Some(&"10".to_string()));

    // Test get_callback_interval
    assert_eq!(lib.get_callback_interval().unwrap(), 10);

    // Test get_transport
    assert_eq!(lib.get_transport().unwrap(), "http".to_string());

    // Test list_transports
    let transports = lib.list_transports().unwrap();
    assert_eq!(transports, vec!["http".to_string(), "dns".to_string()]);

    // Test reset_transport
    lib.reset_transport().unwrap();
    assert_eq!(*agent.reset_transport_calls.lock().unwrap(), 1);

    // Test set_callback_uri
    lib.set_callback_uri("http://example.com".to_string())
        .unwrap();
    assert_eq!(agent.set_callback_uri_calls.lock().unwrap().len(), 1);
    assert_eq!(
        agent.set_callback_uri_calls.lock().unwrap()[0],
        "http://example.com".to_string()
    );

    // Test list_tasks
    let tasks = lib.list_tasks().unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].0.id, 42);

    // Test claim_tasks
    let claimed = lib.claim_tasks().unwrap();
    assert_eq!(claimed.len(), 1);
    assert_eq!(claimed[0].0.id, 42);
    assert_eq!(agent.claim_tasks_calls.lock().unwrap().len(), 1);

    // Test stop_task
    lib.stop_task(42).unwrap();
    assert_eq!(agent.stop_task_calls.lock().unwrap().len(), 1);
    assert_eq!(agent.stop_task_calls.lock().unwrap()[0], 42);

    // Test report_credential
    let cred = pb::eldritch::Credential {
        principal: "user".to_string(),
        secret: "password".to_string(),
        kind: 0,
    };
    lib.report_credential(crate::CredentialWrapper(cred.clone()))
        .unwrap();
    assert_eq!(agent.report_credential_calls.lock().unwrap().len(), 1);

    // Test report_file
    let file = pb::eldritch::File {
        metadata: None,
        chunk: b"data".to_vec(),
    };
    lib.report_file(crate::FileWrapper(file.clone())).unwrap();
    assert_eq!(agent.report_file_calls.lock().unwrap().len(), 1);

    // Test report_process_list
    let plist = pb::eldritch::ProcessList {
        list: vec![pb::eldritch::Process {
            pid: 1,
            ppid: 0,
            name: "test".to_string(),
            path: "path".to_string(),
            cmd: "cmdline".to_string(),
            principal: "root".to_string(),
            env: "env".to_string(),
            cwd: "cwd".to_string(),
            status: 0,
            start_time: 0,
        }],
    };
    lib.report_process_list(crate::ProcessListWrapper(plist.clone()))
        .unwrap();
    assert_eq!(agent.reported_processes.lock().unwrap().len(), 1);

    // Test report_task_output
    lib.report_task_output("output".to_string(), Some("error".to_string()))
        .unwrap();
    assert_eq!(agent.report_output_calls.lock().unwrap().len(), 1);
    let output_call = agent
        .report_output_calls
        .lock()
        .unwrap()
        .first()
        .unwrap()
        .clone();

    match output_call.message.unwrap() {
        pb::c2::report_output_request::Message::TaskOutput(out) => {
            let actual = out.output.unwrap();
            assert_eq!(actual.output, "output".to_string());
            assert_eq!(actual.error.unwrap().msg, "error".to_string());
        }
        _ => panic!("Expected task output"),
    }
}

#[test]
fn test_terminate() {
    // Cannot really test terminate properly since it does std::process::exit(0)
    // However, maybe testing if it's there is enough.
    // Wait, testing it will exit the test runner.
}
