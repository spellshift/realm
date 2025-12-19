use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::Path;

use super::{ConnectionState, NetstatEntry, SocketType};

pub fn netstat() -> Result<Vec<NetstatEntry>> {
    let mut entries = Vec::new();

    // Build inode to PID mapping first
    let inode_to_pid = build_inode_pid_map()?;

    // Parse TCP IPv4
    entries.extend(parse_proc_net_file(
        "/proc/net/tcp",
        SocketType::TCP,
        false,
        &inode_to_pid,
    )?);

    // Parse TCP IPv6
    entries.extend(parse_proc_net_file(
        "/proc/net/tcp6",
        SocketType::TCP,
        true,
        &inode_to_pid,
    )?);

    // Parse UDP IPv4
    entries.extend(parse_proc_net_file(
        "/proc/net/udp",
        SocketType::UDP,
        false,
        &inode_to_pid,
    )?);

    // Parse UDP IPv6
    entries.extend(parse_proc_net_file(
        "/proc/net/udp6",
        SocketType::UDP,
        true,
        &inode_to_pid,
    )?);

    Ok(entries)
}

fn parse_proc_net_file(
    path: &str,
    socket_type: SocketType,
    is_ipv6: bool,
    inode_to_pid: &HashMap<u64, u32>,
) -> Result<Vec<NetstatEntry>> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Failed to read {}: {}", path, e);
            return Ok(Vec::new());
        }
    };

    let mut entries = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        // Skip header line
        if line_num == 0 {
            continue;
        }

        match parse_proc_net_line(line, socket_type.clone(), is_ipv6, inode_to_pid) {
            Ok(Some(entry)) => entries.push(entry),
            Ok(None) => {} // Filtered or invalid entry
            Err(e) => {
                log::warn!("Failed to parse line in {}: {}", path, e);
                continue;
            }
        }
    }

    Ok(entries)
}

fn parse_proc_net_line(
    line: &str,
    socket_type: SocketType,
    is_ipv6: bool,
    inode_to_pid: &HashMap<u64, u32>,
) -> Result<Option<NetstatEntry>> {
    let fields: Vec<&str> = line.split_whitespace().collect();

    if fields.len() < 10 {
        return Ok(None);
    }

    // Parse local address and port
    let local_parts: Vec<&str> = fields[1].split(':').collect();
    if local_parts.len() != 2 {
        return Ok(None);
    }

    let local_address = match is_ipv6 {
        true => parse_hex_address_ipv6(local_parts[0]),
        false => parse_hex_address_ipv4(local_parts[0]),
    }?;

    let local_port = u16::from_str_radix(local_parts[1], 16)?;

    // Parse remote address and port
    let remote_parts: Vec<&str> = fields[2].split(':').collect();
    if remote_parts.len() != 2 {
        return Ok(None);
    }

    let remote_address = match is_ipv6 {
        true => parse_hex_address_ipv6(remote_parts[0]),
        false => parse_hex_address_ipv4(remote_parts[0]),
    }?;

    let remote_port = u16::from_str_radix(remote_parts[1], 16)?;

    // For listening sockets, remote address is 0.0.0.0 or ::
    let remote_address = if is_zero_address(&remote_address) {
        None
    } else {
        Some(remote_address)
    };

    // Parse connection state (field 3)
    let connection_state = if socket_type == SocketType::TCP {
        parse_connection_state(fields[3])?
    } else {
        // UDP has no connection state
        ConnectionState::Unknown
    };

    // Parse inode (field 9)
    let inode: u64 = fields[9].parse()?;

    // Lookup PID and process name
    let (pid, process_name) = if let Some(&pid) = inode_to_pid.get(&inode) {
        let proc_name = read_process_name(pid).ok();
        (pid, proc_name)
    } else {
        (0, None)
    };

    Ok(Some(NetstatEntry {
        socket_type,
        local_address,
        local_port,
        remote_address,
        remote_port,
        connection_state,
        pid,
        process_name,
    }))
}

fn parse_hex_address_ipv6(hex: &str) -> Result<IpAddr> {
    // IPv6 addresses are 32 hex characters (128 bits)
    if hex.len() != 32 {
        return Err(anyhow::anyhow!("Invalid IPv6 hex length: {}", hex.len()));
    }

    let mut bytes = [0u8; 16];
    for i in 0..16 {
        bytes[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)?;
    }

    // Linux stores IPv6 in network byte order but in 32-bit chunks with host endianness
    // We need to swap each 32-bit chunk
    let mut addr_bytes = [0u8; 16];
    for i in 0..4 {
        let chunk_start = i * 4;
        addr_bytes[chunk_start] = bytes[chunk_start + 3];
        addr_bytes[chunk_start + 1] = bytes[chunk_start + 2];
        addr_bytes[chunk_start + 2] = bytes[chunk_start + 1];
        addr_bytes[chunk_start + 3] = bytes[chunk_start];
    }

    Ok(IpAddr::V6(Ipv6Addr::from(addr_bytes)))
}

fn parse_hex_address_ipv4(hex: &str) -> Result<IpAddr> {
    // IPv4 addresses are 8 hex characters (32 bits)
    if hex.len() != 8 {
        return Err(anyhow::anyhow!("Invalid IPv4 hex length: {}", hex.len()));
    }

    let addr_u32 = u32::from_str_radix(hex, 16)?;
    // Linux stores IPv4 addresses in little-endian format
    let bytes = addr_u32.to_le_bytes();
    Ok(IpAddr::V4(Ipv4Addr::from(bytes)))
}

