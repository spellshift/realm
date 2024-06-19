pub mod agent;
mod config;
mod install;
mod run;
mod task;
mod version;
#[cfg(feature = "win_service")]
pub mod win_service;

#[tokio::main(flavor = "multi_thread", worker_threads = 128)]
pub async fn lib_entry() {
    #[cfg(debug_assertions)]
    run::init_logging();

    #[cfg(feature = "win_service")]
    match windows_service::service_dispatcher::start("imix", ffi_service_main) {
        Ok(_) => {}
        Err(_err) => {
            #[cfg(debug_assertions)]
            log::error!("Failed to start service (running as exe?): {_err}");
        }
    }

    run::handle_main().await
}
