use super::super::insert_dict_kv;
use anyhow::Result;
#[cfg(target_os = "freebsd")]
use anyhow::anyhow;
#[cfg(not(target_os = "freebsd"))]
use netstat2::*;
use starlark::{
    collections::SmallMap,
    const_frozen_string,
    values::{dict::Dict, Heap, Value},
};

#[cfg(target_os = "freebsd")]
pub fn netstat(_: &Heap) -> Result<Vec<Dict>> {
    Err(anyhow!("Not implemented for FreeBSD"))
}

#[cfg(not(target_os = "freebsd"))]
pub fn netstat(starlark_heap: &Heap) -> Result<Vec<Dict>> {
    let mut out: Vec<Dict> = Vec::new();
    let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let proto_flags = ProtocolFlags::TCP | ProtocolFlags::UDP;
    let sockets_info = get_sockets_info(af_flags, proto_flags)?;

    for si in sockets_info {
        match si.protocol_socket_info {
            ProtocolSocketInfo::Tcp(tcp_si) => {
                let map: SmallMap<Value, Value> = SmallMap::new();
                // Create Dict type.
                let mut dict = Dict::new(map);
                insert_dict_kv!(dict, starlark_heap, "socket_type", "TCP", String);
                insert_dict_kv!(
                    dict,
                    starlark_heap,
                    "local_address",
                    tcp_si.local_addr.to_string(),
                    String
                );
                insert_dict_kv!(
                    dict,
                    starlark_heap,
                    "local_port",
                    tcp_si.local_port as u32,
                    u32
                );
                insert_dict_kv!(
                    dict,
                    starlark_heap,
                    "remote_address",
                    tcp_si.remote_addr.to_string(),
                    String
                );
                insert_dict_kv!(
                    dict,
                    starlark_heap,
                    "remote_port",
                    tcp_si.remote_port as u32,
                    u32
                );
                insert_dict_kv!(
                    dict,
                    starlark_heap,
                    "state",
                    tcp_si.state.to_string(),
                    String
                );
                insert_dict_kv!(dict, starlark_heap, "pids", si.associated_pids, Vec<_>);
                out.push(dict);
            }
            ProtocolSocketInfo::Udp(udp_si) => {
                let map: SmallMap<Value, Value> = SmallMap::new();
                // Create Dict type.
                let mut dict = Dict::new(map);
                insert_dict_kv!(dict, starlark_heap, "socket_type", "UDP", String);
                insert_dict_kv!(
                    dict,
                    starlark_heap,
                    "local_address",
                    udp_si.local_addr.to_string(),
                    String
                );
                insert_dict_kv!(
                    dict,
                    starlark_heap,
                    "local_port",
                    udp_si.local_port as u32,
                    u32
                );
                insert_dict_kv!(dict, starlark_heap, "pids", si.associated_pids, Vec<_>);
                out.push(dict);
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use starlark::values::{Heap, UnpackValue};
    use std::process::id;
    use tokio::io::copy;
    use tokio::net::TcpListener;
    use tokio::task;

    async fn local_bind_tcp() -> TcpListener {
        // Try three times to bind to a port
        TcpListener::bind("127.0.0.1:0").await.unwrap()
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
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to copy any bytes"))
        }
    }

    #[tokio::test]
    async fn test_netstat() -> Result<()> {
        let heap = Heap::new();
        let listener = local_bind_tcp().await;
        let test_port: i32 = listener.local_addr()?.port().into();
        let _listen_task = task::spawn(local_accept_tcp(listener));
        let res = netstat(&heap)?;
        let pid = id() as i32;
        for socket in res {
            if Some(Some("TCP"))
                != socket
                    .get(const_frozen_string!("socket_type").to_value())
                    .unwrap()
                    .map(|val| val.unpack_str())
            {
                continue;
            }
            if Some(Some("127.0.0.1"))
                != socket
                    .get(const_frozen_string!("local_address").to_value())
                    .unwrap()
                    .map(|val| val.unpack_str())
            {
                continue;
            }
            if Some(Some(test_port))
                != socket
                    .get(const_frozen_string!("local_port").to_value())
                    .unwrap()
                    .map(|val| val.unpack_i32())
            {
                continue;
            }
            if Some(Some("LISTEN"))
                != socket
                    .get(const_frozen_string!("state").to_value())
                    .unwrap()
                    .map(|val| val.unpack_str())
            {
                continue;
            }
            if let Some(Some(pids)) = socket
                .get(const_frozen_string!("pids").to_value())
                .unwrap()
                .map(Vec::<i32>::unpack_value)
            {
                if pids.contains(&pid) {
                    return Ok(());
                }
            }
        }
        Err(anyhow::anyhow!("Failed to find socket"))
    }
}
