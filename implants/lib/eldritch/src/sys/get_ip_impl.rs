use anyhow::Result;
use network_interface::{NetworkInterfaceConfig, NetworkInterface};
use starlark::{values::{dict::Dict, Heap, Value}, collections::SmallMap, const_frozen_string};

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

fn create_dict_from_interface(starlark_heap: &Heap, interface: NetInterface) -> Result<Dict>{
    let res: SmallMap<Value, Value> = SmallMap::new();
    let mut tmp_res = Dict::new(res);

    let tmp_value1 = starlark_heap.alloc_str(&interface.name);
    tmp_res.insert_hashed(const_frozen_string!("name").to_value().get_hashed().unwrap(), tmp_value1.to_value());

    let mut tmp_value2_arr = Vec::<Value>::new();
    for ip in interface.ips {
        tmp_value2_arr.push(starlark_heap.alloc_str(&ip.to_string()).to_value());
    }
    let tmp_value2 = starlark_heap.alloc(tmp_value2_arr);
    tmp_res.insert_hashed(const_frozen_string!("ips").to_value().get_hashed().unwrap(), tmp_value2);

    let tmp_value3 = starlark_heap.alloc_str(&interface.mac);
    tmp_res.insert_hashed(const_frozen_string!("mac").to_value().get_hashed().unwrap(), tmp_value3.to_value());


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
    use std::net::{Ipv4Addr, IpAddr};

    use super::*;

    #[test]
    fn test_sys_get_ip() {
        let starlark_heap = Heap::new();
        let res = get_ip(&starlark_heap).unwrap();
        println!("{:?}", res);
        assert!(format!("{:?}", res).contains("127.0.0.1"));
    }
}