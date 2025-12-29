use anyhow::Result;
use pb::c2::{CreatePortalRequest, CreatePortalResponse};
use pb::portal::payload::Payload as PortalPayloadEnum;
use pb::portal::{BytesMessageKind, TcpMessage, UdpMessage};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tokio_stream::StreamExt;
use transport::Transport;

pub async fn run_create_portal<T: Transport + 'static>(
    task_id: i64,
    mut transport: T,
) -> Result<()> {
    // 1. Setup channels
    let (outbound_tx, outbound_rx) = mpsc::channel(32);
    let (inbound_tx, inbound_rx) = mpsc::channel(32);

    // 2. Send initial registration message
    if outbound_tx
        .send(CreatePortalRequest {
            task_id,
            payload: None,
        })
        .await
        .is_err()
    {
        return Ok(());
    }

    // 3. Start transport stream
    let transport_handle = tokio::spawn(async move {
        if let Err(e) = transport.create_portal(outbound_rx, inbound_tx).await {
            #[cfg(debug_assertions)]
            log::error!("Portal transport error: {}", e);
        }
    });

    // 4. Run loop
    let stream = tokio_stream::wrappers::ReceiverStream::new(inbound_rx);
    let result = run_portal_loop(stream, outbound_tx, task_id).await;

    // Cleanup
    transport_handle.abort();

    result
}

