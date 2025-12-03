use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::SystemTime;

use eldritch_core::{Interpreter, Value};
use eldritch_libagent::std::StdAgentLibrary;
use eldritch_libagent::agent::Agent;
use pb::c2::Task;

lazy_static::lazy_static! {
    static ref TASKS: Mutex<BTreeMap<i64, TaskHandle>> = Mutex::new(BTreeMap::new());
}

struct TaskHandle {
    #[allow(dead_code)] // Keep for future use/tracking
    start_time: SystemTime,
    quest: String,
    // Add a flag for cooperative cancellation if we can inject it into the Interpreter or StdAgentLibrary
    // Currently StdAgentLibrary doesn't check it, but we can add it later.
    // For now, we just track existence.
}

pub struct TaskRegistry;

impl TaskRegistry {
    pub fn spawn(task: Task, agent: Arc<dyn Agent>) {
        let task_id = task.id;
        let tome = task.tome.clone();

        {
            let mut tasks = TASKS.lock().unwrap();
            if tasks.contains_key(&task_id) {
                // Already running
                return;
            }
            tasks.insert(task_id, TaskHandle {
                start_time: SystemTime::now(),
                quest: task.quest_name.clone(),
            });
        }

        thread::spawn(move || {
            if let Some(tome) = tome {
                // Setup Interpreter
                let mut interp = Interpreter::new();

                // Register Agent Library
                let agent_lib = StdAgentLibrary::new(agent.clone(), task_id);
                interp.register_module("agent", Value::Foreign(Arc::new(agent_lib)));

                // TODO: Register other Standard Libraries (File, etc.)

                // Run
                let code = tome.eldritch;
                match interp.interpret(&code) {
                    Ok(_) => {
                        // Success - implicit reporting via agent lib calls
                    }
                    Err(e) => {
                         // Report error
                         let _ = agent.report_task_output(pb::c2::ReportTaskOutputRequest {
                             output: Some(pb::c2::TaskOutput {
                                 id: task_id,
                                 output: String::new(),
                                 error: Some(pb::c2::TaskError { msg: e }),
                                 exec_started_at: None,
                                 exec_finished_at: None,
                             })
                         });
                    }
                }
            } else {
                 log::warn!("Task {} has no tome", task_id);
            }

            // Cleanup
            let mut tasks = TASKS.lock().unwrap();
            tasks.remove(&task_id);
        });
    }

    pub fn list() -> Vec<Task> {
        let tasks = TASKS.lock().unwrap();
        tasks.iter().map(|(id, handle)| {
            Task {
                id: *id,
                tome: None,
                quest_name: handle.quest.clone(),
            }
        }).collect()
    }

    pub fn stop(task_id: i64) {
        let mut tasks = TASKS.lock().unwrap();
        if tasks.remove(&task_id).is_some() {
            log::info!("Task {} stop requested (thread may persist)", task_id);
        }
    }
}
