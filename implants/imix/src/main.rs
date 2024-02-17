// #![windows_subsystem = "windows"]
#[cfg(all(feature = "win_service", windows))]
#[macro_use]
extern crate windows_service;

use imix::{handle_main,win_service::handle_service_main};

#[cfg(feature = "win_service")]
use windows_service::service_dispatcher;

#[cfg(all(feature = "win_service", not(target_os = "windows")))]
compile_error!("Feature win_service is only available on windows targets");

#[cfg(feature = "win_service")]
define_windows_service!(ffi_service_main, service_main);

#[tokio::main(flavor = "multi_thread", worker_threads = 128)]
async fn main() {
    #[cfg(feature = "win_service")]
    service_dispatcher::start("imix", ffi_service_main).unwrap();
    handle_main().await
}

#[cfg(feature = "win_service")]
#[tokio::main(flavor = "multi_thread", worker_threads = 128)]
async fn service_main(arguments: Vec<std::ffi::OsString>) {
    imix::win_service::handle_service_main(arguments);

    handle_main().await;
}

