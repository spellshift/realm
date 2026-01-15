use pb::portal::{
    BytesPayload, BytesPayloadKind, Mote, ReplPayload, ShellPayload, TcpPayload, UdpPayload,
    mote::Payload,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// PayloadSequencer sequences payloads with a stream ID and monotonic sequence ID.
#[derive(Clone)]
pub struct PayloadSequencer {
    next_seq_id: Arc<AtomicU64>,
    stream_id: String,
}

impl PayloadSequencer {
    /// Creates a new PayloadSequencer with the given stream_id.
    pub fn new(stream_id: impl Into<String>) -> Self {
        Self {
            next_seq_id: Arc::new(AtomicU64::new(0)),
            stream_id: stream_id.into(),
        }
    }

    /// Returns the current sequence ID and increments it.
    pub fn next_seq_id(&self) -> u64 {
        self.next_seq_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Creates a new Mote with a BytesPayload.
    pub fn new_bytes_mote(&self, data: Vec<u8>, kind: BytesPayloadKind) -> Mote {
        Mote {
            stream_id: self.stream_id.clone(),
            seq_id: self.next_seq_id(),
            payload: Some(Payload::Bytes(BytesPayload {
                data,
                kind: kind.into(),
            })),
        }
    }

    /// Creates a new Mote with a TCPPayload.
    pub fn new_tcp_mote(&self, data: Vec<u8>, dst_addr: String, dst_port: u32) -> Mote {
        Mote {
            stream_id: self.stream_id.clone(),
            seq_id: self.next_seq_id(),
            payload: Some(Payload::Tcp(TcpPayload {
                data,
                dst_addr,
                dst_port,
            })),
        }
    }

    /// Creates a new Mote with a UDPPayload.
    pub fn new_udp_mote(&self, data: Vec<u8>, dst_addr: String, dst_port: u32) -> Mote {
        Mote {
            stream_id: self.stream_id.clone(),
            seq_id: self.next_seq_id(),
            payload: Some(Payload::Udp(UdpPayload {
                data,
                dst_addr,
                dst_port,
            })),
        }
    }

    /// Creates a new Mote with a ShellPayload.
    pub fn new_shell_mote(&self, data: Vec<u8>) -> Mote {
        Mote {
            stream_id: self.stream_id.clone(),
            seq_id: self.next_seq_id(),
            payload: Some(Payload::Shell(ShellPayload { data })),
        }
    }

    /// Creates a new Mote with a ReplPayload.
    pub fn new_repl_mote(&self, data: Vec<u8>) -> Mote {
        Mote {
            stream_id: self.stream_id.clone(),
            seq_id: self.next_seq_id(),
            payload: Some(Payload::Repl(ReplPayload { data })),
        }
    }
}
