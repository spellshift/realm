use anyhow::Result;
use pb::portal::{BytesPayloadKind, Mote, mote::Payload};
use portal_stream::PayloadSequencer;
use tokio::sync::mpsc;

pub async fn handle_bytes(
    first_mote: Mote,
    mut rx: mpsc::Receiver<Mote>,
    out_tx: mpsc::Sender<Mote>,
    sequencer: PayloadSequencer,
) -> Result<()> {
    // Process first mote
    process_byte_mote(first_mote, &out_tx, &sequencer).await?;

    // Loop
    while let Some(mote) = rx.recv().await {
        process_byte_mote(mote, &out_tx, &sequencer).await?;
    }

    Ok(())
}

async fn process_byte_mote(
    mote: Mote,
    out_tx: &mpsc::Sender<Mote>,
    sequencer: &PayloadSequencer,
) -> Result<()> {
    if let Some(Payload::Bytes(b)) = mote.payload {
        match BytesPayloadKind::try_from(b.kind).ok() {
            Some(BytesPayloadKind::Ping) => {
                // Echo back with same data
                let resp = sequencer.new_bytes_mote(b.data, BytesPayloadKind::Ping);
                out_tx
                    .send(resp)
                    .await
                    .map_err(|e| anyhow::anyhow!("Send failed: {}", e))?;
            }
            Some(BytesPayloadKind::Keepalive) => {
                // Ignore
            }
            _ => {
                // Ignore Data/Unspecified for now as per requirements
            }
        }
    }
    Ok(())
}
