use debugserver_types as ds;

use crate::dap::library::{events::*, requests::*, stream::*};

pub struct DapService {
    _private: (),
}

impl DapService {
    pub fn run<T: DebugServer>(f: impl FnOnce(Client) -> T) {
        server(f(Client::new()));
    }
}

fn server(dap: impl DebugServer) {
    log_begin();

    // Because of the eval we're running in, we probably can't see panics.
    // So mirror them to the log file.
    let orig_hook = std::panic::take_hook();
    std::panic::set_hook(box move |panic_info| {
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            log(&format!("Panic occurred: {:?}", s));
        } else {
            log("Panic occurred: Unknown message");
        }
        orig_hook(panic_info);
    });

    log("DEBUG ADAPTER STARTING");
    loop {
        let recv = read();
        let r: ds::Request = serde_json::from_value(recv).unwrap();
        assert_eq!(r.type_, "request");
        let resp = dispatch(&dap, &r);
        send(serde_json::to_value(resp).unwrap());

        if r.command == "disconnect" {
            break;
        }
    }
    log("DEBUG ADAPTER STOPPING");
}