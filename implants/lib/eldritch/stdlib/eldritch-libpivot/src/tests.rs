use crate::{PivotLibrary, std::StdPivotLibrary};
use alloc::collections::{BTreeMap, BTreeSet};
use eldritch_agent::Agent;
use eldritch_mockagent::MockAgent;
use pb::c2;
use std::sync::{Arc, Mutex};

#[test]
fn test_reverse_shell_pty_delegation() {
    let agent = Arc::new(MockAgent::new());
    let task_id = 999;
    let lib = StdPivotLibrary::new(agent.clone(), eldritch_agent::Context::Task(pb::c2::TaskContext{ task_id, jwt:  "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.eyJiZWFjb25faWQiOjQyOTQ5Njc0OTUsImV4cCI6MTc2Nzc1MTI3MSwiaWF0IjoxNzY3NzQ3NjcxfQ.wVFQemOmhdjCSGdb_ap_DkA9GcGqDHt3UOn2w9fE0nc7nGLbAWqQkkOwuMqlsC9FXZoYglOz11eTUt9UyrmiBQ".to_string()}));

    // Test with command
    lib.reverse_shell_pty(Some("bash".to_string())).unwrap();

    let calls = agent.start_calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, task_id);
    assert_eq!(calls[0].1, Some("bash".to_string()));
}

#[test]
fn test_reverse_shell_pty_no_agent() {
    let lib = StdPivotLibrary::default();
    let result = lib.reverse_shell_pty(None);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "No agent available");
}

#[test]
fn test_reverse_shell_repl_delegation() {
    let agent = Arc::new(MockAgent::new());
    let task_id = 123;
    let lib = StdPivotLibrary::new(agent.clone(), eldritch_agent::Context::Task(pb::c2::TaskContext{ task_id, jwt: "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.eyJiZWFjb25faWQiOjQyOTQ5Njc0OTUsImV4cCI6MTc2Nzc1MTI3MSwiaWF0IjoxNzY3NzQ3NjcxfQ.wVFQemOmhdjCSGdb_ap_DkA9GcGqDHt3UOn2w9fE0nc7nGLbAWqQkkOwuMqlsC9FXZoYglOz11eTUt9UyrmiBQ".to_string()}));

    lib.reverse_shell_repl().unwrap();

    let calls = agent.repl_calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0], task_id);
}

#[test]
fn test_reverse_shell_repl_no_agent() {
    let lib = StdPivotLibrary::default();
    let result = lib.reverse_shell_repl();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "No agent available");
}
