use anyhow::Result;
use network_interface::{NetworkInterfaceConfig, NetworkInterface};
use eldritch_types::network_interface_type::NetworkInterface as NetInt;
const UNKNOWN: &str = "UNKNOWN";

fn handle_get_ip() -> Result<Vec<NetInt>> {
    let mut res = Vec::new();
    for network_interface in NetworkInterface::show()? {
        let name = network_interface.name;

        let mac = match network_interface.mac_addr {
            Some(local_mac) => local_mac,
            None => UNKNOWN.to_string(),
        };

        let mut ips: Vec<String> = Vec::new();
        for ip in network_interface.addr {
            ips.push(ip.ip().to_string());
        }
        
        res.push(NetInt{
            name,
            ips,
            mac,
        });
    }
    Ok(res)
}


pub fn get_ip() -> Result<Vec<NetInt>> {
    let mut final_res: Vec<NetInt> = Vec::new();
    for network_interface in handle_get_ip()? {
        final_res.push(network_interface);
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