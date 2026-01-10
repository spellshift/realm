use alloc::collections::BTreeMap;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::SystemTime;

use eldritch_libagent::agent::Agent;
use eldritchv2::{Interpreter, Printer, Span, assets::std::EmbeddedAssets, conversion::ToValue};
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

struct SubtaskHandle {
    name: String,
    _handle: tokio::task::JoinHandle<()>,
}

struct TaskHandle {
    quest: String,
    subtasks: Arc<RwLock<Vec<SubtaskHandle>>>,
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

    pub fn spawn(&self, task: Task, agent: Arc<dyn Agent>) {
        let task_id = task.id;
        let jwt = task.jwt.clone();

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
                execute_task(task_id, jwt, tome, agent, runtime_handle);
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
                subtasks: Arc::new(RwLock::new(Vec::new())),
            },
        );
        true
    }

    pub fn register_subtask(
        &self,
        task_id: i64,
        name: String,
        handle: tokio::task::JoinHandle<()>,
    ) {
        let tasks = self.tasks.lock().unwrap();
        if let Some(task) = tasks.get(&task_id) {
            let mut subtasks = task.subtasks.write().unwrap();
            subtasks.push(SubtaskHandle {
                name,
                _handle: handle,
            });
        } else {
            // Task might have finished already, or this is an orphan subtask.
            // In the future we might want to track these anyway.
            #[cfg(debug_assertions)]
            log::warn!("Attempted to register subtask '{name}' for non-existent task {task_id}");
        }
    }

    pub fn list(&self) -> Vec<Task> {
        let tasks = self.tasks.lock().unwrap();
        tasks
            .iter()
            .map(|(id, handle)| Task {
                id: *id,
                tome: None,
                quest_name: handle.quest.clone(),
                jwt: String::new(),
            })
            .collect()
    }

    pub fn stop(&self, task_id: i64) {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(handle) = tasks.remove(&task_id) {
            #[cfg(debug_assertions)]
            log::info!("Task {task_id} stop requested (thread may persist)");

            let subtasks = handle.subtasks.read().unwrap();
            for subtask in subtasks.iter() {
                subtask._handle.abort();
            }
        }
    }
}

fn execute_task(
    task_id: i64,
    jwt: String,
    tome: pb::eldritch::Tome,
    agent: Arc<dyn Agent>,
    runtime_handle: tokio::runtime::Handle,
) {
    // Setup StreamPrinter and Interpreter
    let (tx, rx) = mpsc::unbounded_channel();
    let printer = Arc::new(StreamPrinter::new(tx));
    let mut interp = setup_interpreter(task_id, jwt.clone(), &tome, agent.clone(), printer.clone());

    // Report Start
    report_start(task_id, jwt.clone(), &agent);

    // Spawn output consumer task
    let consumer_join_handle = spawn_output_consumer(
        task_id,
        jwt.clone(),
        agent.clone(),
        runtime_handle.clone(),
        rx,
    );

    // Run Interpreter with panic protection
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        interp.interpret(&tome.eldritch)
    }));

    // Explicitly drop interp and printer to close channel
    drop(printer);
    drop(interp);

    // Wait for consumer to finish processing all messages
    match runtime_handle.block_on(consumer_join_handle) {
        Ok(_) => {}
        Err(_e) => {
            #[cfg(debug_assertions)]
            log::error!("task={task_id} failed to wait for output consumer to join: {_e}");
        }
    }

    // Handle result
    match result {
        Ok(exec_result) => report_result(task_id, jwt.clone(), exec_result, &agent),
        Err(e) => {
            report_panic(task_id, jwt, &agent, format!("panic: {e:?}"));
        }
    }
}

fn setup_interpreter(
    task_id: i64,
    jwt: String,
    tome: &pb::eldritch::Tome,
    agent: Arc<dyn Agent>,
    printer: Arc<StreamPrinter>,
) -> Interpreter {
    let mut interp = Interpreter::new_with_printer(printer).with_default_libs();

    // Remote asset filenames
    let remote_assets = tome.file_names.clone();
    // Support embedded assets behind remote asset filenames
    let backend = Arc::new(EmbeddedAssets::<crate::assets::Asset>::new());
    // Register Task Context (Agent, Report, Assets)
    interp = interp.with_task_context(agent, task_id, jwt, remote_assets, backend);

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

fn report_start(task_id: i64, jwt: String, agent: &Arc<dyn Agent>) {
    #[cfg(debug_assertions)]
    log::info!("task={task_id} Started execution");

    match agent.report_task_output(ReportTaskOutputRequest {
        output: Some(TaskOutput {
            id: task_id,
            output: String::new(),
            error: None,
            exec_started_at: Some(Timestamp::from(SystemTime::now())),
            exec_finished_at: None,
        }),
        jwt,
    }) {
        Ok(_) => {}
        Err(_e) => {
            #[cfg(debug_assertions)]
            log::error!("task={task_id} failed to report task start: {_e}");
        }
    }
}

fn spawn_output_consumer(
    task_id: i64,
    jwt: String,
    agent: Arc<dyn Agent>,
    runtime_handle: tokio::runtime::Handle,
    mut rx: mpsc::UnboundedReceiver<String>,
) -> tokio::task::JoinHandle<()> {
    runtime_handle.spawn(async move {
        #[cfg(debug_assertions)]
        log::info!("task={task_id} Started output stream");
        let localjwt = jwt;
        while let Some(msg) = rx.recv().await {
            match agent.report_task_output(ReportTaskOutputRequest {
                output: Some(TaskOutput {
                    id: task_id,
                    output: msg,
                    error: None,
                    exec_started_at: None,
                    exec_finished_at: None,
                }),
                jwt: localjwt.clone(),
            }) {
                Ok(_) => {}
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    log::error!("task={task_id} failed to report output: {_e}");
                }
            }
        }
    })
}

fn report_panic(task_id: i64, jwt: String, agent: &Arc<dyn Agent>, err: String) {
    match agent.report_task_output(ReportTaskOutputRequest {
        output: Some(TaskOutput {
            id: task_id,
            output: String::new(),
            error: Some(TaskError { msg: err }),
            exec_started_at: None,
            exec_finished_at: Some(Timestamp::from(SystemTime::now())),
        }),
        jwt,
    }) {
        Ok(_) => {}
        Err(_e) => {
            #[cfg(debug_assertions)]
            log::error!("task={task_id} failed to report error: {_e}");
        }
    }
}

fn report_result(
    task_id: i64,
    jwt: String,
    result: Result<eldritch_core::Value, String>,
    agent: &Arc<dyn Agent>,
) {
    match result {
        Ok(v) => {
            #[cfg(debug_assertions)]
            log::info!("task={task_id} Success: {v}");

            let _ = agent.report_task_output(ReportTaskOutputRequest {
                output: Some(TaskOutput {
                    id: task_id,
                    output: String::new(),
                    error: None,
                    exec_started_at: None,
                    exec_finished_at: Some(Timestamp::from(SystemTime::now())),
                }),
                jwt,
            });
        }
        Err(e) => {
            #[cfg(debug_assertions)]
            log::info!("task={task_id} Error: {e}");

            match agent.report_task_output(ReportTaskOutputRequest {
                output: Some(TaskOutput {
                    id: task_id,
                    output: String::new(),
                    error: Some(TaskError { msg: e }),
                    exec_started_at: None,
                    exec_finished_at: Some(Timestamp::from(SystemTime::now())),
                }),
                jwt,
            }) {
                Ok(_) => {}
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    log::error!("task={task_id} failed to report task error: {_e}");
                }
            }
        }
    }
}
