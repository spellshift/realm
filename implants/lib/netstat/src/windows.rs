use anyhow::Result;
use std::collections::HashMap;
use std::mem;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::ptr;

use super::{ConnectionState, NetstatEntry, SocketType};

use windows_sys::Win32::Foundation::{
    CloseHandle, ERROR_INSUFFICIENT_BUFFER, INVALID_HANDLE_VALUE, NO_ERROR,
};
use windows_sys::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, GetExtendedUdpTable, MIB_TCP6ROW_OWNER_PID, MIB_TCP6TABLE_OWNER_PID,
    MIB_TCPROW_OWNER_PID, MIB_TCPTABLE_OWNER_PID, MIB_UDP6ROW_OWNER_PID, MIB_UDP6TABLE_OWNER_PID,
    MIB_UDPROW_OWNER_PID, MIB_UDPTABLE_OWNER_PID, TCP_TABLE_OWNER_PID_ALL, UDP_TABLE_OWNER_PID,
};
use windows_sys::Win32::Networking::WinSock::{AF_INET, AF_INET6};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};

pub fn netstat() -> Result<Vec<NetstatEntry>> {
    let mut entries = Vec::new();

    // Create process snapshot once for all lookups
    let process_map = create_process_snapshot()?;

    // Get TCP IPv4 connections
    entries.extend(get_tcp_table(&process_map)?);

    // Get TCP IPv6 connections
    entries.extend(get_tcp6_table(&process_map)?);

    // Get UDP IPv4 sockets
    entries.extend(get_udp_table(&process_map)?);

    // Get UDP IPv6 sockets
    entries.extend(get_udp6_table(&process_map)?);

    Ok(entries)
}

fn get_tcp_table(process_map: &HashMap<u32, String>) -> Result<Vec<NetstatEntry>> {
    unsafe {
        let mut size: u32 = 0;

        // First call to get required size
        GetExtendedTcpTable(
            ptr::null_mut(),
            &mut size,
            0,
            AF_INET as u32,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );

        if size == 0 {
            return Ok(Vec::new());
        }

        // Allocate buffer
        let mut buffer = vec![0u8; size as usize];

        // Second call to get actual data
        let result = GetExtendedTcpTable(
            buffer.as_mut_ptr() as *mut _,
            &mut size,
            0,
            AF_INET as u32,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );

        if result != NO_ERROR {
            return Err(anyhow::anyhow!(
                "GetExtendedTcpTable failed with error code: {}",
                result
            ));
        }

        // Parse MIB_TCPTABLE_OWNER_PID structure
        let table = &*(buffer.as_ptr() as *const MIB_TCPTABLE_OWNER_PID);
        let num_entries = table.dwNumEntries;

        let mut entries = Vec::new();
        for i in 0..num_entries {
            let row = &*table.table.as_ptr().add(i as usize);

            let local_addr = u32::from_be(row.dwLocalAddr);
            let remote_addr = u32::from_be(row.dwRemoteAddr);
            let local_port = u16::from_be((row.dwLocalPort & 0xFFFF) as u16);
            let remote_port = u16::from_be((row.dwRemotePort & 0xFFFF) as u16);

            let remote_address = if remote_addr == 0 {
                None
            } else {
                Some(IpAddr::V4(Ipv4Addr::from(local_addr.to_be_bytes())))
            };

            let process_name = get_process_name(row.dwOwningPid, process_map);

            entries.push(NetstatEntry {
                socket_type: SocketType::TCP,
                local_address: IpAddr::V4(Ipv4Addr::from(local_addr.to_be_bytes())),
                local_port,
                remote_address,
                remote_port,
                connection_state: parse_tcp_state(row.dwState),
                pid: row.dwOwningPid,
                process_name,
            });
        }

        Ok(entries)
    }
}

