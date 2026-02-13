use crate::portal::{bytes, run, tcp, udp};
use pb::c2::{CreatePortalResponse, TaskContext};
use pb::portal::{BytesPayloadKind, Mote, mote::Payload};
use pb::trace::{TraceData, TraceEventKind};
use portal_stream::PayloadSequencer;
use prost::Message;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, UdpSocket};
use tokio::sync::mpsc;
use tokio::time::timeout;
use transport::MockTransport;

#[tokio::test]
async fn test_portal_bytes_ping() {
    let (_tx, rx) = mpsc::channel(100);
    let (out_tx, mut out_rx) = mpsc::channel(100);
    let sequencer = PayloadSequencer::new("test-stream");

    let first_mote = sequencer.new_bytes_mote(vec![1, 2, 3], BytesPayloadKind::Ping);

    let _handle =
        tokio::spawn(async move { bytes::handle_bytes(first_mote, rx, out_tx, sequencer).await });

    let resp = timeout(Duration::from_secs(1), out_rx.recv())
        .await
        .expect("Timeout waiting for ping response")
        .expect("No ping response");

    if let Some(Payload::Bytes(b)) = resp.payload {
        assert_eq!(b.kind, BytesPayloadKind::Ping as i32);
        assert_eq!(b.data, vec![1, 2, 3]);
    } else {
        panic!("Expected bytes payload");
    }
}

#[tokio::test]
async fn test_portal_bytes_keepalive() {
    let (_tx, rx) = mpsc::channel(100);
    let (out_tx, mut out_rx) = mpsc::channel(100);
    let sequencer = PayloadSequencer::new("test-stream");

    let first_mote = sequencer.new_bytes_mote(vec![], BytesPayloadKind::Keepalive);

    let _handle =
        tokio::spawn(async move { bytes::handle_bytes(first_mote, rx, out_tx, sequencer).await });

    // Keepalive should be ignored, so no response expected
    let resp = timeout(Duration::from_millis(100), out_rx.recv()).await;
    assert!(resp.is_err(), "Expected timeout, but got a response");
}

#[tokio::test]
async fn test_portal_bytes_unspecified() {
    let (_tx, rx) = mpsc::channel(100);
    let (out_tx, mut out_rx) = mpsc::channel(100);
    let sequencer = PayloadSequencer::new("test-stream");

    let first_mote = sequencer.new_bytes_mote(vec![1, 2, 3], BytesPayloadKind::Unspecified);

    let _handle =
        tokio::spawn(async move { bytes::handle_bytes(first_mote, rx, out_tx, sequencer).await });

    // Unspecified should be ignored
    let resp = timeout(Duration::from_millis(100), out_rx.recv()).await;
    assert!(resp.is_err(), "Expected timeout, but got a response");
}

#[tokio::test]
async fn test_portal_bytes_multiple_pings() {
    let (tx, rx) = mpsc::channel(100);
    let (out_tx, mut out_rx) = mpsc::channel(100);
    let sequencer = PayloadSequencer::new("test-stream");

    let first_mote = sequencer.new_bytes_mote(vec![1], BytesPayloadKind::Ping);

    let _handle =
        tokio::spawn(async move { bytes::handle_bytes(first_mote, rx, out_tx, sequencer).await });

    // Send second ping
    let sequencer2 = PayloadSequencer::new("test-stream");
    let _ = sequencer2.next_seq_id(); // Skip 0
    let second_mote = sequencer2.new_bytes_mote(vec![2], BytesPayloadKind::Ping);
    tx.send(second_mote).await.unwrap();

    // Verify first response
    let resp1 = timeout(Duration::from_secs(1), out_rx.recv())
        .await
        .expect("Timeout waiting for ping response 1")
        .expect("No ping response 1");
    if let Some(Payload::Bytes(b)) = resp1.payload {
        assert_eq!(b.data, vec![1]);
    }

    // Verify second response
    let resp2 = timeout(Duration::from_secs(1), out_rx.recv())
        .await
        .expect("Timeout waiting for ping response 2")
        .expect("No ping response 2");
    if let Some(Payload::Bytes(b)) = resp2.payload {
        assert_eq!(b.data, vec![2]);
    }
}

