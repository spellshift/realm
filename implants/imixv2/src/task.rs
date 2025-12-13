use alloc::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::SystemTime;

use eldritch_libagent::agent::Agent;
use eldritchv2::{Interpreter, Printer, Span, conversion::ToValue};
use pb::c2::{ReportTaskOutputRequest, Task, TaskError, TaskOutput};
use prost_types::Timestamp;
use tokio::sync::mpsc::{self, UnboundedSender};
use transport::Transport;

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

impl Default for TaskRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskRegistry {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    pub fn spawn<T: Transport + Sync + Send + Clone + 'static>(&self, task: Task, agent: Arc<ImixAgent<T>>) {
        let task_id = task.id;

        // 1. Register logic
        if !self.register_task(&task) {
            return;
        }

        let tasks_registry = self.tasks.clone();
        // Capture runtime handle to spawn streaming task
        let runtime_handle = tokio::runtime::Handle::current();

        // 2. Spawn thread
        #[cfg(debug_assertions)]
        log::info!("Spawning Task: {task_id}");

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

use crate::agent::ImixAgent;

fn execute_task<T: Transport + Sync + Send + Clone + 'static>(
    task_id: i64,
    tome: pb::eldritch::Tome,
    agent: Arc<ImixAgent<T>>,
    runtime_handle: tokio::runtime::Handle,
) {
    // Setup StreamPrinter and Interpreter
    let (tx, rx) = mpsc::unbounded_channel();
    let printer = Arc::new(StreamPrinter::new(tx));

    // We need to pass both agent and sync transport
    let sync_transport = agent.get_sync_transport();
    let mut interp = setup_interpreter(task_id, &tome, agent.clone(), sync_transport, printer.clone());

    // Report Start
    report_start(task_id, &agent);

    // Spawn output consumer task
    let consumer_join_handle =
        spawn_output_consumer(task_id, agent.clone(), runtime_handle.clone(), rx);

    // Run Interpreter with panic protection
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        interp.interpret(&tome.eldritch)
    }));

    // Explicitly drop interp and printer to close channel
    drop(printer);
    drop(interp);

    // Wait for consumer to finish processing all messages
    let _ = runtime_handle.block_on(consumer_join_handle);

    // Handle result
    match result {
        Ok(exec_result) => report_result(task_id, exec_result, &agent),
        Err(_) => {
            report_panic(task_id, &agent);
        }
    }
}

use transport::SyncTransport;

fn setup_interpreter(
    task_id: i64,
    tome: &pb::eldritch::Tome,
    agent: Arc<dyn Agent>,
    transport: Arc<dyn SyncTransport>,
    printer: Arc<StreamPrinter>,
) -> Interpreter {
    let mut interp = Interpreter::new_with_printer(printer).with_default_libs();

    // Register Task Context (Agent, Report, Assets)
    let remote_assets = tome.file_names.clone();
    interp = interp.with_task_context::<crate::assets::Asset>(agent, transport, task_id, remote_assets);

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

fn report_start<T: Transport + Send + Sync + 'static>(task_id: i64, agent: &Arc<ImixAgent<T>>) {
    #[cfg(debug_assertions)]
    log::info!("task={task_id} Started execution");

    // Buffer output locally using ImixAgent's buffer
    let req = ReportTaskOutputRequest {
        output: Some(TaskOutput {
            id: task_id,
            output: String::new(),
            error: None,
            exec_started_at: Some(Timestamp::from(SystemTime::now())),
            exec_finished_at: None,
        }),
    };

    if let Ok(mut buffer) = agent.output_buffer.lock() {
        buffer.push(req);
    }
}

fn spawn_output_consumer<T: Transport + Send + Sync + 'static>(
    task_id: i64,
    agent: Arc<ImixAgent<T>>,
    runtime_handle: tokio::runtime::Handle,
    mut rx: mpsc::UnboundedReceiver<String>,
) -> tokio::task::JoinHandle<()> {
    runtime_handle.spawn(async move {
        #[cfg(debug_assertions)]
        log::info!("task={task_id} Started output stream");

        while let Some(msg) = rx.recv().await {
            let req = ReportTaskOutputRequest {
                output: Some(TaskOutput {
                    id: task_id,
                    output: msg,
                    error: None,
                    exec_started_at: None,
                    exec_finished_at: None,
                }),
            };
             if let Ok(mut buffer) = agent.output_buffer.lock() {
                buffer.push(req);
            }
        }
    })
}

fn report_panic<T: Transport + Send + Sync + 'static>(task_id: i64, agent: &Arc<ImixAgent<T>>) {
    let req = ReportTaskOutputRequest {
        output: Some(TaskOutput {
            id: task_id,
            output: String::new(),
            error: Some(TaskError {
                msg: "Task execution panicked".to_string(),
            }),
            exec_started_at: None,
            exec_finished_at: Some(Timestamp::from(SystemTime::now())),
        }),
    };
    if let Ok(mut buffer) = agent.output_buffer.lock() {
        buffer.push(req);
    }
}

fn report_result<T: Transport + Send + Sync + 'static>(
    task_id: i64,
    result: Result<eldritch_core::Value, String>,
    agent: &Arc<ImixAgent<T>>,
) {
    match result {
        Ok(v) => {
            #[cfg(debug_assertions)]
            log::info!("task={task_id} Success: {v}");

            let req = ReportTaskOutputRequest {
                output: Some(TaskOutput {
                    id: task_id,
                    output: String::new(),
                    error: None,
                    exec_started_at: None,
                    exec_finished_at: Some(Timestamp::from(SystemTime::now())),
                }),
            };
            if let Ok(mut buffer) = agent.output_buffer.lock() {
                buffer.push(req);
            }
        }
        Err(e) => {
            #[cfg(debug_assertions)]
            log::info!("task={task_id} Error: {e}");

            let req = ReportTaskOutputRequest {
                output: Some(TaskOutput {
                    id: task_id,
                    output: String::new(),
                    error: Some(TaskError { msg: e }),
                    exec_started_at: None,
                    exec_finished_at: Some(Timestamp::from(SystemTime::now())),
                }),
            };
            if let Ok(mut buffer) = agent.output_buffer.lock() {
                buffer.push(req);
            }
        }
    }
}