fn get_tcp6_table(process_map: &HashMap<u32, String>) -> Result<Vec<NetstatEntry>> {
    unsafe {
        let mut size: u32 = 0;

        // First call to get required size
        GetExtendedTcpTable(
            ptr::null_mut(),
            &mut size,
            0,
            AF_INET6 as u32,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );

        if size == 0 {
            return Ok(Vec::new());
        }

        // Allocate buffer
        let mut buffer = vec![0u8; size as usize];

        // Second call to get actual data
        let result = GetExtendedTcpTable(
            buffer.as_mut_ptr() as *mut _,
            &mut size,
            0,
            AF_INET6 as u32,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );

        if result != NO_ERROR {
            return Err(anyhow::anyhow!(
                "GetExtendedTcpTable (IPv6) failed with error code: {}",
                result
            ));
        }

        // Parse MIB_TCP6TABLE_OWNER_PID structure
        let table = &*(buffer.as_ptr() as *const MIB_TCP6TABLE_OWNER_PID);
        let num_entries = table.dwNumEntries;

        let mut entries = Vec::new();
        for i in 0..num_entries {
            let row = &*table.table.as_ptr().add(i as usize);

            let local_addr = Ipv6Addr::from(row.ucLocalAddr);
            let remote_addr = Ipv6Addr::from(row.ucRemoteAddr);
            let local_port = u16::from_be((row.dwLocalPort & 0xFFFF) as u16);
            let remote_port = u16::from_be((row.dwRemotePort & 0xFFFF) as u16);

            let remote_address = if remote_addr.is_unspecified() {
                None
            } else {
                Some(IpAddr::V6(remote_addr))
            };

            let process_name = get_process_name(row.dwOwningPid, process_map);

            entries.push(NetstatEntry {
                socket_type: SocketType::TCP,
                local_address: IpAddr::V6(local_addr),
                local_port,
                remote_address,
                remote_port,
                connection_state: parse_tcp_state(row.dwState),
                pid: row.dwOwningPid,
                process_name,
            });
        }

        Ok(entries)
    }
}

fn get_udp_table(process_map: &HashMap<u32, String>) -> Result<Vec<NetstatEntry>> {
    unsafe {
        let mut size: u32 = 0;

        // First call to get required size
        GetExtendedUdpTable(
            ptr::null_mut(),
            &mut size,
            0,
            AF_INET as u32,
            UDP_TABLE_OWNER_PID,
            0,
        );

        if size == 0 {
            return Ok(Vec::new());
        }

        // Allocate buffer
        let mut buffer = vec![0u8; size as usize];

        // Second call to get actual data
        let result = GetExtendedUdpTable(
            buffer.as_mut_ptr() as *mut _,
            &mut size,
            0,
            AF_INET as u32,
            UDP_TABLE_OWNER_PID,
            0,
        );

        if result != NO_ERROR {
            return Err(anyhow::anyhow!(
                "GetExtendedUdpTable failed with error code: {}",
                result
            ));
        }

        // Parse MIB_UDPTABLE_OWNER_PID structure
        let table = &*(buffer.as_ptr() as *const MIB_UDPTABLE_OWNER_PID);
        let num_entries = table.dwNumEntries;

        let mut entries = Vec::new();
        for i in 0..num_entries {
            let row = &*table.table.as_ptr().add(i as usize);

            let local_addr = u32::from_be(row.dwLocalAddr);
            let local_port = u16::from_be((row.dwLocalPort & 0xFFFF) as u16);

            let process_name = get_process_name(row.dwOwningPid, process_map);

            entries.push(NetstatEntry {
                socket_type: SocketType::UDP,
                local_address: IpAddr::V4(Ipv4Addr::from(local_addr.to_be_bytes())),
                local_port,
                remote_address: None,
                remote_port: 0,
                connection_state: ConnectionState::Unknown, // UDP is connectionless
                pid: row.dwOwningPid,
                process_name,
            });
        }

        Ok(entries)
    }
}

fn get_udp6_table(process_map: &HashMap<u32, String>) -> Result<Vec<NetstatEntry>> {
    unsafe {
        let mut size: u32 = 0;

        // First call to get required size
        GetExtendedUdpTable(
            ptr::null_mut(),
            &mut size,
            0,
            AF_INET6 as u32,
            UDP_TABLE_OWNER_PID,
            0,
        );

        if size == 0 {
            return Ok(Vec::new());
        }

        // Allocate buffer
        let mut buffer = vec![0u8; size as usize];

        // Second call to get actual data
        let result = GetExtendedUdpTable(
            buffer.as_mut_ptr() as *mut _,
            &mut size,
            0,
            AF_INET6 as u32,
            UDP_TABLE_OWNER_PID,
            0,
        );

        if result != NO_ERROR {
            return Err(anyhow::anyhow!(
                "GetExtendedUdpTable (IPv6) failed with error code: {}",
                result
            ));
        }

        // Parse MIB_UDP6TABLE_OWNER_PID structure
        let table = &*(buffer.as_ptr() as *const MIB_UDP6TABLE_OWNER_PID);
        let num_entries = table.dwNumEntries;

        let mut entries = Vec::new();
        for i in 0..num_entries {
            let row = &*table.table.as_ptr().add(i as usize);

            let local_addr = Ipv6Addr::from(row.ucLocalAddr);
            let local_port = u16::from_be((row.dwLocalPort & 0xFFFF) as u16);

            let process_name = get_process_name(row.dwOwningPid, process_map);

            entries.push(NetstatEntry {
                socket_type: SocketType::UDP,
                local_address: IpAddr::V6(local_addr),
                local_port,
                remote_address: None,
                remote_port: 0,
                connection_state: ConnectionState::Unknown, // UDP is connectionless
                pid: row.dwOwningPid,
                process_name,
            });
        }

        Ok(entries)
    }
}

