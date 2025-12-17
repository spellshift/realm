use anyhow::Result;
use super::{NetstatEntry, SocketType, ConnectionState};

pub fn netstat() -> Result<Vec<NetstatEntry>> {
    // TODO: Implement macOS libproc FFI
    Err(anyhow::anyhow!("macOS implementation not yet complete"))
}
