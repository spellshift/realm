use crate::sequencer::PayloadSequencer;
use pb::portal::{BytesPayloadKind, Mote};

/// OrderedWriter uses a PayloadSequencer to create motes that are then written to a destination.
pub struct OrderedWriter<F> {
    sequencer: PayloadSequencer,
    sender: F,
}

impl<F> OrderedWriter<F> {
    /// Creates a new OrderedWriter.
    pub fn new(stream_id: impl Into<String>, sender: F) -> Self {
        Self {
            sequencer: PayloadSequencer::new(stream_id),
            sender,
        }
    }
}

impl<F> OrderedWriter<F>
where
    F: FnMut(Mote) -> Result<(), String>,
{
    /// Creates and writes a BytesMote (synchronous).
    pub fn write_bytes(&mut self, data: Vec<u8>, kind: BytesPayloadKind) -> Result<(), String> {
        let mote = self.sequencer.new_bytes_mote(data, kind);
        (self.sender)(mote)
    }

    /// Creates and writes a TCPMote (synchronous).
    pub fn write_tcp(
        &mut self,
        data: Vec<u8>,
        dst_addr: String,
        dst_port: u32,
    ) -> Result<(), String> {
        let mote = self.sequencer.new_tcp_mote(data, dst_addr, dst_port);
        (self.sender)(mote)
    }

    /// Creates and writes a UDPMote (synchronous).
    pub fn write_udp(
        &mut self,
        data: Vec<u8>,
        dst_addr: String,
        dst_port: u32,
    ) -> Result<(), String> {
        let mote = self.sequencer.new_udp_mote(data, dst_addr, dst_port);
        (self.sender)(mote)
    }
}

// Specialization for tokio::sync::mpsc::Sender
#[cfg(feature = "tokio")]
impl OrderedWriter<tokio::sync::mpsc::Sender<Mote>> {
    /// Creates and writes a BytesMote (async).
    pub async fn write_bytes_async(
        &mut self,
        data: Vec<u8>,
        kind: BytesPayloadKind,
    ) -> Result<(), String> {
        let mote = self.sequencer.new_bytes_mote(data, kind);
        self.sender.send(mote).await.map_err(|e| e.to_string())
    }

    /// Creates and writes a TCPMote (async).
    pub async fn write_tcp_async(
        &mut self,
        data: Vec<u8>,
        dst_addr: String,
        dst_port: u32,
    ) -> Result<(), String> {
        let mote = self.sequencer.new_tcp_mote(data, dst_addr, dst_port);
        self.sender.send(mote).await.map_err(|e| e.to_string())
    }

    /// Creates and writes a UDPMote (async).
    pub async fn write_udp_async(
        &mut self,
        data: Vec<u8>,
        dst_addr: String,
        dst_port: u32,
    ) -> Result<(), String> {
        let mote = self.sequencer.new_udp_mote(data, dst_addr, dst_port);
        self.sender.send(mote).await.map_err(|e| e.to_string())
    }
}
