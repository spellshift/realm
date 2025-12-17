use anyhow::Result;
use super::{NetstatEntry, SocketType, ConnectionState};

pub fn netstat() -> Result<Vec<NetstatEntry>> {
    // TODO: Implement Windows API
    Err(anyhow::anyhow!("Windows implementation not yet complete"))
}
