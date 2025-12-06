use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use anyhow::Result;
use local_ip_address::list_afinet_netifas;
use std::net::IpAddr;

fn create_dict_from_interface(name: String, ip: IpAddr) -> Result<BTreeMap<String, String>> {
    let mut tmp_res = BTreeMap::new();

    tmp_res.insert("name".to_string(), name);
    tmp_res.insert("ip".to_string(), ip.to_string());

    Ok(tmp_res)
}

pub fn get_ip() -> Result<Vec<BTreeMap<String, String>>> {
    let network_interfaces = list_afinet_netifas()?;

    let mut final_res: Vec<BTreeMap<String, String>> = Vec::new();
    for (name, ip) in network_interfaces.iter() {
        let tmp_res = create_dict_from_interface(name.clone(), *ip)?;
        final_res.push(tmp_res);
    }
    Ok(final_res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sys_get_ip() {
        let res = get_ip().unwrap();
        println!("{:?}", res);
        assert!(format!("{:?}", res).contains("127.0.0.1"));
    }
}
