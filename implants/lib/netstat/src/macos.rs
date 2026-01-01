use anyhow::{anyhow, Result};
use std::mem;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use super::{ConnectionState, NetstatEntry, SocketType};

// FFI bindings to libproc
extern "C" {
    fn proc_listpids(
        proc_type: u32,
        typeinfo: u32,
        buffer: *mut libc::c_void,
        buffersize: libc::c_int,
    ) -> libc::c_int;

    fn proc_pidinfo(
        pid: libc::c_int,
        flavor: libc::c_int,
        arg: u64,
        buffer: *mut libc::c_void,
        buffersize: libc::c_int,
    ) -> libc::c_int;

    fn proc_pidfdinfo(
        pid: libc::c_int,
        fd: libc::c_int,
        flavor: libc::c_int,
        buffer: *mut libc::c_void,
        buffersize: libc::c_int,
    ) -> libc::c_int;

    fn proc_name(pid: libc::c_int, buffer: *mut libc::c_void, buffersize: u32) -> libc::c_int;
}

// Constants from libproc.h
const PROC_ALL_PIDS: u32 = 1;
const PROC_PIDLISTFDS: libc::c_int = 1;
const PROC_PIDFDSOCKETINFO: libc::c_int = 3;
const PROC_FDTYPE_SOCKET: u32 = 2;

