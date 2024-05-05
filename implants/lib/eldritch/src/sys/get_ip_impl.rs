use std::net::IpAddr;

use anyhow::Result;
use network_interface::{NetworkInterface, NetworkInterfaceConfig};

use super::super::insert_dict_kv;
use starlark::{
    collections::SmallMap,
    const_frozen_string,
    values::{dict::Dict, Heap, Value},
};

const UNKNOWN: &str = "UNKNOWN";

struct NetInterface {
    name: String,
    ips: Vec<String>, //IPv6 and IPv4 Addresses on the interface
    mac: String,
}

fn netmask_to_cidr(netmask: IpAddr) -> Result<u8> {
    let binding = netmask.to_string();
    let mut cidr_prefix = 0;
    for octet in binding.split('.') {
        cidr_prefix += match octet.parse::<u8>() {
            Ok(x) => x.count_ones(),
            Err(_err) => {
                #[cfg(debug_assertions)]
                eprintln!("Failed to convert {} in netmask {}", octet, netmask);
                0
            }
        }
    }

    Ok(cidr_prefix as u8)
}

fn handle_get_ip() -> Result<Vec<NetInterface>> {
    let mut res = Vec::new();
    for network_interface in NetworkInterface::show()? {
        let mac = match network_interface.mac_addr {
            Some(local_mac) => local_mac,
            None => UNKNOWN.to_string(),
        };

        let name = network_interface.name;

        let mut ips: Vec<String> = Vec::new();
        for ip in network_interface.addr {
            if ip.ip().is_ipv4() {
                match ip.netmask() {
                    Some(netmask) => {
                        let cidr = netmask_to_cidr(netmask)?;
                        ips.push(format!("{}/{}", ip.ip(), cidr));
                    },
                    None => {
                        ips.push(ip.ip().to_string())
                    }
                }
            } else {
                ips.push(ip.ip().to_string())
            }
        }

        res.push(NetInterface { name, ips, mac });
    }
    Ok(res)
}

fn create_dict_from_interface(starlark_heap: &Heap, interface: NetInterface) -> Result<Dict> {
    let res: SmallMap<Value, Value> = SmallMap::new();
    let mut tmp_res = Dict::new(res);

    insert_dict_kv!(tmp_res, starlark_heap, "name", &interface.name, String);

    let mut tmp_value2_arr = Vec::<Value>::new();
    for ip in interface.ips {
        tmp_value2_arr.push(starlark_heap.alloc_str(&ip.to_string()).to_value());
    }
    insert_dict_kv!(tmp_res, starlark_heap, "ips", tmp_value2_arr, Vec<_>);
    insert_dict_kv!(tmp_res, starlark_heap, "mac", &interface.mac, String);

    Ok(tmp_res)
}

pub fn get_ip(starlark_heap: &Heap) -> Result<Vec<Dict>> {
    let mut final_res: Vec<Dict> = Vec::new();
    for network_interface in handle_get_ip()? {
        let tmp_res = create_dict_from_interface(starlark_heap, network_interface)?;
        final_res.push(tmp_res);
    }
    Ok(final_res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sys_get_ip() {
        let starlark_heap = Heap::new();
        let res = get_ip(&starlark_heap).unwrap();
        println!("{:?}", res);
        assert!(format!("{:?}", res).contains("127.0.0.1/8"));
    }
}
