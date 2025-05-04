#![deny(warnings)]

pub mod agent;
mod install;
mod run;
mod task;
mod version;

#[tokio::main(flavor = "multi_thread", worker_threads = 128)]
pub async fn lib_entry() {
    #[cfg(debug_assertions)]
    run::init_logging();

    run::handle_main().await
}
