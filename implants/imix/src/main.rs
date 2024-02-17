// #![windows_subsystem = "windows"]
#[cfg(all(feature = "win_service", windows))]
#[macro_use]
extern crate windows_service;

use imix::handle_main;


// ============= Standard ===============

#[cfg(not(feature = "win_service"))]
#[tokio::main(flavor = "multi_thread", worker_threads = 128)]
async fn main() {

    handle_main().await
}


// ============ Windows Service =============

#[cfg(all(feature = "win_service", not(target_os = "windows")))]
compile_error!("Feature win_service is only available on windows targets");

#[cfg(feature = "win_service")]
define_windows_service!(ffi_service_main, service_main);

#[cfg(feature = "win_service")]
fn main() {
    use windows_service::service_dispatcher;
    service_dispatcher::start("imix", ffi_service_main).unwrap();
}

#[cfg(feature = "win_service")]
#[tokio::main(flavor = "multi_thread", worker_threads = 128)]
async fn service_main(arguments: Vec<std::ffi::OsString>) {
    use imix::win_service::handle_service_main;

    handle_service_main(arguments);

    handle_main().await;
}

