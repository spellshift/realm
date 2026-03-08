use anyhow::{Context, Result};
use pb::portal::{Mote, mote::Payload};
use portal_stream::PayloadSequencer;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;

const BUF_SIZE: usize = 64 * 1024;

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

    #[cfg(debug_assertions)]
    log::debug!("Setting up UDP for {}", target_addr);

    // Bind to 0.0.0.0:0
    let socket = UdpSocket::bind("0.0.0.0:0")
        .await
        .context("Failed to bind UDP")?;

    #[cfg(debug_assertions)]
    log::debug!("UDP bound to {:?}", socket.local_addr());

    socket
        .connect(&target_addr)
        .await
        .context("Failed to connect UDP")?;

    #[cfg(debug_assertions)]
    log::info!("UDP connected to {}", target_addr);

    let socket = Arc::new(socket);

    // If initial data exists, send it
    if !initial_data.is_empty() {
        socket.send(&initial_data).await?;
    }

    // Spawn reader task (Socket -> C2)
    let socket_read = socket.clone();
    let out_tx_clone = out_tx.clone();
    let dst_addr_clone = dst_addr.clone();

    #[cfg(debug_assertions)]
    let addr_for_read = target_addr.clone();

    let read_task = tokio::spawn(async move {
        let mut buf = [0u8; BUF_SIZE];
        loop {
            match socket_read.recv(&mut buf).await {
                Ok(n) => {
                    let data = buf[0..n].to_vec();
                    let mote = sequencer.new_udp_mote(data, dst_addr_clone.clone(), dst_port);
                    if out_tx_clone.send(mote).await.is_err() {
                        #[cfg(debug_assertions)]
                        log::warn!(
                            "Failed to send UDP mote to C2 (channel closed) for {}",
                            addr_for_read
                        );
                        break;
                    }
                }
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    log::error!("failed to read from udp socket ({}): {_e:?}", addr_for_read);
                }
            }
        }
    });

    // Write Loop (C2 -> Socket)
    while let Some(mote) = rx.recv().await {
        if let Some(Payload::Udp(udp)) = mote.payload
            && !udp.data.is_empty()
        {
            match socket.send(&udp.data).await {
                Ok(_) => {}
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    log::error!("failed to write udp to {}: {_e:?}", target_addr);

                    break;
                }
            }
        }
    }

    // Cleanup
    read_task.abort(); // UDP recv might block forever

    Ok(())
}