#[tokio::test]
async fn test_portal_tcp_basic() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let (tx, rx) = mpsc::channel(100);
    let (out_tx, mut out_rx) = mpsc::channel(100);
    let sequencer = PayloadSequencer::new("test-stream");

    let first_mote =
        sequencer.new_tcp_mote(vec![1, 2, 3], addr.ip().to_string(), addr.port() as u32);

    let handle =
        tokio::spawn(async move { tcp::handle_tcp(first_mote, rx, out_tx, sequencer).await });

    // Accept connection on the listener
    let (mut socket, _) = timeout(Duration::from_secs(1), listener.accept())
        .await
        .unwrap()
        .unwrap();

    // Verify initial data
    let mut buf = [0u8; 3];
    socket.read_exact(&mut buf).await.unwrap();
    assert_eq!(buf, [1, 2, 3]);

    // Send data from TCP server to agent
    socket.write_all(&[4, 5, 6]).await.unwrap();

    // Verify data received on out_rx
    let mote = timeout(Duration::from_secs(1), out_rx.recv())
        .await
        .expect("Timeout waiting for TCP mote")
        .expect("No TCP mote");
    if let Some(Payload::Tcp(t)) = mote.payload {
        assert_eq!(t.data, vec![4, 5, 6]);
    } else {
        panic!("Expected TCP payload");
    }

    // Send data from C2 to agent
    let sequencer2 = PayloadSequencer::new("test-stream");
    let _ = sequencer2.next_seq_id(); // Skip 0
    let second_mote =
        sequencer2.new_tcp_mote(vec![7, 8, 9], addr.ip().to_string(), addr.port() as u32);
    tx.send(second_mote).await.unwrap();

    // Verify data received by TCP server
    let mut buf = [0u8; 3];
    socket.read_exact(&mut buf).await.unwrap();
    assert_eq!(buf, [7, 8, 9]);

    drop(socket);
    handle.abort();
}

#[tokio::test]
async fn test_portal_run_udp_dispatch() {
    let mut transport = MockTransport::default();
    let task_context = TaskContext {
        task_id: 101,
        jwt: "test-jwt".to_string(),
    };

    let (req_tx_c2, mut req_rx_c2) = mpsc::channel(100);
    let (resp_tx_c2, mut resp_rx_c2) = mpsc::channel(100);

    transport
        .expect_create_portal()
        .return_once(move |mut rx, tx| {
            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        Some(req) = rx.recv() => {
                            req_tx_c2.send(req).await.unwrap();
                        }
                        Some(resp) = resp_rx_c2.recv() => {
                            if tx.send(resp).await.is_err() { break; }
                        }
                        else => break,
                    }
                }
            });
            Ok(())
        });

    let tc_clone = task_context.clone();
    let handle = tokio::spawn(async move { run::run(tc_clone, transport).await });

    // 1. Verify initial registration
    let _ = timeout(Duration::from_secs(1), req_rx_c2.recv()).await;

    // 2. Start a UDP socket to act as listener
    let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // 3. Send UDP Mote from C2
    let mote = Mote {
        stream_id: "udp-stream".to_string(),
        seq_id: 0,
        payload: Some(Payload::Udp(pb::portal::UdpPayload {
            data: vec![1, 2, 3],
            dst_addr: addr.ip().to_string(),
            dst_port: addr.port() as u32,
        })),
    };

    resp_tx_c2
        .send(CreatePortalResponse { mote: Some(mote) })
        .await
        .unwrap();

    // 4. Verify UDP listener receives data
    let mut buf = [0u8; 3];
    let (n, _) = timeout(Duration::from_secs(1), listener.recv_from(&mut buf))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(n, 3);
    assert_eq!(&buf[..3], &[1, 2, 3]);

    handle.abort();
}

