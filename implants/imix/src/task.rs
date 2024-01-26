use anyhow::Result;
use c2::{
    pb::{ReportFileRequest, ReportProcessListRequest, ReportTaskOutputRequest, TaskOutput},
    TavernClient,
};
use eldritch::{channel::Receiver, Runtime};
use tokio::task::JoinHandle;

pub struct TaskHandle {
    pub id: i64,
    pub recv: Receiver,
    pub handle: JoinHandle<()>,
}

impl TaskHandle {
    pub async fn report(&self, tavern: &mut TavernClient) -> Result<()> {
        // Report Task Output
        let output = self.recv.collect_output()?;
        if output.len() > 0 {
            tavern
                .report_task_output(ReportTaskOutputRequest {
                    output: Some(TaskOutput {
                        id: self.id,
                        output: output.join(""),
                        error: None,
                        exec_started_at: None,
                        exec_finished_at: None,
                    }),
                })
                .await?;
        }

        // Report Process Lists
        let process_lists = self.recv.collect_process_lists()?;
        for list in process_lists {
            tavern
                .report_process_list(ReportProcessListRequest {
                    task_id: self.id,
                    list: Some(list),
                })
                .await?;
        }

        // Report Files TODO
        // let files = self.runtime.collect_files();
        // for f in files {
        //     tavern
        //         .report_file(ReportFileRequest {
        //             task_id: self.id,
        //             chunk: Some(f),
        //         })
        //         .await?;
        // }

        Ok(())
    }
}

// fn drain<T>(reciever: &Receiver<T>) -> Result<Vec<T>> {
//     let mut result: Vec<T> = Vec::new();
//     loop {
//         let val = match reciever.recv_timeout(Duration::from_millis(100)) {
//             Ok(v) => v,
//             Err(err) => {
//                 match err.to_string().as_str() {
//                     "channel is empty and sending half is closed" => {
//                         break;
//                     }
//                     "timed out waiting on channel" => {
//                         break;
//                     }
//                     _ => {
//                         #[cfg(debug_assertions)]
//                         eprint!("Error draining channel: {}", err)
//                     }
//                 }
//                 break;
//             }
//         };
//         // let appended_line = format!("{}{}", res.to_owned(), new_res_line);
//         result.push(val);
//     }
//     Ok(result)
// }
