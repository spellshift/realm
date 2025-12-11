use alloc::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::SystemTime;

use eldritch_libagent::agent::Agent;
use eldritchv2::{conversion::ToValue, Interpreter, Printer, Span};
use pb::c2::{ReportTaskOutputRequest, Task, TaskError, TaskOutput};
use prost_types::Timestamp;
use tokio::sync::mpsc::{self, UnboundedSender};

#[derive(Debug)]
struct StreamPrinter {
    tx: UnboundedSender<String>,
}

impl StreamPrinter {
    fn new(tx: UnboundedSender<String>) -> Self {
        Self { tx }
    }
}

impl Printer for StreamPrinter {
    fn print_out(&self, _span: &Span, s: &str) {
        // We format with newline to match BufferPrinter behavior which separates lines
        let _ = self.tx.send(format!("{}\n", s));
    }

    fn print_err(&self, _span: &Span, s: &str) {
        // We format with newline to match BufferPrinter behavior
        let _ = self.tx.send(format!("{}\n", s));
    }
}

struct TaskHandle {
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

        // 1. Register logic
        if !self.register_task(&task) {
            return;
        }

        let tasks_registry = self.tasks.clone();
        // Capture runtime handle to spawn streaming task
        let runtime_handle = tokio::runtime::Handle::current();

        // 2. Spawn thread
        thread::spawn(move || {
            if let Some(tome) = task.tome {
                execute_task(task_id, tome, agent, runtime_handle);
            } else {
                log::warn!("Task {task_id} has no tome");
            }

            // Cleanup
            log::info!("Completed Task: {task_id}");
            let mut tasks = tasks_registry.lock().unwrap();
            tasks.remove(&task_id);
        });
    }

    fn register_task(&self, task: &Task) -> bool {
        let mut tasks = self.tasks.lock().unwrap();
        if tasks.contains_key(&task.id) {
            return false;
        }
        tasks.insert(
            task.id,
            TaskHandle {
                quest: task.quest_name.clone(),
            },
        );
        true
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
            log::info!("Task {task_id} stop requested (thread may persist)");
        }
    }
}

fn execute_task(
    task_id: i64,
    tome: pb::eldritch::Tome,
    agent: Arc<dyn Agent>,
    runtime_handle: tokio::runtime::Handle,
) {
    // Setup StreamPrinter and Interpreter
    let (tx, rx) = mpsc::unbounded_channel();
    let printer = Arc::new(StreamPrinter::new(tx));
    let mut interp = setup_interpreter(task_id, &tome, agent.clone(), printer.clone());

    // Report Start
    report_start(task_id, &agent);

    // Spawn output consumer task
    let consumer_join_handle = spawn_output_consumer(task_id, agent.clone(), runtime_handle.clone(), rx);

    // Run Interpreter
    let result = interp.interpret(&tome.eldritch);

    // Explicitly drop interp and printer to close channel
    drop(printer);
    drop(interp);

    // Wait for consumer to finish processing all messages
    let _ = runtime_handle.block_on(consumer_join_handle);

    // Report Result
    report_result(task_id, result, &agent);
}

fn setup_interpreter(
    task_id: i64,
    tome: &pb::eldritch::Tome,
    agent: Arc<dyn Agent>,
    printer: Arc<StreamPrinter>,
) -> Interpreter {
    let mut interp = Interpreter::new_with_printer(printer).with_default_libs();

    // Register Task Context (Agent, Report, Assets)
    let remote_assets = tome.file_names.clone();
    interp = interp.with_task_context::<crate::assets::Asset>(agent, task_id, remote_assets);

    // Inject input_params
    let params_map: BTreeMap<String, String> = tome
        .parameters
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    let params_val = params_map.to_value();
    interp.define_variable("input_params", params_val);

    interp
}

fn report_start(task_id: i64, agent: &Arc<dyn Agent>) {
    let _ = agent.report_task_output(ReportTaskOutputRequest {
        output: Some(TaskOutput {
            id: task_id,
            output: String::new(),
            error: None,
            exec_started_at: Some(Timestamp::from(SystemTime::now())),
            exec_finished_at: None,
        }),
    });
}

fn spawn_output_consumer(
    task_id: i64,
    agent: Arc<dyn Agent>,
    runtime_handle: tokio::runtime::Handle,
    mut rx: mpsc::UnboundedReceiver<String>,
) -> tokio::task::JoinHandle<()> {
    runtime_handle.spawn(async move {
        while let Some(msg) = rx.recv().await {
            let _ = agent.report_task_output(ReportTaskOutputRequest {
                output: Some(TaskOutput {
                    id: task_id,
                    output: msg,
                    error: None,
                    exec_started_at: None,
                    exec_finished_at: None,
                }),
            });
        }
    })
}

fn report_result(task_id: i64, result: Result<eldritch_core::Value, String>, agent: &Arc<dyn Agent>) {
    match result {
        Ok(v) => {
            log::info!("Task Success: {v}");
            let _ = agent.report_task_output(ReportTaskOutputRequest {
                output: Some(TaskOutput {
                    id: task_id,
                    output: String::new(),
                    error: None,
                    exec_started_at: None,
                    exec_finished_at: Some(Timestamp::from(SystemTime::now())),
                }),
            });
        }
        Err(e) => {
            let _ = agent.report_task_output(ReportTaskOutputRequest {
                output: Some(TaskOutput {
                    id: task_id,
                    output: String::new(),
                    error: Some(TaskError { msg: e }),
                    exec_started_at: None,
                    exec_finished_at: Some(Timestamp::from(SystemTime::now())),
                }),
            });
        }
    }
}
