use anyhow::Result;
use starlark::values::{dict::Dict, Heap};

#[cfg(target_os = "freebsd")]
pub fn netstat(_: &Heap) -> Result<Vec<Dict<'_>>> {
    Err(anyhow::anyhow!("Not implemented for FreeBSD"))
}

#[cfg(not(target_os = "freebsd"))]
pub fn netstat(starlark_heap: &Heap) -> Result<Vec<Dict<'_>>> {
    use super::super::insert_dict_kv;
    use starlark::{collections::SmallMap, const_frozen_string, values::Value};

    let mut out: Vec<Dict> = Vec::new();

    if let Ok(listeners) = listeners::get_all() {
        for l in listeners {
            let map: SmallMap<Value, Value> = SmallMap::new();
            // Create Dict type.
            let mut dict = Dict::new(map);
            insert_dict_kv!(dict, starlark_heap, "socket_type", "TCP", String);
            insert_dict_kv!(
                dict,
                starlark_heap,
                "local_address",
                l.socket.ip().to_string(),
                String
            );
            insert_dict_kv!(
                dict,
                starlark_heap,
                "local_port",
                l.socket.port() as u32,
                u32
            );
            insert_dict_kv!(dict, starlark_heap, "pid", l.process.pid, u32);
            out.push(dict);
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use starlark::const_frozen_string;
    use starlark::values::Heap;
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
        let real_pid = id() as i32;
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
            if let Some(Some(pid)) = socket
                .get(const_frozen_string!("pid").to_value())
                .unwrap()
                .map(|val| val.unpack_i32())
            {
                if pid == real_pid {
                    return Ok(());
                }
            }
        }
        Err(anyhow::anyhow!("Failed to find socket"))
    }
}
