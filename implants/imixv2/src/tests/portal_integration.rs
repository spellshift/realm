use crate::agent::ImixAgent;
use crate::task::TaskRegistry;
use anyhow::Result;
use eldritch_agent::Agent;
use pb::c2::{CreatePortalRequest, CreatePortalResponse};
use pb::config::Config;
use pb::portal::payload::Payload as PortalPayloadEnum;
use pb::portal::{TcpMessage, UdpMessage};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use transport::MockTransport;

#[tokio::test]
async fn test_portal_integration_tcp_traffic() -> Result<()> {
    // 1. Setup Mock Destination Server (HTTP)
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let dst_port = addr.port() as u32;

    tokio::spawn(async move {
        loop {
            if let Ok((mut socket, _)) = listener.accept().await {
                tokio::spawn(async move {
                    let mut buf = [0u8; 1024];
                    if let Ok(n) = socket.read(&mut buf).await {
                        let request = String::from_utf8_lossy(&buf[..n]);
                        if request.contains("GET / HTTP/1.1") {
                            let response =
                                "HTTP/1.1 200 OK\r\nContent-Length: 12\r\n\r\nHello World!";
                            let _ = socket.write_all(response.as_bytes()).await;
                        }
                    }
                });
            }
        }
    });

    // 2. Setup Mock Transport
    let mut mock_transport = MockTransport::default();

    // Channels to communicate with the agent through the transport (simulating C2)
    let (c2_inbox_tx, mut c2_inbox_rx) = mpsc::channel::<CreatePortalRequest>(100);
    let (c2_outbox_tx, c2_outbox_rx) = mpsc::channel::<CreatePortalResponse>(100);

    // We wrap c2_outbox_rx in Arc<Mutex<Option<...>>> to extract it inside the mock closure
    let c2_outbox_rx_container = Arc::new(tokio::sync::Mutex::new(Some(c2_outbox_rx)));

    // Setup expectations
    let c2_inbox_tx_1 = c2_inbox_tx.clone();
    let c2_outbox_rx_container_1 = c2_outbox_rx_container.clone();

    mock_transport
        .expect_create_portal()
        .times(1)
        .returning(move |mut agent_rx, agent_tx| {
            let c2_inbox_tx = c2_inbox_tx_1.clone();
            let c2_outbox_rx_container = c2_outbox_rx_container_1.clone();

            // Bridge Agent -> C2 Inbox
            tokio::spawn(async move {
                while let Some(req) = agent_rx.recv().await {
                    if c2_inbox_tx.send(req).await.is_err() {
                        break;
                    }
                }
            });

            // Bridge C2 Outbox -> Agent
            tokio::spawn(async move {
                let mut rx_opt = c2_outbox_rx_container.lock().await;
                if let Some(mut rx) = rx_opt.take() {
                    drop(rx_opt); // Release lock
                    while let Some(resp) = rx.recv().await {
                        if agent_tx.send(resp).await.is_err() {
                            break;
                        }
                    }
                }
            });

            Ok(())
        });

    // Allow cloning (mockall clones share expectations)
    mock_transport.expect_clone().returning(move || {
        let mut t = MockTransport::default();

        let c2_inbox_tx = c2_inbox_tx.clone();
        let c2_outbox_rx_container = c2_outbox_rx_container.clone();

        t.expect_create_portal()
            .returning(move |mut agent_rx, agent_tx| {
                let c2_inbox_tx = c2_inbox_tx.clone();
                let c2_outbox_rx_container = c2_outbox_rx_container.clone();

                tokio::spawn(async move {
                    while let Some(req) = agent_rx.recv().await {
                        let _ = c2_inbox_tx.send(req).await;
                    }
                });

                tokio::spawn(async move {
                    let mut rx_opt = c2_outbox_rx_container.lock().await;
                    if let Some(mut rx) = rx_opt.take() {
                        drop(rx_opt);
                        while let Some(resp) = rx.recv().await {
                            let _ = agent_tx.send(resp).await;
                        }
                    }
                });

                Ok(())
            });

        t.expect_is_active().returning(|| true);
        t.expect_name().returning(|| "mock");
        t.expect_list_available()
            .returning(|| vec!["mock".to_string()]);

        t
    });

    mock_transport.expect_is_active().returning(|| true);
    mock_transport.expect_name().returning(|| "mock");
    mock_transport
        .expect_list_available()
        .returning(|| vec!["mock".to_string()]);

    // 3. Init Agent
    let runtime = tokio::runtime::Handle::current();
    let task_registry = Arc::new(TaskRegistry::new());
    let agent = ImixAgent::new(Config::default(), mock_transport, runtime, task_registry);

    // 4. Start Create Portal
    let task_id = 1001;
    agent.start_create_portal(task_id).unwrap();

    // 5. Handshake (Agent sends initial empty request)
    let init_req = tokio::time::timeout(Duration::from_secs(5), c2_inbox_rx.recv())
        .await
        .ok()
        .flatten()
        .ok_or_else(|| anyhow::anyhow!("Timeout or channel closed waiting for init request"))?;

    assert_eq!(init_req.task_id, task_id);
    assert!(init_req.payload.is_none());

    // 6. C2 sends TCP data (HTTP Request) via Portal
    let tcp_data = b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n".to_vec();
    let resp_msg = CreatePortalResponse {
        payload: Some(pb::portal::Payload {
            payload: Some(PortalPayloadEnum::Tcp(TcpMessage {
                data: tcp_data.clone(),
                dst_addr: "127.0.0.1".to_string(),
                dst_port,
                src_id: "12345".to_string(),
                seq_id: 0,
            })),
        }),
    };

    c2_outbox_tx.send(resp_msg).await?;

    // 7. Agent receives, fwd to server, gets response, sends back to C2
    let reply_req = tokio::time::timeout(Duration::from_secs(5), c2_inbox_rx.recv())
        .await
        .ok()
        .flatten()
        .ok_or_else(|| anyhow::anyhow!("Timeout or channel closed waiting for reply"))?;

    assert_eq!(reply_req.task_id, task_id);
    if let Some(PortalPayloadEnum::Tcp(tcp)) = reply_req.payload.unwrap().payload {
        let reply_str = String::from_utf8_lossy(&tcp.data);
        assert!(reply_str.contains("HTTP/1.1 200 OK"));
        assert!(reply_str.contains("Hello World!"));
        assert_eq!(tcp.src_id, "12345".to_string());
    } else {
        panic!("Expected TCP message");
    }

    Ok(())
}

