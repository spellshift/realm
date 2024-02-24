use anyhow::Result;
#[cfg(target_os = "windows")]
use network_interface::{NetworkInterface, NetworkInterfaceConfig};
#[cfg(not(target_os = "windows"))]
use pnet::datalink::{interfaces, NetworkInterface};

use super::super::insert_dict_kv;
use starlark::{
    collections::SmallMap,
    const_frozen_string,
    values::{dict::Dict, Heap, Value},
};

const UNKNOWN: &str = "UNKNOWN";

#[derive(Debug)]
#[cfg(target_os = "windows")]
struct NetInterface {
    name: String,
    ips: Vec<std::net::IpAddr>, //IPv6 and IPv4 Addresses on the itnerface
    mac: String,
}

#[cfg(target_os = "windows")]
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

        res.push(NetInterface {
            name: network_interface.name,
            ips: ips,
            mac: mac_addr,
        });
    }
    Ok(res)
}

#[cfg(not(target_os = "windows"))]
fn handle_get_ip() -> Result<Vec<NetworkInterface>> {
    Ok(interfaces())
}

#[cfg(target_os = "windows")]
fn create_dict_from_interface(starlark_heap: &Heap, interface: NetInterface) -> Result<Dict> {
    let res: SmallMap<Value, Value> = SmallMap::new();
    let mut tmp_res = Dict::new(res);

    insert_dict_kv!(tmp_res, starlark_heap, "name", &interface.name, String);

    let mut tmp_value2_arr = Vec::<Value>::new();
    for ip in interface.ips {
        tmp_value2_arr.push(
            starlark_heap
                .alloc_str(&ip.network().to_string())
                .to_value(),
        );
    }
    insert_dict_kv!(tmp_res, starlark_heap, "ips", tmp_value2_arr, Vec<_>);
    insert_dict_kv!(tmp_res, starlark_heap, "mac", &interface.mac, String);

    Ok(tmp_res)
}

#[cfg(not(target_os = "windows"))]
fn create_dict_from_interface(starlark_heap: &Heap, interface: NetworkInterface) -> Result<Dict> {
    let res: SmallMap<Value, Value> = SmallMap::new();
    let mut tmp_res = Dict::new(res);

    insert_dict_kv!(tmp_res, starlark_heap, "name", &interface.name, String);
    let mut tmp_value2_arr = Vec::<Value>::new();
    for ip in interface.ips {
        tmp_value2_arr.push(
            starlark_heap
                .alloc_str(&format!("{}/{}", ip.ip(), ip.prefix()))
                .to_value(),
        );
    }
    insert_dict_kv!(tmp_res, starlark_heap, "ips", tmp_value2_arr, Vec<_>);
    insert_dict_kv!(
        tmp_res,
        starlark_heap,
        "mac",
        &interface
            .mac
            .map(|mac| mac.to_string())
            .unwrap_or(UNKNOWN.to_string()),
        String
    );

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
        assert!(format!("{:?}", res).contains("127.0.0.1"));
    }
}
