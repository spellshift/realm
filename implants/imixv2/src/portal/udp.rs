use anyhow::{Context, Result};
use pb::portal::{Mote, mote::Payload};
use portal_stream::PayloadSequencer;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;

pub async fn handle_udp(
    first_mote: Mote,
    mut rx: mpsc::Receiver<Mote>,
    out_tx: mpsc::Sender<Mote>,
    sequencer: PayloadSequencer,
) -> Result<()> {
    // Extract destination from first mote
    let (dst_addr, dst_port, initial_data) = if let Some(Payload::Udp(udp)) = first_mote.payload {
        (udp.dst_addr, udp.dst_port, udp.data)
    } else {
        return Err(anyhow::anyhow!("Expected UDP payload"));
    };

    let target_addr = format!("{}:{}", dst_addr, dst_port);

    // Bind to 0.0.0.0:0
    let socket = UdpSocket::bind("0.0.0.0:0")
        .await
        .context("Failed to bind UDP")?;
    socket
        .connect(&target_addr)
        .await
        .context("Failed to connect UDP")?;

    let socket = Arc::new(socket);

    // If initial data exists, send it
    if !initial_data.is_empty() {
        socket.send(&initial_data).await?;
    }

    // Spawn reader task (Socket -> C2)
    let socket_read = socket.clone();
    let out_tx_clone = out_tx.clone();
    let dst_addr_clone = dst_addr.clone();

    let read_task = tokio::spawn(async move {
        let mut buf = [0u8; 4096];
        loop {
            match socket_read.recv(&mut buf).await {
                Ok(n) => {
                    let data = buf[0..n].to_vec();
                    let mote = sequencer.new_udp_mote(data, dst_addr_clone.clone(), dst_port);
                    if out_tx_clone.send(mote).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    // Write Loop (C2 -> Socket)
    while let Some(mote) = rx.recv().await {
        if let Some(Payload::Udp(udp)) = mote.payload {
            if !udp.data.is_empty() {
                if socket.send(&udp.data).await.is_err() {
                    break;
                }
            }
        }
    }

    // Cleanup
    let _ = read_task.abort(); // UDP recv might block forever

    Ok(())
}