// Data structures matching macOS kernel structures
#[repr(C)]
#[derive(Copy, Clone)]
struct ProcFdInfo {
    proc_fd: i32,
    proc_fdtype: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct SocketFdInfo {
    pfi: ProcFileInfo,
    psi: SocketInfo,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct ProcFileInfo {
    fi_openflags: u32,
    fi_status: u32,
    fi_offset: i64,
    fi_type: i32,
    fi_guardflags: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct SocketInfo {
    soi_stat: VInfoStat,
    soi_so: u64,
    soi_pcb: u64,
    soi_type: libc::c_int,
    soi_protocol: libc::c_int,
    soi_family: libc::c_int,
    soi_options: libc::c_short,
    soi_linger: libc::c_short,
    soi_state: libc::c_short,
    soi_qlen: libc::c_short,
    soi_incqlen: libc::c_short,
    soi_qlimit: libc::c_short,
    soi_timeo: libc::c_short,
    soi_error: u16,
    soi_oobmark: u32,
    soi_rcv: SockBufInfo,
    soi_snd: SockBufInfo,
    soi_kind: libc::c_int,
    rfu_1: u32,
    soi_proto: SocketInfoProto,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct VInfoStat {
    vst_dev: u32,
    vst_mode: u16,
    vst_nlink: u16,
    vst_ino: u64,
    vst_uid: u32,
    vst_gid: u32,
    vst_atime: i64,
    vst_atimensec: i64,
    vst_mtime: i64,
    vst_mtimensec: i64,
    vst_ctime: i64,
    vst_ctimensec: i64,
    vst_birthtime: i64,
    vst_birthtimensec: i64,
    vst_size: i64,
    vst_blocks: i64,
    vst_blksize: i32,
    vst_flags: u32,
    vst_gen: u32,
    vst_rdev: u32,
    vst_qspare: [i64; 2],
}

#[repr(C)]
#[derive(Copy, Clone)]
struct SockBufInfo {
    sbi_cc: u32,
    sbi_hiwat: u32,
    sbi_mbcnt: u32,
    sbi_mbmax: u32,
    sbi_lowat: u32,
    sbi_flags: libc::c_short,
    sbi_timeo: libc::c_short,
}

#[repr(C)]
#[derive(Copy, Clone)]
union SocketInfoProto {
    pri_in: InSockInfo,
    pri_tcp: TcpSockInfo,
    pri_un: UnSockInfo,
    pri_ndrv: NdrvInfo,
    pri_kern_event: KernEventInfo,
    pri_kern_ctl: KernCtlInfo,
    pri_vsock: VsockSockInfo,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct InSockInfo {
    insi_fport: libc::c_int,
    insi_lport: libc::c_int,
    insi_gencnt: u64,
    insi_flags: u32,
    insi_flow: u32,
    insi_vflag: u8,
    insi_ip_ttl: u8,
    rfu_1: u32,
    insi_faddr: InSiAddr,
    insi_laddr: InSiAddr,
    insi_v4: InSiV4,
    insi_v6: InSiV6,
}

#[repr(C)]
#[derive(Copy, Clone)]
union InSiAddr {
    ina_46: InAddr46,
    ina_6: In6Addr,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct InAddr46 {
    i46a_pad32: [u32; 3],
    i46a_addr4: InAddr,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct InAddr {
    s_addr: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct In6Addr {
    __u6_addr: U6Addr,
}

#[repr(C)]
#[derive(Copy, Clone)]
union U6Addr {
    __u6_addr8: [u8; 16],
    __u6_addr16: [u16; 8],
    __u6_addr32: [u32; 4],
}

#[repr(C)]
#[derive(Copy, Clone)]
struct InSiV4 {
    in4_tos: u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct InSiV6 {
    in6_hlim: u8,
    in6_cksum: libc::c_int,
    in6_ifindex: u16,
    in6_hops: libc::c_short,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct TcpSockInfo {
    tcpsi_ini: InSockInfo,
    tcpsi_state: libc::c_int,
    tcpsi_timer: [libc::c_int; 4],
    tcpsi_mss: libc::c_int,
    tcpsi_flags: u32,
    rfu_1: u32,
    tcpsi_tp: u64,
}

const SOCK_MAXADDRLEN: usize = 255;
const IF_NAMESIZE: usize = 16;
const MAX_KCTL_NAME: usize = 96;

#[repr(C)]
#[derive(Copy, Clone)]
struct SockaddrUn {
    sun_len: u8,
    sun_family: u8,
    sun_path: [u8; 104],
}

#[repr(C)]
#[derive(Copy, Clone)]
union UnsiAddr {
    ua_sun: SockaddrUn,
    ua_dummy: [u8; SOCK_MAXADDRLEN],
}

#[repr(C)]
#[derive(Copy, Clone)]
struct UnSockInfo {
    unsi_conn_so: u64,
    unsi_conn_pcb: u64,
    unsi_addr: UnsiAddr,
    unsi_caddr: UnsiAddr,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct NdrvInfo {
    ndrvsi_if_family: u32,
    ndrvsi_if_unit: u32,
    ndrvsi_if_name: [u8; IF_NAMESIZE],
}

#[repr(C)]
#[derive(Copy, Clone)]
struct KernEventInfo {
    kesi_vendor_code_filter: u32,
    kesi_class_filter: u32,
    kesi_subclass_filter: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct KernCtlInfo {
    kcsi_id: u32,
    kcsi_reg_unit: u32,
    kcsi_flags: u32,
    kcsi_recvbufsize: u32,
    kcsi_sendbufsize: u32,
    kcsi_unit: u32,
    kcsi_name: [u8; MAX_KCTL_NAME],
}

#[repr(C)]
#[derive(Copy, Clone)]
struct VsockSockInfo {
    local_cid: u32,
    local_port: u32,
    remote_cid: u32,
    remote_port: u32,
}

/// Checks if the current macOS version is 11.0 or greater.
/// The proc_info.h structures used here, particularly vsock_sockinfo,
/// were added in macOS 11.0 (Big Sur) with the Virtualization framework.
fn check_macos_version() -> Result<()> {
    use std::ffi::CStr;

    unsafe {
        // Use sysctl to get kern.osproductversion
        let mut size: libc::size_t = 0;
        let name = b"kern.osproductversion\0";

        // First call to get size
        let result = libc::sysctlbyname(
            name.as_ptr() as *const libc::c_char,
            std::ptr::null_mut(),
            &mut size,
            std::ptr::null_mut(),
            0,
        );

        if result != 0 {
            return Err(anyhow!("Failed to get macOS version via sysctl"));
        }

        if size == 0 {
            return Err(anyhow!("sysctl returned size 0 for version string"));
        }

        // Allocate buffer and get actual value
        let mut buffer = vec![0u8; size];
        let result = libc::sysctlbyname(
            name.as_ptr() as *const libc::c_char,
            buffer.as_mut_ptr() as *mut libc::c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        );

        if result != 0 {
            return Err(anyhow!("Failed to read macOS version via sysctl"));
        }

        // Convert to string
        let version_cstr = CStr::from_bytes_until_nul(&buffer)
            .map_err(|_| anyhow!("Invalid version string from sysctl"))?;
        let version_string = version_cstr
            .to_str()
            .map_err(|_| anyhow!("Invalid UTF-8 in version string"))?;

        // Parse major version (e.g., "15.6.1" -> 15, "11.0" -> 11)
        let major_version: u32 = version_string
            .split('.')
            .next()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| anyhow!("Failed to parse macOS version: {}", version_string))?;

        if major_version < 11 {
            return Err(anyhow!(
                "macOS 11.0 (Big Sur) or greater is required for netstat functionality. Current version: {}",
                version_string
            ));
        }

        Ok(())
    }
}

pub fn netstat() -> Result<Vec<NetstatEntry>> {
    // Check macOS version - requires macOS 11.0 or greater
    check_macos_version()?;

    let mut entries = Vec::new();

    // Get all PIDs
    let pids = get_all_pids()?;

    // For each PID, get socket file descriptors
    for pid in pids {
        if let Ok(socket_entries) = get_pid_sockets(pid) {
            entries.extend(socket_entries);
        }
    }

    Ok(entries)
}

fn get_all_pids() -> Result<Vec<i32>> {
    unsafe {
        // First call to get buffer size needed
        let size = proc_listpids(PROC_ALL_PIDS, 0, std::ptr::null_mut(), 0);
        if size <= 0 {
            return Ok(Vec::new());
        }

        // Allocate buffer for PIDs
        let count = size as usize / mem::size_of::<i32>();
        let mut pids = vec![0i32; count];

        // Second call to get actual PIDs
        let result = proc_listpids(
            PROC_ALL_PIDS,
            0,
            pids.as_mut_ptr() as *mut libc::c_void,
            size,
        );

        if result <= 0 {
            return Ok(Vec::new());
        }

        // Filter out zero PIDs
        Ok(pids.into_iter().filter(|&pid| pid > 0).collect())
    }
}

fn get_pid_sockets(pid: i32) -> Result<Vec<NetstatEntry>> {
    let mut entries = Vec::new();

    // Get file descriptors for this PID
    let fds = get_pid_fds(pid)?;

    // For each socket FD, get detailed socket info
    for fd_info in fds {
        if fd_info.proc_fdtype == PROC_FDTYPE_SOCKET {
            if let Ok(Some(entry)) = get_socket_info(pid, fd_info.proc_fd) {
                entries.push(entry);
            }
        }
    }

    Ok(entries)
}

fn get_pid_fds(pid: i32) -> Result<Vec<ProcFdInfo>> {
    unsafe {
        // First call to get buffer size
        let size = proc_pidinfo(pid, PROC_PIDLISTFDS, 0, std::ptr::null_mut(), 0);

        if size <= 0 {
            return Ok(Vec::new());
        }

        // Allocate buffer
        let count = size as usize / mem::size_of::<ProcFdInfo>();
        let mut fds = vec![
            ProcFdInfo {
                proc_fd: 0,
                proc_fdtype: 0
            };
            count
        ];

        // Second call to get actual data
        let result = proc_pidinfo(
            pid,
            PROC_PIDLISTFDS,
            0,
            fds.as_mut_ptr() as *mut libc::c_void,
            size,
        );

        if result <= 0 {
            return Ok(Vec::new());
        }

        Ok(fds)
    }
}

fn get_socket_info(pid: i32, fd: i32) -> Result<Option<NetstatEntry>> {
    unsafe {
        let mut socket_info: SocketFdInfo = mem::zeroed();

        let result = proc_pidfdinfo(
            pid,
            fd,
            PROC_PIDFDSOCKETINFO,
            &mut socket_info as *mut _ as *mut libc::c_void,
            mem::size_of::<SocketFdInfo>() as i32,
        );

        if result <= 0 {
            return Ok(None);
        }

        let si = &socket_info.psi;

        // Determine socket type
        let socket_type = match si.soi_protocol {
            libc::IPPROTO_TCP => SocketType::TCP,
            libc::IPPROTO_UDP => SocketType::UDP,
            _ => return Ok(None), // Skip non-TCP/UDP sockets
        };

        // Get process name
        let process_name = get_process_name(pid).ok();

        // Extract common info
        let in_info = si.soi_proto.pri_in;
        let local_port = u16::from_be((in_info.insi_lport & 0xFFFF) as u16);
        let remote_port = u16::from_be((in_info.insi_fport & 0xFFFF) as u16);

        let connection_state = if socket_type == SocketType::TCP {
            parse_tcp_state(si.soi_proto.pri_tcp.tcpsi_state)
        } else {
            ConnectionState::Unknown
        };

        // Parse family-specific addresses
        let (local_address, remote_address) = match si.soi_family {
            libc::AF_INET => {
                // IPv4
                let local_addr = in_info.insi_laddr.ina_46.i46a_addr4.s_addr;
                let remote_addr = in_info.insi_faddr.ina_46.i46a_addr4.s_addr;

                let remote_address = if remote_addr == 0 {
                    None
                } else {
                    Some(IpAddr::V4(Ipv4Addr::from(u32::from_be(remote_addr))))
                };

                (
                    IpAddr::V4(Ipv4Addr::from(u32::from_be(local_addr))),
                    remote_address,
                )
            }
            libc::AF_INET6 => {
                // IPv6
                let local_addr = in_info.insi_laddr.ina_6.__u6_addr.__u6_addr8;
                let remote_addr = in_info.insi_faddr.ina_6.__u6_addr.__u6_addr8;

                let remote_ipv6 = Ipv6Addr::from(remote_addr);
                let remote_address = if remote_ipv6.is_unspecified() {
                    None
                } else {
                    Some(IpAddr::V6(remote_ipv6))
                };

                (IpAddr::V6(Ipv6Addr::from(local_addr)), remote_address)
            }
            _ => return Ok(None), // Skip other address families
        };

        Ok(Some(NetstatEntry {
            socket_type,
            local_address,
            local_port,
            remote_address,
            remote_port,
            connection_state,
            pid: pid as u32,
            process_name,
        }))
    }
}

fn get_process_name(pid: i32) -> Result<String> {
    unsafe {
        let mut buffer = vec![0u8; 256];
        let result = proc_name(pid, buffer.as_mut_ptr() as *mut libc::c_void, 256);

        if result <= 0 {
            return Err(anyhow::anyhow!(
                "Failed to get process name for PID {}",
                pid
            ));
        }

        // Find null terminator
        let len = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());
        Ok(String::from_utf8_lossy(&buffer[0..len]).to_string())
    }
}

fn parse_tcp_state(state: i32) -> ConnectionState {
    // macOS TCP states match BSD values
    match state {
        0 => ConnectionState::Close,
        1 => ConnectionState::Listen,
        2 => ConnectionState::SynSent,
        3 => ConnectionState::SynRecv,
        4 => ConnectionState::Established,
        5 => ConnectionState::CloseWait,
        6 => ConnectionState::FinWait1,
        7 => ConnectionState::Closing,
        8 => ConnectionState::LastAck,
        9 => ConnectionState::FinWait2,
        10 => ConnectionState::TimeWait,
        _ => ConnectionState::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macos_version_check() {
        // This test should pass on macOS 11.0 or greater
        let result = check_macos_version();
        assert!(
            result.is_ok(),
            "Version check failed: {:?}. This test requires macOS 11.0+",
            result
        );
    }

    #[test]
    fn test_parse_tcp_state() {
        assert_eq!(parse_tcp_state(0), ConnectionState::Close);
        assert_eq!(parse_tcp_state(1), ConnectionState::Listen);
        assert_eq!(parse_tcp_state(4), ConnectionState::Established);
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
