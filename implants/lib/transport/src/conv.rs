/// Shared conversation-protocol helpers used by DNS and ICMP transports.
use anyhow::Result;
use pb::conv::*;
use prost::Message;

pub const CONV_ID_LENGTH: usize = 8;
pub const SEND_WINDOW_SIZE: usize = 10;
pub const MAX_DATA_SIZE: usize = 50 * 1024 * 1024; // 50MB
pub const MAX_RETRIES_PER_CHUNK: usize = 3;

/// Generate a random 8-character alphanumeric conversation ID.
pub fn generate_conv_id() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..CONV_ID_LENGTH)
        .map(|_| CHARSET[rng.gen_range(0..CHARSET.len())] as char)
        .collect()
}

/// CRC32 using IEEE poly 0xedb88320.
pub fn calculate_crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffffffffu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xedb88320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

/// Split data into fixed-size chunks.
pub fn split_into_chunks(data: &[u8], chunk_size: usize) -> Vec<Vec<u8>> {
    if data.is_empty() {
        return vec![vec![]];
    }
    data.chunks(chunk_size).map(|c| c.to_vec()).collect()
}

/// Build a ConvPacket with the given fields.
pub fn build_conv_packet(
    ptype: PacketType,
    seq: u32,
    conv_id: &str,
    data: Vec<u8>,
    crc32: u32,
) -> ConvPacket {
    ConvPacket {
        r#type: ptype as i32,
        sequence: seq,
        conversation_id: conv_id.to_string(),
        data,
        crc32,
        acks: vec![],
        nacks: vec![],
    }
}

/// Serialize an InitPayload to bytes.
pub fn encode_init_payload(
    method_code: &str,
    total_chunks: u32,
    data_crc32: u32,
    file_size: u32,
) -> Vec<u8> {
    let payload = InitPayload {
        method_code: method_code.to_string(),
        total_chunks,
        data_crc32,
        file_size,
    };
    let mut buf = Vec::new();
    payload.encode(&mut buf).expect("encode InitPayload");
    buf
}

/// Parse a STATUS ConvPacket response → (acked_seqs, nacked_seqs).
pub fn parse_status_response(data: &[u8]) -> Result<(Vec<u32>, Vec<u32>)> {
    let pkt = ConvPacket::decode(data)?;
    let mut acks = Vec::new();
    let mut nacks = Vec::new();
    if pkt.r#type == PacketType::Status as i32 {
        for range in &pkt.acks {
            for seq in range.start_seq..=range.end_seq {
                acks.push(seq);
            }
        }
        nacks.extend_from_slice(&pkt.nacks);
    }
    Ok((acks, nacks))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pb::conv::{AckRange, ConvPacket, InitPayload, PacketType};
    use prost::Message;

    #[test]
    fn test_generate_conv_id_length_and_charset() {
        let id = generate_conv_id();
        assert_eq!(id.len(), CONV_ID_LENGTH);
        assert!(
            id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
            "conv_id must be lowercase alphanumeric, got: {id}"
        );
    }

    #[test]
    fn test_generate_conv_id_unique() {
        // Two IDs generated in sequence should almost certainly differ.
        let id1 = generate_conv_id();
        let id2 = generate_conv_id();
        // This could theoretically fail 1 in 36^8 runs; acceptable for a smoke test.
        assert_ne!(id1, id2, "two generated IDs should differ");
    }

    #[test]
    fn test_calculate_crc32_empty() {
        // Empty input: crc starts at 0xFFFFFFFF, no bytes processed, !0xFFFFFFFF = 0.
        assert_eq!(calculate_crc32(b""), 0x00000000);
    }

    #[test]
    fn test_calculate_crc32_known_value() {
        // Standard CRC-32/ISO-HDLC check value for the ASCII string "123456789".
        assert_eq!(calculate_crc32(b"123456789"), 0xCBF43926);
    }

    #[test]
    fn test_calculate_crc32_deterministic() {
        let data = b"hello world";
        assert_eq!(calculate_crc32(data), calculate_crc32(data));
    }

    #[test]
    fn test_split_into_chunks_basic() {
        let data: Vec<u8> = (0..10).collect();
        let chunks = split_into_chunks(&data, 3);
        assert_eq!(chunks.len(), 4);
        assert_eq!(chunks[0], vec![0, 1, 2]);
        assert_eq!(chunks[1], vec![3, 4, 5]);
        assert_eq!(chunks[2], vec![6, 7, 8]);
        assert_eq!(chunks[3], vec![9]);
    }

    #[test]
    fn test_split_into_chunks_empty() {
        let chunks = split_into_chunks(b"", 100);
        assert_eq!(chunks, vec![Vec::<u8>::new()]);
    }

    #[test]
    fn test_split_into_chunks_exact_boundary() {
        let data = vec![1u8, 2, 3];
        let chunks = split_into_chunks(&data, 3);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], vec![1, 2, 3]);
    }

    #[test]
    fn test_build_conv_packet() {
        let pkt = build_conv_packet(PacketType::Init, 0, "testconv", vec![1, 2, 3], 0xABCD);
        assert_eq!(pkt.r#type, PacketType::Init as i32);
        assert_eq!(pkt.sequence, 0);
        assert_eq!(pkt.conversation_id, "testconv");
        assert_eq!(pkt.data, vec![1, 2, 3]);
        assert_eq!(pkt.crc32, 0xABCD);
        assert!(pkt.acks.is_empty());
        assert!(pkt.nacks.is_empty());
    }

    #[test]
    fn test_encode_init_payload_roundtrip() {
        let bytes = encode_init_payload("/c2.C2/ClaimTasks", 5, 0x12345678, 1024);
        let decoded = InitPayload::decode(bytes.as_slice()).expect("decode InitPayload");
        assert_eq!(decoded.method_code, "/c2.C2/ClaimTasks");
        assert_eq!(decoded.total_chunks, 5);
        assert_eq!(decoded.data_crc32, 0x12345678);
        assert_eq!(decoded.file_size, 1024);
    }

    #[test]
    fn test_parse_status_response_acks_and_nacks() {
        let pkt = ConvPacket {
            r#type: PacketType::Status as i32,
            acks: vec![AckRange { start_seq: 1, end_seq: 3 }],
            nacks: vec![5, 6],
            ..Default::default()
        };
        let mut buf = Vec::new();
        pkt.encode(&mut buf).expect("encode ConvPacket");

        let (acks, nacks) = parse_status_response(&buf).expect("parse_status_response");
        assert_eq!(acks, vec![1, 2, 3]);
        assert_eq!(nacks, vec![5, 6]);
    }

    #[test]
    fn test_parse_status_response_non_status_returns_empty() {
        let pkt = ConvPacket {
            r#type: PacketType::Data as i32,
            ..Default::default()
        };
        let mut buf = Vec::new();
        pkt.encode(&mut buf).expect("encode ConvPacket");

        let (acks, nacks) = parse_status_response(&buf).expect("parse_status_response");
        assert!(acks.is_empty());
        assert!(nacks.is_empty());
    }

    #[test]
    fn test_parse_status_response_invalid_bytes() {
        let result = parse_status_response(&[0xFF, 0xFF, 0xFF, 0xFF]);
        assert!(result.is_err());
    }
}
