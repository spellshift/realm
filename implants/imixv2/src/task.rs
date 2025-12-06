use alloc::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::SystemTime;

use eldritch_libagent::agent::Agent;
use eldritchv2::{conversion::ToValue, BufferPrinter, Interpreter};
use pb::c2::Task;
use prost_types::Timestamp;

struct TaskHandle {
    #[allow(dead_code)] // Keep for future use/tracking
    start_time: SystemTime,
    quest: String,
}

#[derive(Clone)]
pub struct TaskRegistry {
    tasks: Arc<Mutex<BTreeMap<i64, TaskHandle>>>,
}

impl TaskRegistry {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    pub fn spawn(&self, task: Task, agent: Arc<dyn Agent>) {
        let task_id = task.id;
        let tome = task.tome.clone();

        {
            let mut tasks = self.tasks.lock().unwrap();
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

        let tasks_registry = self.tasks.clone();
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

                // Report Start
                let start_time = SystemTime::now();
                let _ = agent.report_task_output(pb::c2::ReportTaskOutputRequest {
                    output: Some(pb::c2::TaskOutput {
                        id: task_id,
                        output: String::new(),
                        error: None,
                        exec_started_at: Some(Timestamp::from(start_time)),
                        exec_finished_at: None,
                    }),
                });

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
                                exec_finished_at: Some(Timestamp::from(SystemTime::now())),
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
                                exec_finished_at: Some(Timestamp::from(SystemTime::now())),
                            }),
                        });
                    }
                }
            } else {
                log::warn!("Task {task_id} has no tome");
            }

            // Cleanup
            log::info!("Completed Task: {task_id}");
            let mut tasks = tasks_registry.lock().unwrap();
            tasks.remove(&task_id);
        });
    }

    pub fn list(&self) -> Vec<Task> {
        let tasks = self.tasks.lock().unwrap();
        tasks
            .iter()
            .map(|(id, handle)| Task {
                id: *id,
                tome: None,
                quest_name: handle.quest.clone(),
            })
            .collect()
    }

    pub fn stop(&self, task_id: i64) {
        let mut tasks = self.tasks.lock().unwrap();
        if tasks.remove(&task_id).is_some() {
            log::info!("Task {} stop requested (thread may persist)", task_id);
        }
    }
}