#[tokio::test]
async fn test_portal_integration_udp_traffic() -> Result<()> {
    // 1. Setup Mock Destination Server (UDP)
    let socket = tokio::net::UdpSocket::bind("127.0.0.1:0").await?;
    let addr = socket.local_addr()?;
    let dst_port = addr.port() as u32;

    tokio::spawn(async move {
        let mut buf = [0u8; 1024];
        loop {
            if let Ok((n, peer)) = socket.recv_from(&mut buf).await {
                let msg = &buf[..n];
                if msg == b"PING" {
                    let _ = socket.send_to(b"PONG", peer).await;
                }
            }
        }
    });

    // 2. Setup Mock Transport
    let mut mock_transport = MockTransport::default();
    let (c2_inbox_tx, mut c2_inbox_rx) = mpsc::channel::<CreatePortalRequest>(100);
    let (c2_outbox_tx, c2_outbox_rx) = mpsc::channel::<CreatePortalResponse>(100);
    let c2_outbox_rx_container = Arc::new(tokio::sync::Mutex::new(Some(c2_outbox_rx)));

    // Helper closure to configure mock transport
    // We clone the handles *before* creating the closure to move them in
    let c2_inbox_tx_proto = c2_inbox_tx.clone();
    let c2_outbox_rx_container_proto = c2_outbox_rx_container.clone();

    // We can't reuse closure easily with `move` values.
    // So we just copy paste the config logic.

    let c2_inbox_tx_1 = c2_inbox_tx_proto.clone();
    let c2_outbox_rx_container_1 = c2_outbox_rx_container_proto.clone();

    mock_transport
        .expect_create_portal()
        .returning(move |mut agent_rx, agent_tx| {
            let c2_inbox_tx = c2_inbox_tx_1.clone();
            let c2_outbox_rx_container = c2_outbox_rx_container_1.clone();
            tokio::spawn(async move {
                while let Some(req) = agent_rx.recv().await {
                    let _ = c2_inbox_tx.send(req).await;
                }
            });
            tokio::spawn(async move {
                let mut rx_opt = c2_outbox_rx_container.lock().await;
                if let Some(mut rx) = rx_opt.take() {
                    drop(rx_opt);
                    while let Some(resp) = rx.recv().await {
                        let _ = agent_tx.send(resp).await;
                    }
                }
            });
            Ok(())
        });
    mock_transport.expect_is_active().returning(|| true);
    mock_transport.expect_name().returning(|| "mock");
    mock_transport
        .expect_list_available()
        .returning(|| vec!["mock".to_string()]);

    mock_transport.expect_clone().returning(move || {
        let mut t = MockTransport::default();
        let c2_inbox_tx = c2_inbox_tx_proto.clone();
        let c2_outbox_rx_container = c2_outbox_rx_container_proto.clone();

        t.expect_create_portal()
            .returning(move |mut agent_rx, agent_tx| {
                let c2_inbox_tx = c2_inbox_tx.clone();
                let c2_outbox_rx_container = c2_outbox_rx_container.clone();
                tokio::spawn(async move {
                    while let Some(req) = agent_rx.recv().await {
                        let _ = c2_inbox_tx.send(req).await;
                    }
                });
                tokio::spawn(async move {
                    let mut rx_opt = c2_outbox_rx_container.lock().await;
                    if let Some(mut rx) = rx_opt.take() {
                        drop(rx_opt);
                        while let Some(resp) = rx.recv().await {
                            let _ = agent_tx.send(resp).await;
                        }
                    }
                });
                Ok(())
            });
        t.expect_is_active().returning(|| true);
        t.expect_name().returning(|| "mock");
        t.expect_list_available()
            .returning(|| vec!["mock".to_string()]);
        t
    });

    // 3. Init Agent
    let runtime = tokio::runtime::Handle::current();
    let task_registry = Arc::new(TaskRegistry::new());
    let agent = ImixAgent::new(Config::default(), mock_transport, runtime, task_registry);

    // 4. Start Create Portal
    let task_id = 1002;
    agent.start_create_portal(task_id).unwrap();

    // 5. Handshake
    let init_req = tokio::time::timeout(Duration::from_secs(5), c2_inbox_rx.recv())
        .await
        .ok()
        .flatten()
        .ok_or_else(|| anyhow::anyhow!("Timeout or channel closed waiting for init request"))?;
    assert_eq!(init_req.task_id, task_id);

    // 6. C2 sends UDP data
    let udp_data = b"PING".to_vec();
    let resp_msg = CreatePortalResponse {
        payload: Some(pb::portal::Payload {
            payload: Some(PortalPayloadEnum::Udp(UdpMessage {
                data: udp_data.clone(),
                dst_addr: "127.0.0.1".to_string(),
                dst_port,
                src_id: "54321".to_string(),
                seq_id: 0,
            })),
        }),
    };

    c2_outbox_tx.send(resp_msg).await?;

    // 7. Assert Response
    let reply_req = tokio::time::timeout(Duration::from_secs(5), c2_inbox_rx.recv())
        .await
        .ok()
        .flatten()
        .ok_or_else(|| anyhow::anyhow!("Timeout or channel closed waiting for reply"))?;

    assert_eq!(reply_req.task_id, task_id);
    if let Some(PortalPayloadEnum::Udp(udp)) = reply_req.payload.unwrap().payload {
        assert_eq!(udp.data, b"PONG");
        assert_eq!(udp.src_id, "54321".to_string());
    } else {
        panic!("Expected UDP message");
    }

    Ok(())
}