#[tokio::test]
async fn test_portal_run_ordered_reader() {
    let mut transport = MockTransport::default();
    let task_context = TaskContext {
        task_id: 789,
        jwt: "test-jwt".to_string(),
    };

    let (req_tx_c2, mut req_rx_c2) = mpsc::channel(100);
    let (resp_tx_c2, mut resp_rx_c2) = mpsc::channel(100);

    transport
        .expect_create_portal()
        .return_once(move |mut rx, tx| {
            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        Some(req) = rx.recv() => {
                            req_tx_c2.send(req).await.unwrap();
                        }
                        Some(resp) = resp_rx_c2.recv() => {
                            if tx.send(resp).await.is_err() { break; }
                        }
                        else => break,
                    }
                }
            });
            Ok(())
        });

    let tc_clone = task_context.clone();
    let handle = tokio::spawn(async move { run::run(tc_clone, transport).await });

    // 1. Verify initial registration
    let _ = timeout(Duration::from_secs(1), req_rx_c2.recv()).await;

    // 2. Send motes out of order
    let mote2 = Mote {
        stream_id: "ordered-stream".to_string(),
        seq_id: 1,
        payload: Some(Payload::Bytes(pb::portal::BytesPayload {
            data: vec![2],
            kind: pb::portal::BytesPayloadKind::Ping as i32,
        })),
    };
    let mote1 = Mote {
        stream_id: "ordered-stream".to_string(),
        seq_id: 0,
        payload: Some(Payload::Bytes(pb::portal::BytesPayload {
            data: vec![1],
            kind: pb::portal::BytesPayloadKind::Ping as i32,
        })),
    };

    resp_tx_c2
        .send(CreatePortalResponse { mote: Some(mote2) })
        .await
        .unwrap();
    resp_tx_c2
        .send(CreatePortalResponse { mote: Some(mote1) })
        .await
        .unwrap();

    // 3. Verify they are handled in order (Ping echos back)
    let resp1 = timeout(Duration::from_secs(1), req_rx_c2.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(resp1.mote.unwrap().seq_id, 0);

    let resp2 = timeout(Duration::from_secs(1), req_rx_c2.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(resp2.mote.unwrap().seq_id, 1);

    handle.abort();
}

#[tokio::test]
async fn test_portal_udp_basic() {
    let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let (tx, rx) = mpsc::channel(100);
    let (out_tx, mut out_rx) = mpsc::channel(100);
    let sequencer = PayloadSequencer::new("test-stream");

    let first_mote =
        sequencer.new_udp_mote(vec![1, 2, 3], addr.ip().to_string(), addr.port() as u32);

    let handle =
        tokio::spawn(async move { udp::handle_udp(first_mote, rx, out_tx, sequencer).await });

    // Verify initial data on listener
    let mut buf = [0u8; 1024];
    let (n, peer) = timeout(Duration::from_secs(1), listener.recv_from(&mut buf))
        .await
        .expect("Timeout waiting for UDP packet")
        .unwrap();
    assert_eq!(&buf[0..n], &[1, 2, 3]);

    // Send data from UDP server to agent
    listener.send_to(&[4, 5, 6], peer).await.unwrap();

    // Verify data received on out_rx
    let mote = timeout(Duration::from_secs(1), out_rx.recv())
        .await
        .expect("Timeout waiting for UDP mote")
        .expect("No UDP mote");
    if let Some(Payload::Udp(u)) = mote.payload {
        assert_eq!(u.data, vec![4, 5, 6]);
    } else {
        panic!("Expected UDP payload");
    }

    // Send data from C2 to agent
    let sequencer2 = PayloadSequencer::new("test-stream");
    let _ = sequencer2.next_seq_id(); // Skip 0
    let second_mote =
        sequencer2.new_udp_mote(vec![7, 8, 9], addr.ip().to_string(), addr.port() as u32);
    tx.send(second_mote).await.unwrap();

    // Verify data received by UDP server
    let (n, _) = timeout(Duration::from_secs(1), listener.recv_from(&mut buf))
        .await
        .expect("Timeout waiting for second UDP packet")
        .unwrap();
    assert_eq!(&buf[0..n], &[7, 8, 9]);

    handle.abort();
}

#[tokio::test]
async fn test_portal_run_trace() {
    let mut transport = MockTransport::default();
    let task_context = TaskContext {
        task_id: 123,
        jwt: "test-jwt".to_string(),
    };

    let (req_tx_c2, mut req_rx_c2) = mpsc::channel(100);
    let (resp_tx_c2, mut resp_rx_c2) = mpsc::channel(100);

    transport
        .expect_create_portal()
        .return_once(move |mut rx, tx| {
            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        Some(req) = rx.recv() => {
                            req_tx_c2.send(req).await.unwrap();
                        }
                        Some(resp) = resp_rx_c2.recv() => {
                            if tx.send(resp).await.is_err() { break; }
                        }
                        else => break,
                    }
                }
            });
            Ok(())
        });

    let tc_clone = task_context.clone();
    let handle = tokio::spawn(async move { run::run(tc_clone, transport).await });

    // 1. Verify initial registration
    let req = timeout(Duration::from_secs(1), req_rx_c2.recv())
        .await
        .expect("Timeout waiting for registration")
        .expect("No registration request");
    assert_eq!(req.context.unwrap().task_id, 123);
    assert!(req.mote.is_none());

    // 2. Send Trace Mote
    let trace_data = TraceData::default();
    let mut buf = Vec::new();
    trace_data.encode(&mut buf).unwrap();

    let mote = Mote {
        stream_id: "trace-stream".to_string(),
        seq_id: 0,
        payload: Some(Payload::Bytes(pb::portal::BytesPayload {
            data: buf,
            kind: pb::portal::BytesPayloadKind::Trace as i32,
        })),
    };

    resp_tx_c2
        .send(CreatePortalResponse { mote: Some(mote) })
        .await
        .unwrap();

    // 3. Verify Echoed Trace Mote
    let req = timeout(Duration::from_secs(1), req_rx_c2.recv())
        .await
        .expect("Timeout waiting for trace response")
        .expect("No trace response");

    let echoed_mote = req.mote.unwrap();
    if let Some(Payload::Bytes(b)) = echoed_mote.payload {
        assert_eq!(b.kind, pb::portal::BytesPayloadKind::Trace as i32);
        let echoed_trace = TraceData::decode(&b.data[..]).unwrap();
        // Should have AgentRecv and AgentSend events
        assert_eq!(echoed_trace.events.len(), 2);
        assert_eq!(
            echoed_trace.events[0].kind,
            TraceEventKind::AgentRecv as i32
        );
        assert_eq!(
            echoed_trace.events[1].kind,
            TraceEventKind::AgentSend as i32
        );
    } else {
        panic!("Expected bytes payload");
    }

    handle.abort();
}

