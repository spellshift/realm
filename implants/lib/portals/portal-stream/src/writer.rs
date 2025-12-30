use crate::sequencer::PayloadSequencer;
use pb::portal::{BytesPayloadKind, Mote};

/// OrderedWriter uses a PayloadSequencer to create motes that are then written to a destination.
pub struct OrderedWriter<F>
where
    F: FnMut(Mote) -> Result<(), String>,
{
    sequencer: PayloadSequencer,
    sender: F,
}

impl<F> OrderedWriter<F>
where
    F: FnMut(Mote) -> Result<(), String>,
{
    /// Creates a new OrderedWriter.
    pub fn new(stream_id: impl Into<String>, sender: F) -> Self {
        Self {
            sequencer: PayloadSequencer::new(stream_id),
            sender,
        }
    }

    /// Creates and writes a BytesMote.
    pub fn write_bytes(&mut self, data: Vec<u8>, kind: BytesPayloadKind) -> Result<(), String> {
        let mote = self.sequencer.new_bytes_mote(data, kind);
        (self.sender)(mote)
    }

    /// Creates and writes a TCPMote.
    pub fn write_tcp(
        &mut self,
        data: Vec<u8>,
        dst_addr: String,
        dst_port: u32,
    ) -> Result<(), String> {
        let mote = self.sequencer.new_tcp_mote(data, dst_addr, dst_port);
        (self.sender)(mote)
    }

    /// Creates and writes a UDPMote.
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
