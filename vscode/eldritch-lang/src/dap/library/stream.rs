//! Lowest level stream communication as JSON.
//! Because DAP debugging is hard, we write everything we see to stdout (for the protocol)
//! AND stderr (for debugging).

use std::{
    env,
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    path::PathBuf,
};

use serde_json::Value;

// Debugging anything through DAP is a nightmare, because VS Code doesn't surface any logs.
// Therefore, do the hacky thing of putting logs next to the binary.
fn log_file() -> PathBuf {
    let mut res = env::current_exe().unwrap();
    res.set_extension("dap.log");
    res
}

pub(crate) fn log_begin() {
    File::create(log_file()).unwrap();
}

pub(crate) fn log(x: &str) {
    let mut file = OpenOptions::new().append(true).open(log_file()).unwrap();
    file.write_all(format!("{}\n", x).as_bytes()).unwrap()
}

pub(crate) fn send(x: Value) {
    let s = x.to_string();
    log(&format!("SEND: {}", s));
    print!("Content-Length: {}\r\n\r\n{}", s.len(), s);
    io::stdout().flush().unwrap()
}

pub(crate) fn read() -> Value {
    let mut s = String::new();
    io::stdin().read_line(&mut s).unwrap();
    let len: usize = s
        .strip_prefix("Content-Length: ")
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    io::stdin().read_line(&mut s).unwrap();
    let mut res = vec![0u8; len];
    io::stdin().lock().read_exact(&mut res).unwrap();
    let s = String::from_utf8_lossy(&res);
    log(&format!("RECV: {}", s));
    serde_json::from_str(&s).unwrap()
}