#[tokio::test]
async fn test_portal_run_tcp_dispatch() {
    let mut transport = MockTransport::default();
    let task_context = TaskContext {
        task_id: 456,
        jwt: "test-jwt".to_string(),
    };

    let (req_tx_c2, mut req_rx_c2) = mpsc::channel(100);
    let (resp_tx_c2, mut resp_rx_c2) = mpsc::channel(100);

    transport
        .expect_create_portal()
        .return_once(move |mut rx, tx| {
            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        Some(req) = rx.recv() => {
                            req_tx_c2.send(req).await.unwrap();
                        }
                        Some(resp) = resp_rx_c2.recv() => {
                            if tx.send(resp).await.is_err() { break; }
                        }
                        else => break,
                    }
                }
            });
            Ok(())
        });

    let tc_clone = task_context.clone();
    let handle = tokio::spawn(async move { run::run(tc_clone, transport).await });

    // 1. Verify initial registration
    let _ = timeout(Duration::from_secs(1), req_rx_c2.recv()).await;

    // 2. Start a TCP listener
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // 3. Send TCP Mote from C2
    let mote = Mote {
        stream_id: "tcp-stream".to_string(),
        seq_id: 0,
        payload: Some(Payload::Tcp(pb::portal::TcpPayload {
            data: vec![1, 2, 3],
            dst_addr: addr.ip().to_string(),
            dst_port: addr.port() as u32,
        })),
    };

    resp_tx_c2
        .send(CreatePortalResponse { mote: Some(mote) })
        .await
        .unwrap();

    // 4. Verify TCP listener receives data
    let (mut socket, _) = timeout(Duration::from_secs(1), listener.accept())
        .await
        .unwrap()
        .unwrap();
    let mut buf = [0u8; 3];
    socket.read_exact(&mut buf).await.unwrap();
    assert_eq!(buf, [1, 2, 3]);

    handle.abort();
}
