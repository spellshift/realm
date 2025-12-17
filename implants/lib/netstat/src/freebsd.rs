use anyhow::Result;
use super::{NetstatEntry, SocketType, ConnectionState};

pub fn netstat() -> Result<Vec<NetstatEntry>> {
    // TODO: Implement FreeBSD sysctl
    Err(anyhow::anyhow!("FreeBSD implementation not yet complete"))
}
