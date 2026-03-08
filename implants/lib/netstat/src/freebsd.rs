use anyhow::Result;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use super::{ConnectionState, InterfaceEntry, NetstatEntry, SocketType};

pub fn netstat() -> Result<Vec<NetstatEntry>> {
    // TODO: Implement FreeBSD sysctl using net.inet.tcp.pcblist
    // For now, return empty list rather than error to allow basic functionality
    log::warn!("FreeBSD netstat implementation is not yet complete, returning empty results");
    Ok(Vec::new())
}

struct IfaceData {
    mac: [u8; 6],
    ipv4: Option<IpAddr>,
    ipv6: Option<IpAddr>,
}

pub fn list_interfaces() -> Result<Vec<InterfaceEntry>> {
    let mut ifaddrs_ptr: *mut libc::ifaddrs = std::ptr::null_mut();

    unsafe {
        if libc::getifaddrs(&mut ifaddrs_ptr) != 0 {
            return Err(anyhow::anyhow!(
                "getifaddrs failed: {}",
                std::io::Error::last_os_error()
            ));
        }
    }

    let mut ifaces: HashMap<String, IfaceData> = HashMap::new();

    unsafe {
        let mut cur = ifaddrs_ptr;
        while !cur.is_null() {
            let ifa = &*cur;

            if !ifa.ifa_addr.is_null() {
                let name = std::ffi::CStr::from_ptr(ifa.ifa_name)
                    .to_string_lossy()
                    .to_string();
                let family = (*ifa.ifa_addr).sa_family as i32;
                let entry = ifaces.entry(name).or_insert(IfaceData {
                    mac: [0u8; 6],
                    ipv4: None,
                    ipv6: None,
                });

                match family {
                    libc::AF_LINK => {
                        let sdl = &*(ifa.ifa_addr as *const libc::sockaddr_dl);
                        if sdl.sdl_alen == 6 {
                            let mac_offset = sdl.sdl_nlen as usize;
                            let data_ptr = sdl.sdl_data.as_ptr().add(mac_offset) as *const u8;
                            entry
                                .mac
                                .copy_from_slice(std::slice::from_raw_parts(data_ptr, 6));
                        }
                    }
                    libc::AF_INET => {
                        if entry.ipv4.is_none() {
                            let sin = &*(ifa.ifa_addr as *const libc::sockaddr_in);
                            let bytes = sin.sin_addr.s_addr.to_ne_bytes();
                            entry.ipv4 = Some(IpAddr::V4(Ipv4Addr::from(bytes)));
                        }
                    }
                    libc::AF_INET6 => {
                        if entry.ipv6.is_none() {
                            let sin6 = &*(ifa.ifa_addr as *const libc::sockaddr_in6);
                            entry.ipv6 = Some(IpAddr::V6(Ipv6Addr::from(sin6.sin6_addr.s6_addr)));
                        }
                    }
                    _ => {}
                }
            }

            cur = ifa.ifa_next;
        }

        libc::freeifaddrs(ifaddrs_ptr);
    }

    Ok(ifaces
        .into_iter()
        .map(|(name, data)| InterfaceEntry {
            iface_name: name,
            mac_address: data.mac,
            ip_address: data.ipv4.or(data.ipv6),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_netstat_returns_empty() {
        let result = netstat();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_list_interfaces() {
        let result = list_interfaces();
        assert!(result.is_ok());

        let interfaces = result.unwrap();
        assert!(!interfaces.is_empty(), "Should have at least one interface");
    }
}