fn is_zero_address(addr: &IpAddr) -> bool {
    match addr {
        IpAddr::V4(ipv4) => ipv4.octets() == [0, 0, 0, 0],
        IpAddr::V6(ipv6) => ipv6.octets() == [0; 16],
    }
}

fn parse_connection_state(state_hex: &str) -> Result<ConnectionState> {
    // Linux TCP state values from include/net/tcp_states.h
    let state_val = u8::from_str_radix(state_hex, 16)?;

    Ok(match state_val {
        0x01 => ConnectionState::Established,
        0x02 => ConnectionState::SynSent,
        0x03 => ConnectionState::SynRecv,
        0x04 => ConnectionState::FinWait1,
        0x05 => ConnectionState::FinWait2,
        0x06 => ConnectionState::TimeWait,
        0x07 => ConnectionState::Close,
        0x08 => ConnectionState::CloseWait,
        0x09 => ConnectionState::LastAck,
        0x0A => ConnectionState::Listen,
        0x0B => ConnectionState::Closing,
        _ => ConnectionState::Unknown,
    })
}

fn build_inode_pid_map() -> Result<HashMap<u64, u32>> {
    let mut map = HashMap::new();

    let proc_dir = Path::new("/proc");
    let entries = match fs::read_dir(proc_dir) {
        Ok(e) => e,
        Err(e) => {
            log::warn!("Failed to read /proc directory: {}", e);
            return Ok(map);
        }
    };

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();

        // Check if this is a PID directory (numeric)
        if let Ok(pid) = name_str.parse::<u32>() {
            // Read the fd directory for this PID
            let fd_dir = proc_dir.join(name_str.as_ref()).join("fd");

            if let Ok(fd_entries) = fs::read_dir(fd_dir) {
                for fd_entry in fd_entries.flatten() {
                    // Read the symlink to see if it's a socket
                    if let Ok(link_target) = fs::read_link(fd_entry.path()) {
                        let target_str = link_target.to_string_lossy();

                        // Socket links look like: "socket:[12345]"
                        if let Some(inode_str) = target_str.strip_prefix("socket:[") {
                            if let Some(inode_str) = inode_str.strip_suffix(']') {
                                if let Ok(inode) = inode_str.parse::<u64>() {
                                    map.insert(inode, pid);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(map)
}

fn read_process_name(pid: u32) -> Result<String> {
    // Try /proc/[pid]/comm first (short name)
    let comm_path = format!("/proc/{}/comm", pid);
    if let Ok(name) = fs::read_to_string(&comm_path) {
        return Ok(name.trim().to_string());
    }

    // Fallback to cmdline
    let cmdline_path = format!("/proc/{}/cmdline", pid);
    if let Ok(cmdline) = fs::read_to_string(&cmdline_path) {
        // cmdline is null-separated, take first component
        if let Some(first) = cmdline.split('\0').next() {
            if !first.is_empty() {
                // Extract basename if it's a path
                if let Some(basename) = first.rsplit('/').next() {
                    return Ok(basename.to_string());
                }
                return Ok(first.to_string());
            }
        }
    }

    Err(anyhow::anyhow!(
        "Unable to read process name for PID {}",
        pid
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_address_ipv4() {
        // Test parsing 127.0.0.1 (0x0100007F in little-endian)
        let addr = parse_hex_address_ipv4("0100007F").unwrap();
        assert_eq!(addr, IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));

        // Test parsing 0.0.0.0
        let addr = parse_hex_address_ipv4("00000000").unwrap();
        assert_eq!(addr, IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)));

        // Test parsing 192.168.1.1 (0x0101A8C0 in little-endian)
        let addr = parse_hex_address_ipv4("0101A8C0").unwrap();
        assert_eq!(addr, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
    }

    #[test]
    fn test_parse_hex_address_ipv6() {
        // Test parsing :: (all zeros)
        let addr = parse_hex_address_ipv6("00000000000000000000000000000000").unwrap();
        assert_eq!(addr, IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)));
    }

    #[test]
    fn test_parse_connection_state() {
        assert_eq!(
            parse_connection_state("01").unwrap(),
            ConnectionState::Established
        );
        assert_eq!(
            parse_connection_state("02").unwrap(),
            ConnectionState::SynSent
        );
        assert_eq!(
            parse_connection_state("0A").unwrap(),
            ConnectionState::Listen
        );
    }

    #[test]
    fn test_is_zero_address() {
        assert!(is_zero_address(&IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))));
        assert!(!is_zero_address(&IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))));

        assert!(is_zero_address(&IpAddr::V6(Ipv6Addr::new(
            0, 0, 0, 0, 0, 0, 0, 0
        ))));
    }

    #[test]
    #[ignore] // Requires actual system with /proc
    fn test_netstat_integration() {
        let result = netstat();
        assert!(result.is_ok());

        let entries = result.unwrap();
        // Should have at least some network connections on a running system
        assert!(!entries.is_empty());

        // Verify all entries have valid data
        for entry in entries {
            // Verify socket type is set
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
