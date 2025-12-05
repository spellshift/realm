use alloc::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use core::sync::atomic::{AtomicBool, Ordering};

use eldritchv2::Interpreter;
use eldritch_core::{BufferPrinter, Value};
use eldritch_core::conversion::ToValue;
use eldritch_libagent::agent::Agent;
use eldritch_libagent::std::StdAgentLibrary;
use eldritch_libassets::std::StdAssetsLibrary;
use eldritch_libpivot::std::StdPivotLibrary;
use pb::c2::Task;
use tokio::task::JoinHandle;

lazy_static::lazy_static! {
    static ref TASKS: Mutex<BTreeMap<i64, TaskHandle>> = Mutex::new(BTreeMap::new());
}

struct TaskHandle {
    #[allow(dead_code)] // Keep for future use/tracking
    start_time: SystemTime,
    quest: String,
    interrupt_flag: Arc<AtomicBool>,
    handle: JoinHandle<()>,
}

pub struct TaskRegistry;

impl TaskRegistry {
    pub fn spawn(task: Task, agent: Arc<dyn Agent>) {
        let task_id = task.id;
        let tome = task.tome.clone();

        let interrupt_flag = Arc::new(AtomicBool::new(false));
        let flag_for_thread = interrupt_flag.clone();
        let quest_name = task.quest_name.clone();

        let handle = tokio::task::spawn_blocking(move || {
            if let Some(tome) = tome {
                // Setup Interpreter
                let printer = Arc::new(BufferPrinter::new());
                let mut interp = Interpreter::new_with_printer(printer.clone())
                    .with_default_libs();

                // Inject interruption capability
                interp.inner = interp.inner.with_interrupt(flag_for_thread);

                // Register Agent Library
                let agent_lib = StdAgentLibrary::new(agent.clone(), task_id);
                interp.register_module("agent", Value::Foreign(Arc::new(agent_lib)));

                // Register Assets Library
                let remote_assets = tome.file_names.clone();
                let assets_lib = StdAssetsLibrary::new(agent.clone(), remote_assets);
                interp.register_module("assets", Value::Foreign(Arc::new(assets_lib)));

                // Inject Pivot Library with Agent context (override default)
                let pivot_lib = StdPivotLibrary::new(Some(agent.clone()));
                interp.inner.register_lib(pivot_lib);

                // Inject input_params
                let params_map: BTreeMap<String, String> = tome.parameters.into_iter().collect();
                let params_val = params_map.to_value();
                interp.define_variable("input_params", params_val);

                // Run
                let code = tome.eldritch;
                match interp.interpret(&code) {
                    Ok(v) => {
                        let out = printer.read();
                        log::info!("Task Success: {v} {out}");
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

        let mut tasks = TASKS.lock().unwrap();
        tasks.insert(
            task_id,
            TaskHandle {
                start_time: SystemTime::now(),
                quest: quest_name,
                interrupt_flag,
                handle,
            },
        );
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
        if let Some(handle) = tasks.remove(&task_id) {
            log::info!("Task {} stop requested", task_id);
            handle.interrupt_flag.store(true, Ordering::Relaxed);
            handle.handle.abort();
        }
    }
}