async fn run_portal_loop<S>(
    mut resp_stream: S,
    outbound_tx: tokio::sync::mpsc::Sender<CreatePortalRequest>,
    task_id: i64,
) -> Result<()>
where
    S: tokio_stream::Stream<Item = CreatePortalResponse> + Unpin,
{
    // Map stores Sender to the connection handler task
    // Key: src_id
    // Value: Sender for (seq_id, data)
    let connections: Arc<Mutex<HashMap<String, tokio::sync::mpsc::Sender<(u64, Vec<u8>)>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    while let Some(msg) = resp_stream.next().await {
        if let Some(payload_enum) = msg.payload.and_then(|p| p.payload) {
            match payload_enum {
                PortalPayloadEnum::Tcp(tcp_msg) => {
                    let src_id = tcp_msg.src_id.clone();

                    let tx = {
                        let mut map = connections.lock().unwrap();
                        if let Some(tx) = map.get(&src_id) {
                            if tx.is_closed() {
                                None
                            } else {
                                Some(tx.clone())
                            }
                        } else {
                            None
                        }
                    };

                    if let Some(tx) = tx {
                        if !tcp_msg.data.is_empty() {
                            let _ = tx.send((tcp_msg.seq_id, tcp_msg.data)).await;
                        }
                    } else {
                        let (tx, rx) = tokio::sync::mpsc::channel(100);

                        connections
                            .lock()
                            .unwrap()
                            .insert(src_id.clone(), tx.clone());

                        let map_clone = connections.clone();
                        let outbound_tx_clone = outbound_tx.clone();
                        let dst_addr = tcp_msg.dst_addr;
                        let dst_port = tcp_msg.dst_port;
                        let initial_data = tcp_msg.data;
                        let seq_id = tcp_msg.seq_id;

                        tokio::spawn(async move {
                            handle_tcp_connection(
                                rx,
                                src_id,
                                dst_addr,
                                dst_port,
                                outbound_tx_clone,
                                map_clone,
                                task_id,
                            )
                            .await;
                        });

                        if !initial_data.is_empty() {
                            let _ = tx.send((seq_id, initial_data)).await;
                        }
                    }
                }
                PortalPayloadEnum::Udp(udp_msg) => {
                    let src_id = udp_msg.src_id.clone();

                    let tx = {
                        let mut map = connections.lock().unwrap();
                        if let Some(tx) = map.get(&src_id) {
                            if tx.is_closed() {
                                None
                            } else {
                                Some(tx.clone())
                            }
                        } else {
                            None
                        }
                    };

                    if let Some(tx) = tx {
                        if !udp_msg.data.is_empty() {
                            let _ = tx.send((udp_msg.seq_id, udp_msg.data)).await;
                        }
                    } else {
                        let (tx, rx) = tokio::sync::mpsc::channel(100);
                        connections
                            .lock()
                            .unwrap()
                            .insert(src_id.clone(), tx.clone());

                        let map_clone = connections.clone();
                        let outbound_tx_clone = outbound_tx.clone();
                        let dst_addr = udp_msg.dst_addr;
                        let dst_port = udp_msg.dst_port;
                        let initial_data = udp_msg.data;
                        let seq_id = udp_msg.seq_id;

                        tokio::spawn(async move {
                            handle_udp_connection(
                                rx,
                                src_id,
                                dst_addr,
                                dst_port,
                                outbound_tx_clone,
                                map_clone,
                                task_id,
                            )
                            .await;
                        });

                        if !initial_data.is_empty() {
                            let _ = tx.send((seq_id, initial_data)).await;
                        }
                    }
                }
                PortalPayloadEnum::Bytes(bytes_msg) => {
                    if bytes_msg.kind == BytesMessageKind::Ping as i32 {
                        let req = CreatePortalRequest {
                            task_id,
                            payload: Some(pb::portal::Payload {
                                payload: Some(PortalPayloadEnum::Bytes(bytes_msg)),
                            }),
                        };
                        let _ = outbound_tx.send(req).await;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn handle_tcp_connection(
    mut rx: tokio::sync::mpsc::Receiver<(u64, Vec<u8>)>,
    src_id: String,
    dst_addr: String,
    dst_port: u32,
    outbound_tx: tokio::sync::mpsc::Sender<CreatePortalRequest>,
    connections: Arc<Mutex<HashMap<String, tokio::sync::mpsc::Sender<(u64, Vec<u8>)>>>>,
    task_id: i64,
) {
    let addr = format!("{}:{}", dst_addr, dst_port);
    match tokio::net::TcpStream::connect(&addr).await {
        Ok(stream) => {
            if let Err(e) = stream.set_nodelay(true) {
                #[cfg(debug_assertions)]
                log::warn!("Failed to set nodelay: {}", e);
            }

            let (mut reader, mut writer) = stream.into_split();
            let mut buf = [0u8; 64 * 1024];

            let mut join_set = JoinSet::new();
            let mut out_seq_id = 0u64;

            let mut reorder_buf: HashMap<u64, Vec<u8>> = HashMap::new();
            let mut next_expected_seq = 0u64;
            let buffer_limit = 100;

            loop {
                tokio::select! {
                    res = reader.read(&mut buf) => {
                        match res {
                            Ok(0) => break, // EOF
                            Ok(n) => {
                                let data = buf[0..n].to_vec();
                                let req = CreatePortalRequest {
                                    task_id,
                                    payload: Some(pb::portal::Payload {
                                        payload: Some(PortalPayloadEnum::Tcp(TcpMessage {
                                            data,
                                            dst_addr: dst_addr.clone(),
                                            dst_port,
                                            src_id: src_id.clone(),
                                            seq_id: out_seq_id,
                                        })),
                                    }),
                                };
                                out_seq_id += 1;

                                let tx_clone = outbound_tx.clone();
                                join_set.spawn(async move {
                                    tx_clone.send(req).await
                                });

                                if join_set.len() >= 50 {
                                    join_set.join_next().await;
                                }
                            }
                            Err(_) => break,
                        }
                    }
                    res = rx.recv() => {
                        match res {
                            Some((seq_id, data)) => {
                                if seq_id > next_expected_seq {
                                    if reorder_buf.len() >= buffer_limit {
                                        #[cfg(debug_assertions)]
                                        log::warn!("Stall detected, closing connection {}", src_id);
                                        break;
                                    }
                                    reorder_buf.insert(seq_id, data);
                                } else if seq_id == next_expected_seq {
                                    if writer.write_all(&data).await.is_err() {
                                        break;
                                    }
                                    next_expected_seq += 1;

                                    while let Some(next_data) = reorder_buf.remove(&next_expected_seq) {
                                        if writer.write_all(&next_data).await.is_err() {
                                            break;
                                        }
                                        next_expected_seq += 1;
                                    }
                                }
                            }
                            None => break,
                        }
                    }
                }
            }
        }
        Err(e) => {
            #[cfg(debug_assertions)]
            log::error!("TCP Connect failed: {}", e);
        }
    }

    connections.lock().unwrap().remove(&src_id);
}

async fn handle_udp_connection(
    mut rx: tokio::sync::mpsc::Receiver<(u64, Vec<u8>)>,
    src_id: String,
    dst_addr: String,
    dst_port: u32,
    outbound_tx: tokio::sync::mpsc::Sender<CreatePortalRequest>,
    connections: Arc<Mutex<HashMap<String, tokio::sync::mpsc::Sender<(u64, Vec<u8>)>>>>,
    task_id: i64,
) {
    let addr = format!("{}:{}", dst_addr, dst_port);
    let socket = match tokio::net::UdpSocket::bind("0.0.0.0:0").await {
        Ok(s) => s,
        Err(_) => {
            connections.lock().unwrap().remove(&src_id);
            return;
        }
    };
    if socket.connect(&addr).await.is_err() {
        connections.lock().unwrap().remove(&src_id);
        return;
    }

    let socket = Arc::new(socket);
    let mut buf = [0u8; 65535];

    let mut out_seq_id = 0u64;
    let mut join_set = JoinSet::new();

    loop {
        tokio::select! {
            res = socket.recv(&mut buf) => {
                match res {
                    Ok(n) => {
                         let req = CreatePortalRequest {
                            task_id,
                            payload: Some(pb::portal::Payload {
                                payload: Some(PortalPayloadEnum::Udp(UdpMessage {
                                    data: buf[0..n].to_vec(),
                                    dst_addr: dst_addr.clone(),
                                    dst_port,
                                    src_id: src_id.clone(),
                                    seq_id: out_seq_id,
                                })),
                            }),
                        };
                        out_seq_id += 1;

                        let tx_clone = outbound_tx.clone();
                        join_set.spawn(async move {
                            tx_clone.send(req).await
                        });

                        if join_set.len() >= 50 {
                            join_set.join_next().await;
                        }
                    }
                    Err(_) => break,
                }
            }
            res = rx.recv() => {
                match res {
                    Some((_seq, data)) => {
                        if socket.send(&data).await.is_err() {
                            break;
                        }
                    }
                    None => break,
                }
            }
        }
    }
    connections.lock().unwrap().remove(&src_id);
}

#[cfg(test)]
mod tests {
    use super::*;
    use pb::portal::TcpMessage;
    use pb::portal::payload::Payload as PortalPayloadEnum;
    use std::time::Duration;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_run_portal_loop_tcp() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (target_tx, mut target_rx) = mpsc::channel(10);

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 1024];
            let n = socket.read(&mut buf).await.unwrap();
            target_tx.send(buf[0..n].to_vec()).await.unwrap();
            socket.write_all(b"pong").await.unwrap();
        });

        let (outbound_tx, mut outbound_rx) = mpsc::channel(10);
        let task_id = 123;

        let (server_tx, server_rx) = mpsc::channel(10);
        let stream = tokio_stream::wrappers::ReceiverStream::new(server_rx);

        let loop_handle =
            tokio::spawn(async move { run_portal_loop(stream, outbound_tx, task_id).await });

        server_tx
            .send(CreatePortalResponse {
                payload: Some(pb::portal::Payload {
                    payload: Some(PortalPayloadEnum::Tcp(TcpMessage {
                        data: b"ping".to_vec(),
                        dst_addr: "127.0.0.1".to_string(),
                        dst_port: addr.port() as u32,
                        src_id: "abcdefg".to_string(),
                        seq_id: 0,
                    })),
                }),
            })
            .await
            .unwrap();

        let received = tokio::time::timeout(Duration::from_secs(2), target_rx.recv())
            .await
            .expect("timeout waiting for target receive")
            .expect("target channel closed");
        assert_eq!(received, b"ping");

        let resp = tokio::time::timeout(Duration::from_secs(2), outbound_rx.recv())
            .await
            .expect("timeout waiting for outbound response")
            .expect("outbound channel closed");

        assert_eq!(resp.task_id, task_id);
        if let Some(PortalPayloadEnum::Tcp(tcp)) = resp.payload.unwrap().payload {
            assert_eq!(tcp.data, b"pong");
            assert_eq!(tcp.src_id, "abcdefg".to_string());
            assert_eq!(tcp.seq_id, 0);
        } else {
            panic!("Expected TCP message");
        }

        drop(server_tx);
        loop_handle.await.unwrap().unwrap();
    }
}
