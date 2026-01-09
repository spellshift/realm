use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use eldritch_core::Value;

#[cfg(target_os = "freebsd")]
pub fn netstat() -> Result<Vec<BTreeMap<String, Value>>, String> {
    Err("Not implemented for FreeBSD".to_string())
}

#[cfg(not(target_os = "freebsd"))]
pub fn netstat() -> Result<Vec<BTreeMap<String, Value>>, String> {
    const UNKNOWN: &str = "UNKNOWN";
    let entries = netstat::netstat().map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for entry in entries {
        let mut map = BTreeMap::new();
        map.insert(
            "socket_type".to_string(),
            Value::String(entry.socket_type.to_string()),
        );
        map.insert(
            "local_address".to_string(),
            Value::String(entry.local_address.to_string()),
        );
        map.insert(
            "local_port".to_string(),
            Value::Int(entry.local_port as i64),
        );
        let remote_addr = entry
            .remote_address
            .map(|ip| ip.to_string())
            .unwrap_or_else(|| UNKNOWN.to_string());
        map.insert("remote_address".to_string(), Value::String(remote_addr));
        map.insert(
            "remote_port".to_string(),
            Value::Int(entry.remote_port as i64),
        );
        map.insert(
            "connection_state".to_string(),
            Value::String(entry.connection_state.to_string()),
        );
        map.insert("pid".to_string(), Value::Int(entry.pid as i64));
        map.insert(
            "process_name".to_string(),
            Value::String(
                entry
                    .process_name
                    .unwrap_or(UNKNOWN.to_string())
                    .to_string(),
            ),
        );
        out.push(map);
    }
    Ok(out)
}

#[cfg(all(test, feature = "stdlib"))]
mod tests {
    use super::super::ProcessLibrary;
    use super::super::StdProcessLibrary;
    use eldritch_core::Value;
    use std::process::id;
    use tokio::io::copy;
    use tokio::net::TcpListener;
    use tokio::task;

    #[test]
    fn test_std_process_netstat() {
        let lib = StdProcessLibrary;
        // netstat relies on system permissions and open ports, so we just check it doesn't crash
        // and returns a result (even empty).
        let res = lib.netstat();
        assert!(res.is_ok());
    }

    async fn local_bind_tcp() -> TcpListener {
        // Try to bind to a random port
        TcpListener::bind("127.0.0.1:0").await.unwrap()
    }

    async fn local_accept_tcp(listener: TcpListener) {
        // Accept new connection
        let (mut socket, _) = listener.accept().await.unwrap();
        // Split reader and writer references
        let (mut reader, mut writer) = socket.split();
        // Copy from reader to writer to echo message back.
        let bytes_copied = copy(&mut reader, &mut writer).await.unwrap();
        // If message sent break loop
        assert!(bytes_copied > 0);
    }

    #[tokio::test]
    async fn test_netstat() {
        let listener = local_bind_tcp().await;
        let test_port: u32 = listener.local_addr().unwrap().port().into();
        let _listen_task = task::spawn(local_accept_tcp(listener));
        let lib = StdProcessLibrary;
        let res = lib.netstat().unwrap();
        let real_pid = id();

        let mut found = false;
        for socket in res {
            let socket_type = socket.get("socket_type").and_then(|val| {
                if let Value::String(s) = val {
                    Some(s.as_str())
                } else {
                    None
                }
            });

            if socket_type != Some("TCP") {
                continue;
            }

            let local_addr = socket.get("local_address").and_then(|val| {
                if let Value::String(s) = val {
                    Some(s.as_str())
                } else {
                    None
                }
            });

            if local_addr != Some("127.0.0.1") {
                continue;
            }

            let local_port = socket.get("local_port").and_then(|val| {
                if let Value::Int(i) = val {
                    Some(*i as i32)
                } else {
                    None
                }
            });

            if local_port != Some(test_port as i32) {
                continue;
            }

            let pid = socket.get("pid").and_then(|val| {
                if let Value::Int(i) = val {
                    Some(*i as i32)
                } else {
                    None
                }
            });

            // Verify all required fields are present
            assert!(socket.contains_key("remote_address"));
            assert!(socket.contains_key("remote_port"));
            assert!(socket.contains_key("connection_state"));
            assert!(socket.contains_key("process_name"));

            // If we can get the PID, it should match ours
            if let Some(socket_pid) = pid
                && socket_pid == real_pid as i32
            {
                found = true;
                break;
            }
        }

        assert!(found, "Failed to find our test socket in netstat results");
    }

    #[tokio::test]
    async fn test_netstat_all_fields_present() {
        let lib = StdProcessLibrary;
        let res = lib.netstat().unwrap();

        // Verify every entry has all required fields
        for socket in res {
            assert!(socket.contains_key("socket_type"));
            assert!(socket.contains_key("local_address"));
            assert!(socket.contains_key("local_port"));
            assert!(socket.contains_key("remote_address"));
            assert!(socket.contains_key("remote_port"));
            assert!(socket.contains_key("connection_state"));
            assert!(socket.contains_key("pid"));
            assert!(socket.contains_key("process_name"));
        }
    }
}
