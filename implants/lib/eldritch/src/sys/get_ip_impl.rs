use super::super::insert_dict_kv;
use anyhow::Result;
use local_ip_address::list_afinet_netifas;
use starlark::{
    collections::SmallMap,
    const_frozen_string,
    values::{dict::Dict, Heap, Value},
};
use std::net::IpAddr;

fn create_dict_from_interface(starlark_heap: &Heap, name: String, ip: IpAddr) -> Result<Dict<'_>> {
    let res: SmallMap<Value, Value> = SmallMap::new();
    let mut tmp_res = Dict::new(res);

    insert_dict_kv!(tmp_res, starlark_heap, "name", name, String);
    insert_dict_kv!(tmp_res, starlark_heap, "ip", ip.to_string(), String);

    Ok(tmp_res)
}

pub fn get_ip(starlark_heap: &Heap) -> Result<Vec<Dict<'_>>> {
    let network_interfaces = list_afinet_netifas()?;

    let mut final_res: Vec<Dict> = Vec::new();
    for (name, ip) in network_interfaces.iter() {
        let tmp_res = create_dict_from_interface(starlark_heap, name.clone(), *ip)?;
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
