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
#[derive(Debug, Clone)]
struct ArpResponse {
    _source_ip: Ipv4Addr,
    source_mac: MacAddr,
    interface: String,
}

// use pnet to listen on given interface for ARP packets
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
    let ips = data
        .lock()
        .unwrap()
        .keys()
        .cloned()
        .collect::<Vec<Ipv4Addr>>();
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
        let mut arp = MutableArpPacket::new(&mut arp_packet).unwrap();
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
        let mut eth = MutableEthernetPacket::new(&mut eth_packet).unwrap();
        eth.set_destination(MacAddr::broadcast());
        eth.set_source(mac);
        eth.set_ethertype(EtherType(0x0806));
        eth.set_payload(arp.packet());
        let tx_res = tx.send_to(eth.packet(), None).unwrap();
        if let Err(err) = tx_res {
            return println!("Failed to tx: {} {}", interface.name, err);
        }
        let now = SystemTime::now();
        loop {
            if now.elapsed().unwrap() > Duration::from_secs(5) {
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
                        *data.lock().unwrap().get_mut(&ip).unwrap() = Some(ArpResponse {
                            _source_ip: source_ip,
                            source_mac,
                            interface: interface.name.clone(),
                        });
                        break;
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

pub fn arp_scan(starlark_heap: &Heap, target_cidrs: Vec<String>) -> Result<Vec<Dict>> {
    let target_cidrs = target_cidrs
        .iter()
        .map(|cidr| {
            let (addr, prefix) = cidr.split_at(cidr.find('/').unwrap());
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
        .collect::<Result<Vec<Ipv4Network>>>();
    match target_cidrs {
        Ok(target_cidrs) => {
            let listener_out: Arc<Mutex<HashMap<Ipv4Addr, Option<ArpResponse>>>> =
                Arc::new(Mutex::new(HashMap::new()));
            for target_cidr in target_cidrs {
                for ip in target_cidr.iter() {
                    listener_out.lock().unwrap().insert(ip, None);
                }
            }
            let interfaces = datalink::interfaces();
            for interface in interfaces {
                let inner_out = listener_out.clone();
                let interface = interface.clone();
                let thread = std::thread::spawn(move || {
                    start_listener(interface, inner_out);
                });
                thread.join().unwrap();
            }
            let mut out: Vec<Dict> = Vec::new();
            let final_listener_output = listener_out.lock().unwrap().clone();
            for (ipaddr, res) in final_listener_output {
                if let Some(res) = res {
                    let hit_small_map = SmallMap::new();
                    let mut hit_dict = Dict::new(hit_small_map);
                    let ipaddr_value = starlark_heap.alloc_str(&ipaddr.to_string());
                    let source_mac_value = starlark_heap.alloc_str(&res.source_mac.to_string());
                    let interface_value = starlark_heap.alloc_str(&res.interface.to_string());
                    hit_dict.insert_hashed(
                        const_frozen_string!("IP").to_value().get_hashed().unwrap(),
                        ipaddr_value.to_value(),
                    );
                    hit_dict.insert_hashed(
                        const_frozen_string!("MAC").to_value().get_hashed().unwrap(),
                        source_mac_value.to_value(),
                    );
                    hit_dict.insert_hashed(
                        const_frozen_string!("Interface")
                            .to_value()
                            .get_hashed()
                            .unwrap(),
                        interface_value.to_value(),
                    );
                    out.push(hit_dict);
                }
            }
            Ok(out)
        }
        Err(e) => Err(e),
    }
}
