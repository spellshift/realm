use anyhow::Result;
use starlark::values::{dict::Dict, Heap};

const UNKNOWN: &str = "UNKNOWN";

pub fn netstat(starlark_heap: &'_ Heap) -> Result<Vec<Dict<'_>>> {
    use super::super::insert_dict_kv;
    use starlark::{collections::SmallMap, const_frozen_string, values::Value};

    let entries = netstat::netstat()?;
    let mut out: Vec<Dict> = Vec::new();

    for entry in entries {
        let map: SmallMap<Value, Value> = SmallMap::new();
        let mut dict = Dict::new(map);

        // socket_type: "TCP" | "UDP"
        insert_dict_kv!(
            dict,
            starlark_heap,
            "socket_type",
            entry.socket_type.to_string(),
            String
        );

        // local_address
        insert_dict_kv!(
            dict,
            starlark_heap,
            "local_address",
            entry.local_address.to_string(),
            String
        );

        // local_port
        insert_dict_kv!(
            dict,
            starlark_heap,
            "local_port",
            entry.local_port as u32,
            u32
        );

        // remote_address: IP or "UNKNOWN"
        let remote_addr = entry
            .remote_address
            .map(|ip| ip.to_string())
            .unwrap_or_else(|| UNKNOWN.to_string());
        insert_dict_kv!(dict, starlark_heap, "remote_address", remote_addr, String);

        // remote_port: u16
        insert_dict_kv!(
            dict,
            starlark_heap,
            "remote_port",
            entry.remote_port as u32,
            u32
        );

        // connection_state: "ESTABLISHED" | "LISTEN" | ... | "UNKNOWN"
        insert_dict_kv!(
            dict,
            starlark_heap,
            "connection_state",
            entry.connection_state.to_string(),
            String
        );

        // pid: u32
        insert_dict_kv!(dict, starlark_heap, "pid", entry.pid, u32);

        // process_name: "node" | "UNKNOWN"
        let proc_name = entry.process_name.unwrap_or_else(|| UNKNOWN.to_string());
        insert_dict_kv!(dict, starlark_heap, "process_name", proc_name, String);

        out.push(dict);
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
        // Try to bind to a random port
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
        let test_port: u32 = listener.local_addr()?.port().into();
        let _listen_task = task::spawn(local_accept_tcp(listener));
        let res = netstat(&heap)?;
        let real_pid = id();

        for socket in res {
            let socket_type = socket
                .get(const_frozen_string!("socket_type").to_value())
                .unwrap()
                .and_then(|val| val.unpack_str());

            if socket_type != Some("TCP") {
                continue;
            }

            let local_addr = socket
                .get(const_frozen_string!("local_address").to_value())
                .unwrap()
                .and_then(|val| val.unpack_str());

            if local_addr != Some("127.0.0.1") {
                continue;
            }

            let local_port = socket
                .get(const_frozen_string!("local_port").to_value())
                .unwrap()
                .and_then(|val| val.unpack_i32());

            if local_port != Some(test_port as i32) {
                continue;
            }

            let pid = socket
                .get(const_frozen_string!("pid").to_value())
                .unwrap()
                .and_then(|val| val.unpack_i32());

            // Verify all required fields are present
            assert!(socket
                .get(const_frozen_string!("remote_address").to_value())
                .unwrap()
                .is_some());
            assert!(socket
                .get(const_frozen_string!("remote_port").to_value())
                .unwrap()
                .is_some());
            assert!(socket
                .get(const_frozen_string!("connection_state").to_value())
                .unwrap()
                .is_some());
            assert!(socket
                .get(const_frozen_string!("process_name").to_value())
                .unwrap()
                .is_some());

            // If we can get the PID, it should match ours
            if let Some(socket_pid) = pid {
                if socket_pid == real_pid as i32 {
                    return Ok(());
                }
            }
        }

        Err(anyhow::anyhow!(
            "Failed to find our test socket in netstat results"
        ))
    }

    #[tokio::test]
    async fn test_netstat_all_fields_present() -> Result<()> {
        let heap = Heap::new();
        let res = netstat(&heap)?;

        // Verify every entry has all required fields
        for socket in res {
            assert!(socket
                .get(const_frozen_string!("socket_type").to_value())
                .unwrap()
                .is_some());
            assert!(socket
                .get(const_frozen_string!("local_address").to_value())
                .unwrap()
                .is_some());
            assert!(socket
                .get(const_frozen_string!("local_port").to_value())
                .unwrap()
                .is_some());
            assert!(socket
                .get(const_frozen_string!("remote_address").to_value())
                .unwrap()
                .is_some());
            assert!(socket
                .get(const_frozen_string!("remote_port").to_value())
                .unwrap()
                .is_some());
            assert!(socket
                .get(const_frozen_string!("connection_state").to_value())
                .unwrap()
                .is_some());
            assert!(socket
                .get(const_frozen_string!("pid").to_value())
                .unwrap()
                .is_some());
            assert!(socket
                .get(const_frozen_string!("process_name").to_value())
                .unwrap()
                .is_some());
        }

        Ok(())
    }
}
