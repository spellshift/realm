use std::collections::HashMap;
#[cfg(target_os = "linux")]
use procfs::{
    process::*,
    net::*
};
use starlark::{values::{dict::Dict, Heap, Value}, collections::SmallMap, const_frozen_string};
use anyhow::Result;

#[derive(Debug)]
enum SocketData {
    TCP(TcpNetEntry),
    UDP(UdpNetEntry),
    Unix(UnixNetEntry)
}

#[cfg(not(target_os = "linux"))]
pub fn netstat(starlark_heap: &Heap) -> Result<Vec<Dict>> {
    let map: SmallMap<Value, Value> = SmallMap::new();
    // Create Dict type.
    let mut dict = Dict::new(map);
    dict.insert_hashed(const_frozen_string!("err").to_value().get_hashed()?, starlark_heap.alloc_str("Not implemented").to_value());
    Ok(Vec::from([dict]))
}

#[cfg(target_os = "linux")]
pub fn netstat(starlark_heap: &Heap) -> Result<Vec<Dict>> {
    let mut out: Vec<Dict> = Vec::new();
    let all_procs = procfs::process::all_processes().unwrap();

    // build up a map between socket inodes and process stat info:
    let mut map: HashMap<u64, Stat> = HashMap::new();
    for p in all_procs {
        let process = p.unwrap();
        if let (Ok(stat), Ok(fds)) = (process.stat(), process.fd()) {
            for fd in fds {
                if let FDTarget::Socket(inode) = fd.unwrap().target {
                    map.insert(inode, stat.clone());
                }
            }
        }
    }

    // get the tcp table
    let tcp = procfs::net::tcp().unwrap();
    let tcp_new = tcp.iter().map(|x| (SocketData::TCP(x.clone()), map.get(&x.inode).cloned()));
    let tcp6 = procfs::net::tcp6().unwrap();
    let tcp6_new = tcp6.iter().map(|x| (SocketData::TCP(x.clone()), map.get(&x.inode).cloned()));
    let udp = procfs::net::udp().unwrap();
    let udp_new = udp.iter().map(|x| (SocketData::UDP(x.clone()), map.get(&x.inode).cloned()));
    let udp6 = procfs::net::udp().unwrap();
    let udp6_new = udp6.iter().map(|x| (SocketData::UDP(x.clone()), map.get(&x.inode).cloned()));
    let unix = procfs::net::unix().unwrap();
    let unix_new = unix.iter().map(|x| (SocketData::Unix(x.clone()), map.get(&x.inode).cloned()));
    let all_sockets: Vec<(SocketData, Option<Stat>)> = tcp_new.into_iter().chain(tcp6_new).chain(udp_new).chain(udp6_new).chain(unix_new).collect();
    for socket in all_sockets {
        let mut dict = match socket.0 {
            SocketData::TCP(tcp) => {
                let map: SmallMap<Value, Value> = SmallMap::new();
                // Create Dict type.
                let mut dict = Dict::new(map);
                dict.insert_hashed(const_frozen_string!("local_address").to_value().get_hashed()?, starlark_heap.alloc_str(&tcp.local_address.to_string()).to_value());
                dict.insert_hashed(const_frozen_string!("remote_address").to_value().get_hashed()?, starlark_heap.alloc_str(&tcp.remote_address.to_string()).to_value());
                dict.insert_hashed(const_frozen_string!("state").to_value().get_hashed()?, starlark_heap.alloc_str(&format!("{:?}", tcp.state)).to_value());
                dict.insert_hashed(const_frozen_string!("rx_queue").to_value().get_hashed()?, starlark_heap.alloc(tcp.rx_queue));
                dict.insert_hashed(const_frozen_string!("tx_queue").to_value().get_hashed()?, starlark_heap.alloc(tcp.tx_queue));
                dict.insert_hashed(const_frozen_string!("uid").to_value().get_hashed()?, starlark_heap.alloc(tcp.uid));
                dict.insert_hashed(const_frozen_string!("inode").to_value().get_hashed()?, starlark_heap.alloc(tcp.inode));
                dict
            },
            SocketData::UDP(udp) => {
                let map: SmallMap<Value, Value> = SmallMap::new();
                // Create Dict type.
                let mut dict = Dict::new(map);
                dict.insert_hashed(const_frozen_string!("local_address").to_value().get_hashed()?, starlark_heap.alloc_str(&udp.local_address.to_string()).to_value());
                dict.insert_hashed(const_frozen_string!("remote_address").to_value().get_hashed()?, starlark_heap.alloc_str(&udp.remote_address.to_string()).to_value());
                dict.insert_hashed(const_frozen_string!("state").to_value().get_hashed()?, starlark_heap.alloc_str(&format!("{:?}", udp.state)).to_value());
                dict.insert_hashed(const_frozen_string!("rx_queue").to_value().get_hashed()?, starlark_heap.alloc(udp.rx_queue));
                dict.insert_hashed(const_frozen_string!("tx_queue").to_value().get_hashed()?, starlark_heap.alloc(udp.tx_queue));
                dict.insert_hashed(const_frozen_string!("uid").to_value().get_hashed()?, starlark_heap.alloc(udp.uid));
                dict.insert_hashed(const_frozen_string!("inode").to_value().get_hashed()?, starlark_heap.alloc(udp.inode));
                dict
            },
            SocketData::Unix(unix) => {
                let map: SmallMap<Value, Value> = SmallMap::new();
                // Create Dict type.
                let mut dict = Dict::new(map);
                dict.insert_hashed(const_frozen_string!("ref_count").to_value().get_hashed()?, starlark_heap.alloc(unix.ref_count));
                dict.insert_hashed(const_frozen_string!("socket_type").to_value().get_hashed()?, starlark_heap.alloc(unix.socket_type as u32));
                dict.insert_hashed(const_frozen_string!("state").to_value().get_hashed()?, starlark_heap.alloc_str(&format!("{:?}", unix.state)).to_value());
                dict.insert_hashed(const_frozen_string!("inode").to_value().get_hashed()?, starlark_heap.alloc(unix.inode));
                dict.insert_hashed(const_frozen_string!("path").to_value().get_hashed()?, starlark_heap.alloc_str(&unix.path.map(|x| x.display().to_string()).unwrap_or("N/A".to_string())).to_value());
                dict
            },
        };
        match socket.1 {
            Some(proc) => {
                let map: SmallMap<Value, Value> = SmallMap::new();
                // Create Dict type.
                let mut proc_dict = Dict::new(map);
                proc_dict.insert_hashed(const_frozen_string!("pid").to_value().get_hashed()?, starlark_heap.alloc(proc.pid));
                proc_dict.insert_hashed(const_frozen_string!("name").to_value().get_hashed()?, starlark_heap.alloc_str(&proc.comm).to_value());
                dict.insert_hashed(const_frozen_string!("proc").to_value().get_hashed()?, starlark_heap.alloc(proc_dict));
            },
            None => {
                dict.insert_hashed(const_frozen_string!("proc").to_value().get_hashed()?, Value::new_none());
            }
        }
        out.push(dict);
    }

    Ok(out)
}