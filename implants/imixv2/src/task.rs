use alloc::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::SystemTime;

use eldritchv2::{conversion::ToValue, BufferPrinter, Interpreter};
use eldritch_libagent::agent::Agent;
use pb::c2::Task;

lazy_static::lazy_static! {
    static ref TASKS: Mutex<BTreeMap<i64, TaskHandle>> = Mutex::new(BTreeMap::new());
}

struct TaskHandle {
    #[allow(dead_code)] // Keep for future use/tracking
    start_time: SystemTime,
    quest: String,
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
            tasks.insert(
                task_id,
                TaskHandle {
                    start_time: SystemTime::now(),
                    quest: task.quest_name.clone(),
                },
            );
        }

        thread::spawn(move || {
            if let Some(tome) = tome {
                // Setup Interpreter
                let printer = Arc::new(BufferPrinter::new());
                let mut interp = Interpreter::new_with_printer(printer.clone()).with_default_libs();

                // Register Task Context (Agent, Report, Assets) using the new helper
                let remote_assets = tome.file_names.clone();
                interp = interp.with_task_context(agent.clone(), task_id, remote_assets);

                // Inject input_params
                // tome.parameters is converted to a BTreeMap which ToValue supports
                let params_map: BTreeMap<String, String> = tome.parameters.into_iter().collect();
                let params_val = params_map.to_value();
                interp.define_variable("input_params", params_val);

                // Run
                let code = tome.eldritch;
                match interp.interpret(&code) {
                    Ok(v) => {
                        let out = printer.read();
                        log::info!("Task Success: {v} {out}");
                        // Success - implicit reporting via agent lib calls
                        let _ = agent.report_task_output(pb::c2::ReportTaskOutputRequest {
                            output: Some(pb::c2::TaskOutput {
                                id: task_id,
                                output: out,
                                error: None,
                                exec_started_at: None,
                                exec_finished_at: None,
                            }),
                        });
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
                            }),
                        });
                    }
                }
            } else {
                log::warn!("Task {task_id} has no tome");
            }

            // Cleanup
            log::info!("Completed Task: {task_id}");
            let mut tasks = TASKS.lock().unwrap();
            tasks.remove(&task_id);
        });
    }

    pub fn list() -> Vec<Task> {
        let tasks = TASKS.lock().unwrap();
        tasks
            .iter()
            .map(|(id, handle)| Task {
                id: *id,
                tome: None,
                quest_name: handle.quest.clone(),
            })
            .collect()
    }

    pub fn stop(task_id: i64) {
        let mut tasks = TASKS.lock().unwrap();
        if tasks.remove(&task_id).is_some() {
            log::info!("Task {} stop requested (thread may persist)", task_id);
        }
    }
}
