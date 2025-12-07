#[cfg(not(target_os = "windows"))]
use {
    super::super::insert_dict_kv,
    ipnetwork::{IpNetwork, Ipv4Network},
    pnet::{
        datalink::{self, channel, Channel::Ethernet, NetworkInterface},
        packet::{
            arp::{ArpOperations, ArpPacket, MutableArpPacket},
            ethernet::{EtherType, EthernetPacket, MutableEthernetPacket},
            Packet,
        },
        util::MacAddr,
    },
    starlark::collections::SmallMap,
    starlark::const_frozen_string,
    std::collections::HashMap,
    std::net::{IpAddr, Ipv4Addr},
    std::str::FromStr,
    std::sync::{Arc, Mutex},
    std::time::{Duration, SystemTime},
};

use anyhow::{anyhow, Result};
use starlark::values::{dict::Dict, Heap};

#[cfg(not(target_os = "windows"))]
#[derive(Debug, Clone, PartialEq)]
pub struct ArpResponse {
    _source_ip: Ipv4Addr,
    source_mac: MacAddr,
    interface: String,
}

#[cfg(not(target_os = "windows"))]
fn start_listener(
    interface: NetworkInterface,
    data: Arc<Mutex<HashMap<Ipv4Addr, Option<ArpResponse>>>>,
) -> Result<()> {
    use anyhow::Context;

    if interface.ips.iter().filter(|ip| ip.is_ipv4()).count() == 0 {
        return Err(anyhow!("Interface does not have a v4 address"));
    }
    let mac = interface.mac.context("Could not obtain MAC of interface")?;
    let (mut tx, mut rx) = match channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => return Err(anyhow!("Unhandled channel type")),
        Err(e) => {
            return Err(anyhow!("Error creating channel: {}", e));
        }
    };
    let ips = match data.lock() {
        Ok(lock) => lock.keys().cloned().collect::<Vec<Ipv4Addr>>(),
        Err(err) => {
            return Err(anyhow!("Failed to get lock on ips: {}", err));
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
                return Err(anyhow!("Failed to create MutableArpPacket to send."));
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
                return Err(anyhow!("Failed to create MutableEthernetPacket to send."));
            }
        };
        eth.set_destination(MacAddr::broadcast());
        eth.set_source(mac);
        eth.set_ethertype(EtherType(0x0806));
        eth.set_payload(arp.packet());
        match tx.send_to(eth.packet(), None) {
            Some(Ok(_)) => {}
            Some(Err(err)) => {
                return Err(anyhow!("Failed to tx on {}: {}", interface.name, err));
            }
            None => {
                return Err(anyhow!("Failed to tx on {}: Returned None", interface.name));
            }
        };
        let now = SystemTime::now();
        loop {
            let elapsed = match now.elapsed() {
                Ok(elapsed) => elapsed,
                Err(err) => {
                    return Err(anyhow!(
                        "Failed to get elapsed time on {}: {}",
                        interface.name,
                        err
                    ));
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
                                return Err(anyhow!("Failed to find {} in HashMap", ip));
                            }
                            Err(err) => {
                                return Err(anyhow!("Failed to get lock on data: {}", err));
                            }
                        }
                    }
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::TimedOut {
                        continue;
                    }
                    return Err(anyhow!("Error receiving packet: {}", e));
                }
            }
        }
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn handle_arp_scan(
    target_cidrs: Vec<String>,
) -> Result<HashMap<Ipv4Addr, Option<ArpResponse>>> {
    use anyhow::Context;

    let listener_out: Arc<Mutex<HashMap<Ipv4Addr, Option<ArpResponse>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let target_cidrs = target_cidrs
        .iter()
        .map(|cidr| {
            let (addr, prefix) = cidr.split_at(
                cidr.find('/')
                    .context(format!("Failed to find / in Network {}", cidr))?,
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
        let thread =
            std::thread::spawn(
                move || match start_listener(inner_interface.clone(), inner_out) {
                    Ok(_) => {}
                    Err(_err) => {
                        #[cfg(debug_assertions)]
                        log::error!("Listener on {} failed: {}", inner_interface.name, _err);
                    }
                },
            );
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
pub fn arp_scan(starlark_heap: &'_ Heap, target_cidrs: Vec<String>) -> Result<Vec<Dict<'_>>> {
    let mut out: Vec<Dict> = Vec::new();
    let final_listener_output = handle_arp_scan(target_cidrs)?;
    for (ipaddr, res) in final_listener_output {
        if let Some(res) = res {
            let hit_small_map = SmallMap::new();
            let mut hit_dict = Dict::new(hit_small_map);

            insert_dict_kv!(hit_dict, starlark_heap, "ip", &ipaddr.to_string(), String);
            insert_dict_kv!(
                hit_dict,
                starlark_heap,
                "mac",
                &res.source_mac.to_string(),
                String
            );
            insert_dict_kv!(
                hit_dict,
                starlark_heap,
                "interface",
                &res.interface.to_string(),
                String
            );
            out.push(hit_dict);
        }
    }
    Ok(out)
}

#[cfg(target_os = "windows")]
pub fn arp_scan(_starlark_heap: &'_ Heap, _target_cidrs: Vec<String>) -> Result<Vec<Dict<'_>>> {
    Err(anyhow!("ARP Scanning is not available on Windows."))
}

#[cfg(not(target_os = "windows"))]
#[cfg(test)]
mod tests {
    use pnet::datalink::interfaces;
    use std::{
        collections::HashMap,
        net::Ipv4Addr,
        sync::{Arc, Mutex},
        thread,
        time::Duration,
    };

    use crate::pivot::arp_scan_impl::{handle_arp_scan, start_listener, ArpResponse};

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

    #[test]
    fn test_lock_failure() {
        let data = Arc::from(Mutex::from(HashMap::<Ipv4Addr, Option<ArpResponse>>::from(
            [(Ipv4Addr::LOCALHOST, None)],
        )));
        let data_clone = data.clone();
        thread::spawn(move || {
            let _x = data_clone.lock().unwrap();
            panic!("Need to panic");
        });
        thread::sleep(Duration::from_secs(3));
        let loopback = {
            let interfaces = interfaces();
            interfaces.iter().find(|x| x.is_loopback()).unwrap().clone()
        };
        assert!(start_listener(loopback, data).is_err());
    }
}
