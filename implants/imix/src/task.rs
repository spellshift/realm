use anyhow::Result;
use pb::c2::{ReportTaskOutputRequest, TaskError, TaskOutput};
use transport::Transport;
use tokio::sync::mpsc;
use eldritch_core::Interpreter;
use crate::actions::ImixAction;
use crate::eldritch::{ACTION_SENDER, TASK_ID};
use crate::run::Config;

/*
 * Task handle is responsible for tracking a running task and reporting it's output.
 */
pub struct TaskHandle {
    id: i64,
    rx: mpsc::UnboundedReceiver<ImixAction>,
    pool: tokio::task::JoinSet<()>,
    interpreter_handle: Option<tokio::task::JoinHandle<()>>,
}

impl TaskHandle {
    // Track a new task handle and start execution.
    pub fn new(id: i64, tome: String) -> TaskHandle {
        let (tx, rx) = mpsc::unbounded_channel();
        let pool = tokio::task::JoinSet::new();

        // Spawn interpreter in a blocking task
        let interpreter_handle = tokio::task::spawn_blocking(move || {
            // Set task local context
            let _ = ACTION_SENDER.scope(tx, async move {
                let _ = TASK_ID.scope(id, async move {
                    // Create and run interpreter
                    let mut interpreter = Interpreter::new();

                    // We need to ensure global libraries are registered already.
                    // Assuming they are registered at startup.

                    // Execute tome
                    match interpreter.interpret(&tome) {
                        Ok(val) => {
                             #[cfg(debug_assertions)]
                             log::info!("Task {} finished with value: {:?}", id, val);
                             // We could report the return value as text if needed
                        }
                        Err(e) => {
                             #[cfg(debug_assertions)]
                             log::error!("Task {} failed: {}", id, e);

                             let action = ImixAction::ReportError(id, e);
                             let _ = ACTION_SENDER.try_with(|sender| {
                                 let _ = sender.send(action);
                             });
                        }
                    }
                }).await;
            });
        });

        TaskHandle {
            id,
            rx,
            pool,
            interpreter_handle: Some(interpreter_handle),
        }
    }

    // Returns true if the task has been completed, false otherwise.
    pub fn is_finished(&self) -> bool {
        // Check Report Pool
        if !self.pool.is_empty() {
            return false;
        }

        // Check Interpreter
        if let Some(handle) = &self.interpreter_handle {
             return handle.is_finished();
        }

        true
    }

    // Report any available task output.
    // Also responsible for downloading any files requested by the eldritch runtime.
    pub async fn report(
        &mut self,
        tavern: &mut (impl Transport + 'static),
        cfg: Config,
    ) -> Result<Config> {
        let mut ret_cfg = cfg.clone();

        // Drain channel
        while let Ok(msg) = self.rx.try_recv() {
            let id = self.id;
            let msg_debug = format!("{:?}", msg);

            // Dispatch in task pool
            let mut t = tavern.clone();
            let action = msg;
            let current_cfg = ret_cfg.clone(); // Clone for async block if needed

            // For now, let's run dispatch in the pool to mimic v1 async behavior.
            // But some actions might update config (Sync in v1).
            // ImixAction::SetConfig needs to be handled synchronously here to update `ret_cfg`.

            match action {
                ImixAction::SetConfig(new_cfg) => {
                    ret_cfg = new_cfg;
                    continue;
                }
                _ => {
                     // Async dispatch
                     self.pool.spawn(async move {
                        match action.dispatch(&mut t, current_cfg).await {
                            Ok(_) => {
                                #[cfg(debug_assertions)]
                                log::info!(
                                    "message success (task_id={},msg={})",
                                    id,
                                    msg_debug
                                );
                            }
                            Err(err) => {
                                #[cfg(debug_assertions)]
                                log::error!(
                                    "message failed (task_id={},msg={}): {}",
                                    id,
                                    msg_debug,
                                    err
                                );

                                // Report dispatch error
                                let _ = t.report_task_output(ReportTaskOutputRequest {
                                    output: Some(TaskOutput {
                                        id,
                                        output: String::new(),
                                        error: Some(TaskError {
                                            msg: format!(
                                                "dispatch error ({}): {:#?}",
                                                msg_debug, err
                                            ),
                                        }),
                                        exec_started_at: None,
                                        exec_finished_at: None,
                                    }),
                                }).await;
                            }
                        }
                     });
                }
            }
        }

        Ok(ret_cfg)
    }
}
