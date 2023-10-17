use std::collections::HashMap;
#[cfg(target_os = "linux")]
use procfs::{
    process::*,
    net::*
};
use starlark::{values::{dict::Dict, Heap, Value}, collections::SmallMap, const_frozen_string};
use anyhow::Result;

#[cfg(target_os = "linux")]
#[derive(Debug)]
enum SocketData {
    TCP(TcpNetEntry),
    UDP(UdpNetEntry),
    Unix(UnixNetEntry)
}

#[cfg(not(target_os = "linux"))]
pub fn netstat(starlark_heap: &Heap) -> Result<Vec<Dict>> {
    return Err(anyhow!("Not implemented for this platform"));
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
                dict.insert_hashed(const_frozen_string!("socket_type").to_value().get_hashed()?, const_frozen_string!("TCP").to_value());
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
                dict.insert_hashed(const_frozen_string!("socket_type").to_value().get_hashed()?, const_frozen_string!("UDP").to_value());
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
                dict.insert_hashed(const_frozen_string!("socket_type").to_value().get_hashed()?, const_frozen_string!("Unix").to_value());
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

#[cfg(test)]
#[cfg(target_os = "linux")]
mod tests {
    use super::*;
    use starlark::values::{Heap, Value};
    use anyhow::Result;
    use tokio::net::TcpListener;
    use tokio::task;
    use tokio::io::copy;

    async fn local_bind_tcp() -> TcpListener {
        // Try three times to bind to a port
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        return listener;
    }

    async fn local_accept_tcp(listener: TcpListener) -> Result<()> {
        // Accept new connection
        let (mut socket, _) = listener.accept().await?;
        // Split reader and writer references
        let (mut reader, mut writer) = socket.split();
        // Copy from reader to writer to echo message back.
        let bytes_copied = copy(&mut reader, &mut writer).await?;
        // If message sent break loop
        if bytes_copied > 1 {
            return Ok(());
        } else {
            return Err(anyhow::anyhow!("Failed to copy any bytes"));
        }
    }

    #[tokio::test]
    async fn test_netstat() -> Result<()>{
        let heap = Heap::new();
        let listener = local_bind_tcp().await;
        let test_port: i32 = listener.local_addr()?.port().into();
        let _listen_task = task::spawn(local_accept_tcp(listener));
        let res = netstat(&heap)?;
        for socket in res {
            if Some(Some("TCP")) != socket.get(const_frozen_string!("socket_type").to_value()).unwrap().map(|val| val.unpack_str()) {
                continue;
            }
            if Some(Some(format!("127.0.0.1:{}", test_port).as_str())) != socket.get(const_frozen_string!("local_address").to_value()).unwrap().map(|val| val.unpack_str()) {
                continue;
            }
            if Some(Some("Listen")) != socket.get(const_frozen_string!("state").to_value()).unwrap().map(|val| val.unpack_str()) {
                continue;
            }
            return Ok(())
        }
        Err(anyhow::anyhow!("Failed to find socket"))
    }
}

#[cfg(not(target_os = "linux"))]
mod tests {
    #[test]
    fn test_netstat_not_linux() -> Result<()> {
        let heap = Heap::new();
        assert!(netstat(&heap).is_err());
        Ok(())
    }
}