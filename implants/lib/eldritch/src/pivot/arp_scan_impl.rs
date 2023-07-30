use anyhow::Result;
use ipnetwork::{IpNetwork, Ipv4Network};
use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{self, channel, NetworkInterface};
use pnet::packet::arp::{ArpOperations, ArpPacket, MutableArpPacket};
use pnet::packet::ethernet::MutableEthernetPacket;
use pnet::packet::ethernet::{EtherType, EthernetPacket};
use pnet::packet::Packet;
use pnet::util::MacAddr;
use starlark::collections::SmallMap;
use starlark::const_frozen_string;
use starlark::values::{dict::Dict, Heap};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
#[derive(Debug, Clone, PartialEq)]
pub struct ArpResponse {
    _source_ip: Ipv4Addr,
    source_mac: MacAddr,
    interface: String,
}

fn start_listener(
    interface: NetworkInterface,
    data: Arc<Mutex<HashMap<Ipv4Addr, Option<ArpResponse>>>>,
) {
    if interface.ips.iter().filter(|ip| ip.is_ipv4()).count() == 0 {
        return;
    }
    let mac = match interface.mac {
        Some(mac) => mac,
        None => return,
    };
    let (mut tx, mut rx) = match channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => {
            println!("Error creating channel: {}", e);
            return;
        }
    };
    let ips = match data.lock() {
        Ok(lock) => lock.keys().cloned().collect::<Vec<Ipv4Addr>>(),
        Err(err) => {
            println!("Failed to get lock on ips: {}", err);
            return;
        }
    };
    for ip in ips {
        let ip_to_use = match interface
            .ips
            .iter()
            .find(|interface_ip| interface_ip.contains(IpAddr::V4(ip)))
        {
            Some(IpNetwork::V4(ipnet)) => ipnet,
            Some(_) => continue,
            None => continue,
        };
        let mut arp_packet = [0u8; 28];
        let mut arp = match MutableArpPacket::new(&mut arp_packet) {
            Some(arp) => arp,
            None => {
                println!("Failed to create MutableArpPacket to send.");
                return;
            }
        };
        arp.set_hardware_type(pnet::packet::arp::ArpHardwareType(1));
        arp.set_protocol_type(pnet::packet::ethernet::EtherType(0x0800));
        arp.set_hw_addr_len(6);
        arp.set_proto_addr_len(4);
        arp.set_operation(ArpOperations::Request);
        arp.set_sender_hw_addr(mac);
        arp.set_sender_proto_addr(ip_to_use.ip());
        arp.set_target_hw_addr(MacAddr::zero());
        arp.set_target_proto_addr(ip);
        let mut eth_packet = [0u8; 60];
        let mut eth = match MutableEthernetPacket::new(&mut eth_packet) {
            Some(eth) => eth,
            None => {
                println!("Failed to create MutableEthernetPacket to send.");
                return;
            }
        };
        eth.set_destination(MacAddr::broadcast());
        eth.set_source(mac);
        eth.set_ethertype(EtherType(0x0806));
        eth.set_payload(arp.packet());
        match tx.send_to(eth.packet(), None) {
            Some(Ok(_)) => {}
            Some(Err(err)) => {
                println!("Failed to tx on {}: {}", interface.name, err);
                return;
            }
            None => {
                println!("Failed to tx on {}: Returned None", interface.name);
                return;
            }
        };
        let now = SystemTime::now();
        loop {
            let elapsed = match now.elapsed() {
                Ok(elapsed) => elapsed,
                Err(err) => {
                    println!("Failed to get elapsed time on {}: {}", interface.name, err);
                    return;
                }
            };
            if elapsed > Duration::from_secs(5) {
                break;
            }
            match rx.next() {
                Ok(packet) => {
                    let eth = match EthernetPacket::new(packet) {
                        Some(eth) => eth,
                        None => continue,
                    };
                    let arp = match ArpPacket::new(eth.payload()) {
                        Some(arp) => arp,
                        None => continue,
                    };
                    if arp.get_operation() != ArpOperations::Reply {
                        continue;
                    }
                    let source_ip = arp.get_sender_proto_addr();
                    let source_mac = arp.get_sender_hw_addr();
                    if source_ip == ip {
                        match data.lock() {
                            Ok(mut lock) => {
                                if let Some(target) = lock.get_mut(&ip) {
                                    *target = Some(ArpResponse {
                                        _source_ip: source_ip,
                                        source_mac,
                                        interface: interface.name.clone(),
                                    });
                                    break;
                                }
                                println!("Failed to find {} in HashMap", ip);
                                return;
                            }
                            Err(err) => {
                                println!("Failed to get lock on data: {}", err);
                                return;
                            }
                        }
                    }
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::TimedOut {
                        continue;
                    }
                    println!("Error receiving packet: {}", e);
                    return;
                }
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn handle_arp_scan(
    target_cidrs: Vec<String>,
) -> Result<HashMap<Ipv4Addr, Option<ArpResponse>>> {
    let listener_out: Arc<Mutex<HashMap<Ipv4Addr, Option<ArpResponse>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let target_cidrs = target_cidrs
        .iter()
        .map(|cidr| {
            let (addr, prefix) = cidr.split_at(
                cidr.find('/')
                    .ok_or(anyhow::anyhow!("Failed to find / in Network {}", cidr))?,
            );
            let addr = match Ipv4Addr::from_str(addr) {
                Ok(addr) => addr,
                Err(_) => {
                    return Err(anyhow::anyhow!("Invalid IPv4 address: {}", addr));
                }
            };
            let prefix: u8 = match prefix[1..].parse() {
                Ok(prefix) => prefix,
                Err(_) => {
                    return Err(anyhow::anyhow!("Invalid CIDR prefix: {}", prefix));
                }
            };
            let network = match Ipv4Network::new(addr, prefix) {
                Ok(network) => network,
                Err(_) => {
                    return Err(anyhow::anyhow!("Invalid CIDR: {}", cidr));
                }
            };
            Ok(network)
        })
        .collect::<Result<Vec<Ipv4Network>>>()?;
    for target_cidr in target_cidrs {
        for ip in target_cidr.iter() {
            match listener_out.lock() {
                Ok(mut listener_lock) => {
                    listener_lock.insert(ip, None);
                }
                Err(err) => return Err(anyhow::anyhow!("Failed to get lock on IP List: {}", err)),
            }
        }
    }
    let interfaces = datalink::interfaces();
    for interface in interfaces {
        let inner_out = listener_out.clone();
        let inner_interface = interface.clone();
        let thread = std::thread::spawn(move || {
            start_listener(inner_interface, inner_out);
        });
        thread.join().map_err(|err| {
            anyhow::anyhow!(
                "Failed to join thread for interface {}: {:?}",
                interface.name,
                err
            )
        })?
    }
    let out = listener_out
        .lock()
        .map_err(|err| anyhow::anyhow!("Failed to get final lock when returning results: {}", err))?
        .clone();
    Ok(out)
}

#[cfg(not(target_os = "windows"))]
pub fn arp_scan(starlark_heap: &Heap, target_cidrs: Vec<String>) -> Result<Vec<Dict>> {
    let mut out: Vec<Dict> = Vec::new();
    let final_listener_output = handle_arp_scan(target_cidrs)?;
    for (ipaddr, res) in final_listener_output {
        if let Some(res) = res {
            let hit_small_map = SmallMap::new();
            let mut hit_dict = Dict::new(hit_small_map);
            let ipaddr_value = starlark_heap.alloc_str(&ipaddr.to_string());
            let source_mac_value = starlark_heap.alloc_str(&res.source_mac.to_string());
            let interface_value = starlark_heap.alloc_str(&res.interface.to_string());
            hit_dict.insert_hashed(
                const_frozen_string!("ip").to_value().get_hashed()?,
                ipaddr_value.to_value(),
            );
            hit_dict.insert_hashed(
                const_frozen_string!("mac").to_value().get_hashed()?,
                source_mac_value.to_value(),
            );
            hit_dict.insert_hashed(
                const_frozen_string!("interface").to_value().get_hashed()?,
                interface_value.to_value(),
            );
            out.push(hit_dict);
        }
    }
    Ok(out)
}

#[cfg(target_os = "windows")]
pub fn arp_scan(starlark_heap: &Heap, target_cidrs: Vec<String>) -> Result<Vec<Dict>> {
    Err(anyhow::anyhow!("ARP Scanning is not available on Windows."))
}

#[cfg(target_os = "windows")]
#[cfg(test)]
mod tests {
    use super::arp_scan;

    #[test]
    fn test_windows_failure() {
        assert_eq!(
            arp_scan(starlark_heap, ["127.0.0.1/8".to_string()]),
            Err(anyhow::anyhow!("ARP Scanning is not available on Windows."))
        );
    }
}

#[cfg(not(target_os = "windows"))]
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::pivot::arp_scan_impl::handle_arp_scan;

    #[test]
    fn test_positive() {
        assert_eq!(
            handle_arp_scan(Vec::from(["127.0.0.1/32".to_string()])).unwrap(),
            HashMap::from([("127.0.0.1".parse().unwrap(), None)])
        );
    }

    #[test]
    fn test_no_slash() {
        assert!(handle_arp_scan(Vec::from(["127.0.0.1".to_string()])).is_err());
    }

    #[test]
    fn test_invalid_ipv4() {
        assert!(handle_arp_scan(Vec::from(["127.0.0.256".to_string()])).is_err());
    }

    #[test]
    fn test_invalid_cidr() {
        assert!(handle_arp_scan(Vec::from(["127.0.0.1/33".to_string()])).is_err());
    }
}
