use anyhow::Result;
use network_interface::{NetworkInterfaceConfig, NetworkInterface};
use starlark::values::{dict::Dict, Heap};

const UNKNOWN: &str = "UNKNOWN";

#[derive(Debug)]
struct NetInterface {
    name: String,
    ips: Vec<std::net::IpAddr>, //IPv6 and IPv4 Addresses on the itnerface
    mac: String,
}

fn handle_get_ip() -> Result<Vec<NetInterface>> {
    let mut res = Vec::new();
    for network_interface in NetworkInterface::show()? {

        let mac_addr = match network_interface.mac_addr {
            Some(local_mac) => local_mac,
            None => UNKNOWN.to_string(),
        };

        let mut ips: Vec<std::net::IpAddr> = Vec::new();
        for ip in network_interface.addr {
            ips.push(ip.ip());
        }
        
        res.push(NetInterface{
            name: network_interface.name,
            ips: ips,
            mac: mac_addr,
        });
    }
    Ok(res)
}

pub fn get_ip<'v>(starlark_heap: &Heap) -> Result<Vec<Dict<'v>>> {
    todo!();
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, IpAddr};

    use super::*;

    #[test]
    fn test_sys_get_ip() {
        let res = handle_get_ip().unwrap();
        for interface in res {
            if interface.name == "lo" {
                assert!(interface.ips.contains(&IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))));
            }
        }
    }
}