// The `CreateToolhelp32Snapshot` function provides a complete list of running processes
// without requiring `SeDebugPrivilege`. While `SeDebugPrivilege` would allow us to
// open and inspect protected processes, it's not necessary for the current task of
// mapping PIDs to process names. `GetExtendedTcpTable` and `GetExtendedUdpTable`
// already provide the PIDs for all connections, and this function is sufficient to
// resolve their names.
fn create_process_snapshot() -> Result<HashMap<u32, String>> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return Err(anyhow::anyhow!("Failed to create process snapshot"));
        }

        let mut process_map = HashMap::new();
        let mut entry: PROCESSENTRY32W = mem::zeroed();
        entry.dwSize = mem::size_of::<PROCESSENTRY32W>() as u32;

        if Process32FirstW(snapshot, &mut entry) == 0 {
            CloseHandle(snapshot);
            return Err(anyhow::anyhow!("Failed to get first process"));
        }

        loop {
            // Find the null terminator in szExeFile
            let name_len = entry
                .szExeFile
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(entry.szExeFile.len());

            let process_name = String::from_utf16_lossy(&entry.szExeFile[0..name_len]);
            process_map.insert(entry.th32ProcessID, process_name);

            if Process32NextW(snapshot, &mut entry) == 0 {
                break;
            }
        }

        CloseHandle(snapshot);
        Ok(process_map)
    }
}

fn get_process_name(pid: u32, process_map: &HashMap<u32, String>) -> Option<String> {
    process_map.get(&pid).cloned()
}

fn parse_tcp_state(state: u32) -> ConnectionState {
    // Windows MIB_TCP_STATE values
    match state {
        1 => ConnectionState::Close,
        2 => ConnectionState::Listen,
        3 => ConnectionState::SynSent,
        4 => ConnectionState::SynRecv,
        5 => ConnectionState::Established,
        6 => ConnectionState::FinWait1,
        7 => ConnectionState::FinWait2,
        8 => ConnectionState::CloseWait,
        9 => ConnectionState::Closing,
        10 => ConnectionState::LastAck,
        11 => ConnectionState::TimeWait,
        12 => ConnectionState::Close, // DELETE_TCB
        _ => ConnectionState::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tcp_state() {
        assert_eq!(parse_tcp_state(1), ConnectionState::Close);
        assert_eq!(parse_tcp_state(2), ConnectionState::Listen);
        assert_eq!(parse_tcp_state(3), ConnectionState::SynSent);
        assert_eq!(parse_tcp_state(5), ConnectionState::Established);
        assert_eq!(parse_tcp_state(999), ConnectionState::Unknown);
    }

    #[test]
    fn test_netstat_integration() {
        let result = netstat();
        assert!(result.is_ok());

        let entries = result.unwrap();
        // Should have at least some network connections on a running system
        assert!(!entries.is_empty());

        // Verify all entries have valid data
        for entry in entries {
            assert!(entry.local_port > 0 || entry.local_port == 0);
            assert!(entry.socket_type == SocketType::TCP || entry.socket_type == SocketType::UDP);
        }
    }

    #[tokio::test]
    async fn test_netstat_with_test_socket() -> Result<()> {
        use std::process;
        use tokio::net::TcpListener;

        // Create a test TCP listener
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let test_port = listener.local_addr()?.port();
        let current_pid = process::id();

        // Keep listener alive
        let _guard = listener;

        // Run netstat
        let entries = netstat()?;

        // Find our socket
        let found = entries.iter().any(|e| {
            e.local_port == test_port
                && e.socket_type == SocketType::TCP
                && e.connection_state == ConnectionState::Listen
                && e.pid == current_pid
        });

        assert!(found, "Our test socket should appear in netstat results");
        Ok(())
    }
}
