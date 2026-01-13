/*
 * Event callbacks that let eldritch functions run when the implant does certain tasks
 */
#[cfg(feature = "events")]
use std::sync::Arc;
#[cfg(feature = "events")]
use eldritchv2::{
    Interpreter,
    assets::std::{EmbeddedAssets, StdAssetsLibrary},
};

#[cfg(feature = "events")]
pub fn onevent(name: &str){
    // See if the event script exists
    let event_script_name = "event/".to_owned() + name + ".eldritch";
    let event_script = match crate::assets::Asset::get(&event_script_name) {
        Some(s) => s,
        None => return,
    };

    #[cfg(debug_assertions)]
    log::info!("Running event script '{}': {}", name, event_script_name);

    // Embedded assets are exposed to the callbacks
    let asset_backend = Arc::new(EmbeddedAssets::<crate::assets::Asset>::new());
    let mut locker = StdAssetsLibrary::new();
    let _ = locker.add(asset_backend);
    // Execute using Eldritch V2 Interpreter
    let mut interpreter = Interpreter::new().with_default_libs();
    interpreter.register_lib(locker);
    

    let content = String::from_utf8_lossy(&event_script.data).to_string();
    match interpreter.interpret(&content) {
        Ok(_) => {}
        Err(_e) => {
            #[cfg(debug_assertions)]
            log::error!("Failed to execute event script: {event_script_name}: {_e}");
        }
    }
}