use anyhow::Result;
use std::net::IpAddr;

#[cfg(target_os = "freebsd")]
mod freebsd;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SocketType {
    TCP,
    UDP,
}

impl std::fmt::Display for SocketType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SocketType::TCP => write!(f, "TCP"),
            SocketType::UDP => write!(f, "UDP"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    Listen,
    Established,
    SynSent,
    SynRecv,
    FinWait1,
    FinWait2,
    TimeWait,
    Close,
    CloseWait,
    LastAck,
    Closing,
    Unknown,
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionState::Listen => write!(f, "LISTEN"),
            ConnectionState::Established => write!(f, "ESTABLISHED"),
            ConnectionState::SynSent => write!(f, "SYN_SENT"),
            ConnectionState::SynRecv => write!(f, "SYN_RECV"),
            ConnectionState::FinWait1 => write!(f, "FIN_WAIT1"),
            ConnectionState::FinWait2 => write!(f, "FIN_WAIT2"),
            ConnectionState::TimeWait => write!(f, "TIME_WAIT"),
            ConnectionState::Close => write!(f, "CLOSE"),
            ConnectionState::CloseWait => write!(f, "CLOSE_WAIT"),
            ConnectionState::LastAck => write!(f, "LAST_ACK"),
            ConnectionState::Closing => write!(f, "CLOSING"),
            ConnectionState::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NetstatEntry {
    pub socket_type: SocketType,
    pub local_address: IpAddr,
    pub local_port: u16,
    pub remote_address: Option<IpAddr>,
    pub remote_port: u16,
    pub connection_state: ConnectionState,
    pub pid: u32,
    pub process_name: Option<String>,
}

/// Primary public API function to get all network connections
pub fn netstat() -> Result<Vec<NetstatEntry>> {
    #[cfg(target_os = "linux")]
    return linux::netstat();

    #[cfg(target_os = "macos")]
    return macos::netstat();

    #[cfg(target_os = "windows")]
    return windows::netstat();

    #[cfg(target_os = "freebsd")]
    return freebsd::netstat();

    #[cfg(not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "windows",
        target_os = "freebsd"
    )))]
    return Ok(Vec::new());
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceEntry {
    pub iface_name: String,
    pub mac_address: [u8; 6],
    pub ip_address: Option<IpAddr>,
}

pub fn list_interfaces() -> Result<Vec<InterfaceEntry>> {
    #[cfg(target_os = "linux")]
    return linux::list_interfaces();

    #[cfg(target_os = "macos")]
    return macos::list_interfaces();

    #[cfg(target_os = "windows")]
    return windows::list_interfaces();

    #[cfg(target_os = "freebsd")]
    return freebsd::list_interfaces();

    #[cfg(not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "windows",
        target_os = "freebsd"
    )))]
    return Ok(Vec::new());
}
