#[cfg(not(target_os = "windows"))]
use {
    alloc::string::ToString,
    ipnetwork::{IpNetwork, Ipv4Network},
    pnet::{
        datalink::{self, Channel::Ethernet, NetworkInterface, channel},
        packet::{
            Packet,
            arp::{ArpOperations, ArpPacket, MutableArpPacket},
            ethernet::{EtherType, EthernetPacket, MutableEthernetPacket},
        },
        util::MacAddr,
    },
    std::collections::HashMap,
    std::net::{IpAddr, Ipv4Addr},
    std::str::FromStr,
    std::sync::{Arc, Mutex},
    std::time::{Duration, SystemTime},
};

use crate::std::StdPivotLibrary;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use anyhow::{Result, anyhow};
use eldritch_core::Value;

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
            return Err(anyhow!("Error creating channel: {e}"));
        }
    };
    let ips = match data.lock() {
        Ok(lock) => lock.keys().cloned().collect::<Vec<Ipv4Addr>>(),
        Err(err) => {
            return Err(anyhow!("Failed to get lock on ips: {err}"));
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
                                return Err(anyhow!("Failed to find {ip} in HashMap"));
                            }
                            Err(err) => {
                                return Err(anyhow!("Failed to get lock on data: {err}"));
                            }
                        }
                    }
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::TimedOut {
                        continue;
                    }
                    return Err(anyhow!("Error receiving packet: {e}"));
                }
            }
        }
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn parse_cidrs_for_arp_scan(target_cidrs: Vec<String>) -> Result<Vec<Ipv4Network>> {
    use anyhow::Context;
    target_cidrs
        .iter()
        .map(|cidr| {
            let (addr, prefix) = cidr.split_at(
                cidr.find('/')
                    .context(format!("Failed to find / in Network {cidr}"))?,
            );
            let addr = match Ipv4Addr::from_str(addr) {
                Ok(addr) => addr,
                Err(_) => {
                    return Err(anyhow::anyhow!("Invalid IPv4 address: {addr}"));
                }
            };
            let prefix: u8 = match prefix[1..].parse() {
                Ok(prefix) => prefix,
                Err(_) => {
                    return Err(anyhow::anyhow!("Invalid CIDR prefix: {prefix}"));
                }
            };
            let network = match Ipv4Network::new(addr, prefix) {
                Ok(network) => network,
                Err(_) => {
                    return Err(anyhow::anyhow!("Invalid CIDR: {cidr}"));
                }
            };
            Ok(network)
        })
        .collect::<Result<Vec<Ipv4Network>>>()
}

#[cfg(not(target_os = "windows"))]
pub fn handle_arp_scan(
    target_cidrs: Vec<String>,
) -> Result<HashMap<Ipv4Addr, Option<ArpResponse>>> {
    let listener_out: Arc<Mutex<HashMap<Ipv4Addr, Option<ArpResponse>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let target_networks = parse_cidrs_for_arp_scan(target_cidrs)?;

    for target_cidr in target_networks {
        for ip in target_cidr.iter() {
            match listener_out.lock() {
                Ok(mut listener_lock) => {
                    listener_lock.insert(ip, None);
                }
                Err(err) => return Err(anyhow::anyhow!("Failed to get lock on IP List: {err}")),
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
        .map_err(|err| anyhow::anyhow!("Failed to get final lock when returning results: {err}"))?
        .clone();
    Ok(out)
}

#[cfg(not(target_os = "windows"))]
pub fn run(
    lib: &StdPivotLibrary,
    target_cidrs: Vec<String>,
) -> Result<Vec<BTreeMap<String, Value>>, String> {
    let (tx, rx) = std::sync::mpsc::channel();
    let target_cidrs_clone = target_cidrs.clone();

    let fut = async move {
        // Use spawn_blocking for blocking operation
        let res = tokio::task::spawn_blocking(move || {
            handle_arp_scan(target_cidrs_clone)
        }).await;

        let inner_res = match res {
            Ok(r) => r,
            Err(e) => Err(anyhow!("Task join error: {}", e)),
        };

        let _ = tx.send(inner_res);
    };

    lib.agent
        .spawn_subtask(lib.task_id, "arp_scan".to_string(), alloc::boxed::Box::pin(fut))
        .map_err(|e| e.to_string())?;

    let response = rx.recv().map_err(|e| format!("Failed to receive result: {}", e))?;

    match response {
        Ok(final_listener_output) => {
            let mut out = Vec::new();
            for (ipaddr, res) in final_listener_output {
                if let Some(res) = res {
                    let mut hit_dict = BTreeMap::new();
                    hit_dict.insert("ip".into(), Value::String(ipaddr.to_string()));
                    hit_dict.insert("mac".into(), Value::String(res.source_mac.to_string()));
                    hit_dict.insert("interface".into(), Value::String(res.interface.to_string()));
                    out.push(hit_dict);
                }
            }
            Ok(out)
        }
        Err(err) => Err(format!("ARP Scan failed: {:?}", err)),
    }
}

#[cfg(target_os = "windows")]
pub fn run(
    _lib: &StdPivotLibrary,
    _target_cidrs: Vec<String>,
) -> Result<Vec<BTreeMap<String, Value>>, String> {
    Err("ARP Scanning is not available on Windows.".to_string())
}
