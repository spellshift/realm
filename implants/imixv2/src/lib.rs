extern crate alloc;

pub mod agent;
pub mod assets;
pub mod task;
pub mod run;
pub mod version;
pub mod shell;

#[unsafe(no_mangle)]
pub extern "C" fn lib_entry() {
    #[cfg(debug_assertions)]
    run::init_logger();

    // Create a runtime and block on the async function
    // We avoid #[tokio::main] on extern "C" function directly
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        let _ = run::run_agent().await;
    });
}
