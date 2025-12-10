#![cfg_attr(
    all(not(debug_assertions), not(feature = "win_service")),
    windows_subsystem = "windows"
)]
#![deny(warnings)]

#[cfg(all(feature = "win_service", windows))]
#[macro_use]
extern crate windows_service;
#[cfg(all(feature = "win_service", windows))]
mod win_service;

mod agent;
mod install;
mod run;
mod task;
mod version;
use run::handle_main;

// ============= Standard ===============

#[tokio::main(flavor = "multi_thread", worker_threads = 128)]
async fn main() {
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

    handle_main().await
}

// ============ Windows Service =============

#[cfg(all(feature = "win_service", not(target_os = "windows")))]
compile_error!("Feature win_service is only available on windows targets");

#[cfg(feature = "win_service")]
define_windows_service!(ffi_service_main, service_main);

#[cfg(feature = "win_service")]
#[tokio::main(flavor = "multi_thread", worker_threads = 128)]
async fn service_main(arguments: Vec<std::ffi::OsString>) {
    crate::win_service::handle_service_main(arguments);
    handle_main().await;
}
