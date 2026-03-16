use crate::AgentLibrary;
use crate::agent::Agent;
use crate::std::StdAgentLibrary;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::sync::Arc;
use eldritch_core::Value;
use eldritch_mockagent::MockAgent;
use std::sync::RwLock;
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
