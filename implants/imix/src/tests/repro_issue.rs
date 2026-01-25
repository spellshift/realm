use anyhow::Result;
use std::time::Duration;
use tokio::net::{TcpListener};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn test_proxy_cold_start() -> Result<()> {
    // 1. Start a mock "Target" server (echo server)
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let target_addr = listener.local_addr()?;

    let target_handle = tokio::spawn(async move {
        loop {
            let (mut socket, _) = listener.accept().await.unwrap();
            tokio::spawn(async move {
                let mut buf = [0; 1024];
                loop {
                    let n = socket.read(&mut buf).await.unwrap();
                    if n == 0 { return; }
                    socket.write_all(&buf[0..n]).await.unwrap();
                }
            });
        }
    });

    // 2. Start the Proxy (This requires mocking the Agent/Transport)

    // We import from crate::portal since we are inside the crate
    use crate::portal::tcp::handle_tcp;
    use pb::portal::{Mote, mote::Payload, TcpPayload};
    use portal_stream::PayloadSequencer;
    use tokio::sync::mpsc;

    let (c2_to_agent_tx, c2_to_agent_rx) = mpsc::channel(10);
    let (agent_to_c2_tx, mut agent_to_c2_rx) = mpsc::channel(10);

    let stream_id = "stream_1".to_string();
    let sequencer = PayloadSequencer::new(stream_id.clone());

    // Create the "First Mote" which initiates connection
    let first_mote = Mote {
        stream_id: stream_id.clone(),
        seq_id: 0,
        payload: Some(Payload::Tcp(TcpPayload {
            dst_addr: target_addr.ip().to_string(),
            dst_port: target_addr.port() as u32,
            data: b"hello".to_vec(), // Initial data
        })),
    };

    // Run handle_tcp in a background task
    let proxy_task = tokio::spawn(async move {
        handle_tcp(first_mote, c2_to_agent_rx, agent_to_c2_tx, sequencer).await
    });

    // Expect response from agent (echoed data)
    let response = tokio::time::timeout(Duration::from_secs(2), agent_to_c2_rx.recv()).await?;

    assert!(response.is_some(), "Should receive response");
    let mote = response.unwrap();
    if let Some(Payload::Tcp(tcp)) = mote.payload {
        assert_eq!(tcp.data, b"hello", "Should receive echoed data");
    } else {
        panic!("Expected TCP payload");
    }

    // Now send a second message
    let second_mote = Mote {
        stream_id: stream_id.clone(),
        seq_id: 1,
        payload: Some(Payload::Tcp(TcpPayload {
            dst_addr: target_addr.ip().to_string(),
            dst_port: target_addr.port() as u32,
            data: b"world".to_vec(),
        })),
    };
    c2_to_agent_tx.send(second_mote).await?;

    let response2 = tokio::time::timeout(Duration::from_secs(2), agent_to_c2_rx.recv()).await?;
    assert!(response2.is_some(), "Should receive second response");

    // Cleanup
    drop(c2_to_agent_tx);
    let _ = proxy_task.await;
    target_handle.abort();

    Ok(())
}
