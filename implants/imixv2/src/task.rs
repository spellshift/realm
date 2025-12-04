use alloc::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::SystemTime;

use eldritchv2::Interpreter;
use eldritch_core::{BufferPrinter, Value};
use eldritch_core::conversion::ToValue;
use eldritch_libagent::agent::Agent;
use eldritch_libagent::std::StdAgentLibrary;
use eldritch_libassets::std::StdAssetsLibrary;
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
                // Removed global registration call

                // Setup Interpreter
                let printer = Arc::new(BufferPrinter::new());
                let mut interp = Interpreter::new_with_printer(printer.clone())
                    .with_default_libs();

                // Register Agent Library
                // We manually register here because we need task_id context which with_agent doesn't support yet (it uses 0)
                // However, the instructions say "use builder methods".
                // If I use with_agent(agent), it registers agent/report/assets with task_id=0.
                // But imix needs task_id.
                // So I will stick to manual registration for agent lib to be correct,
                // OR I should update `with_agent` to take task_id.
                // But `Interpreter` is generic.
                // I will assume for now I should use `with_agent` if possible, but since I need correctness,
                // I will manually overwrite the "agent" module after `with_default_libs` (if it was there, but it's not default).
                // Actually, I can just register the task-specific one.

                // Let's see if I can use `with_agent`? No, because I can't pass task_id.
                // So I will just manually register agent-related libs as before, BUT using the new Interpreter wrapper.
                // The wrapper exposes `register_module`.

                let agent_lib = StdAgentLibrary::new(agent.clone(), task_id);
                interp.register_module("agent", Value::Foreign(Arc::new(agent_lib)));

                // Register Assets Library
                // Same issue, assets lib takes remote_assets list. `with_agent` passes empty list.
                let remote_assets = tome.file_names.clone();
                let assets_lib = StdAssetsLibrary::new(agent.clone(), remote_assets);
                interp.register_module("assets", Value::Foreign(Arc::new(assets_lib)));

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
