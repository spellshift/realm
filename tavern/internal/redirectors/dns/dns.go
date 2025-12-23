package dns

import (
	"context"
	"encoding/base32"
	"encoding/binary"
	"errors"
	"fmt"
	"hash/crc32"
	"io"
	"log/slog"
	"net"
	"net/url"
	"sort"
	"strings"
	"sync"
	"sync/atomic"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/metadata"
	"google.golang.org/protobuf/proto"
	"realm.pub/tavern/internal/c2/dnspb"
	"realm.pub/tavern/internal/redirectors"
)

const (
	convTimeout    = 15 * time.Minute
	defaultUDPPort = "53"

	// DNS protocol constants
	dnsHeaderSize  = 12
	maxLabelLength = 63
	txtRecordType  = 16
	aRecordType    = 1
	aaaaRecordType = 28
	dnsClassIN     = 1
	dnsTTLSeconds  = 60

	// DNS response flags
	dnsResponseFlags = 0x8180
	dnsErrorFlags    = 0x8183
	dnsPointer       = 0xC00C

	txtMaxChunkSize = 255

	// Async protocol configuration
	MaxActiveConversations     = 10000
	NormalConversationTimeout  = 15 * time.Minute
	ReducedConversationTimeout = 5 * time.Minute
	CapacityRecoveryThreshold  = 0.5 // 50%
	MaxAckRangesInResponse     = 20
	MaxNacksInResponse         = 50
	MaxDataSize                = 50 * 1024 * 1024 // 50MB max data size
)

func init() {
	redirectors.Register("dns", &Redirector{})
}

// Redirector handles DNS-based C2 communication
type Redirector struct {
	conversations       sync.Map
	baseDomains         []string
	conversationCount   int32 // Atomic counter for active conversations
	conversationTimeout time.Duration
}

// Conversation tracks state for a request-response exchange
type Conversation struct {
	mu               sync.Mutex
	ID               string
	MethodPath       string
	TotalChunks      uint32
	ExpectedCRC      uint32
	ExpectedDataSize uint32 // Data size provided by client
	Chunks           map[uint32][]byte
	LastActivity     time.Time
	ResponseData     []byte
	ResponseChunks   [][]byte // Split response for multi-fetch
	ResponseCRC      uint32
	Completed        bool // Set to true when all chunks received
}

func (r *Redirector) Redirect(ctx context.Context, listenOn string, upstream *grpc.ClientConn) error {
	listenAddr, domains, err := ParseListenAddr(listenOn)
	if err != nil {
		return fmt.Errorf("failed to parse listen address: %w", err)
	}

	if len(domains) == 0 {
		return fmt.Errorf("no base domains specified in listenOn parameter")
	}

	r.baseDomains = domains
	r.conversationTimeout = NormalConversationTimeout

	udpAddr, err := net.ResolveUDPAddr("udp", listenAddr)
	if err != nil {
		return fmt.Errorf("failed to resolve UDP address: %w", err)
	}

	conn, err := net.ListenUDP("udp", udpAddr)
	if err != nil {
		return fmt.Errorf("failed to listen on UDP: %w", err)
	}
	defer conn.Close()

	slog.Info("DNS redirector started", "listen_on", listenAddr, "base_domains", r.baseDomains)

	go r.cleanupConversations(ctx)

	buf := make([]byte, 4096)
	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
			conn.SetReadDeadline(time.Now().Add(1 * time.Second))

			n, addr, err := conn.ReadFromUDP(buf)
			if err != nil {
				if netErr, ok := err.(net.Error); ok && netErr.Timeout() {
					continue
				}
				slog.Error("failed to read UDP", "error", err)
				continue
			}

			go r.handleDNSQuery(ctx, conn, addr, buf[:n], upstream)
		}
	}
}

// ParseListenAddr extracts address and domain parameters from listenOn string
func ParseListenAddr(listenOn string) (string, []string, error) {
	parts := strings.SplitN(listenOn, "?", 2)
	addr := parts[0]

	if !strings.Contains(addr, ":") {
		addr = net.JoinHostPort(addr, defaultUDPPort)
	}

	if len(parts) == 1 {
		return addr, nil, nil
	}

	queryParams := parts[1]
	domains := []string{}

	for _, param := range strings.Split(queryParams, "&") {
		kv := strings.SplitN(param, "=", 2)
		if len(kv) != 2 {
			continue
		}

		key := kv[0]
		value := kv[1]

		if key == "domain" && value != "" {
			decoded, err := url.QueryUnescape(value)
			if err != nil {
				return "", nil, fmt.Errorf("failed to decode domain: %w", err)
			}
			domains = append(domains, decoded)
		}
	}

	return addr, domains, nil
}

func (r *Redirector) cleanupConversations(ctx context.Context) {
	ticker := time.NewTicker(1 * time.Minute)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			now := time.Now()
			count := atomic.LoadInt32(&r.conversationCount)

			// Adjust timeout based on capacity
			if count >= MaxActiveConversations {
				r.conversationTimeout = ReducedConversationTimeout
			} else if float64(count) < float64(MaxActiveConversations)*CapacityRecoveryThreshold {
				r.conversationTimeout = NormalConversationTimeout
			}

			r.conversations.Range(func(key, value interface{}) bool {
				conv := value.(*Conversation)
				conv.mu.Lock()
				if now.Sub(conv.LastActivity) > r.conversationTimeout {
					r.conversations.Delete(key)
					atomic.AddInt32(&r.conversationCount, -1)
				}
				conv.mu.Unlock()
				return true
			})
		}
	}
}

func (r *Redirector) handleDNSQuery(ctx context.Context, conn *net.UDPConn, addr *net.UDPAddr, query []byte, upstream *grpc.ClientConn) {
	if len(query) < dnsHeaderSize {
		slog.Debug("query too short")
		return
	}

	transactionID := binary.BigEndian.Uint16(query[0:2])

	domain, queryType, err := r.parseDomainNameAndType(query[dnsHeaderSize:])
	if err != nil {
		slog.Debug("failed to parse domain", "error", err)
		return
	}

	domain = strings.ToLower(domain)

	// Log ALL queries to track Cloudflare filtering patterns
	if queryType == txtRecordType {
		slog.Info("TXT query received", "domain", domain, "from", addr.String())
	} else {
		slog.Debug("received DNS query", "domain", domain, "query_type", queryType, "from", addr.String())
	}

	// Extract subdomain
	subdomain, err := r.extractSubdomain(domain)
	if err != nil {
		slog.Debug("domain doesn't match base domains", "domain", domain)
		r.sendErrorResponse(conn, addr, transactionID)
		return
	}

	// Decode packet
	packet, err := r.decodePacket(subdomain)
	if err != nil {
		// Silently drop queries that fail to decode - likely legitimate DNS queries or probes
		// Cloudflare forwards all queries under our zone, not just C2 traffic
		slog.Debug("ignoring non-C2 query", "domain", domain, "error", err)

		// For A record queries, return benign IP (127.0.0.1) instead of NXDOMAIN
		// Cloudflare does recursive lookups on subdomain components - if we return NXDOMAIN
		// for the parent subdomain, it won't forward the full TXT query for INIT packets
		if queryType == aRecordType {
			slog.Debug("returning benign A record for non-C2 subdomain", "domain", domain)
			r.sendDNSResponse(conn, addr, transactionID, domain, queryType, []byte{127, 0, 0, 1})
			return
		}

		// For other types, return NXDOMAIN
		r.sendErrorResponse(conn, addr, transactionID)
		return
	}

	// Validate packet type before processing
	if packet.Type == dnspb.PacketType_PACKET_TYPE_UNSPECIFIED {
		// Invalid/empty packet - likely parsing artifact from random domain
		slog.Debug("ignoring packet with unspecified type", "domain", domain)

		// Return benign A record for A queries to satisfy Cloudflare recursive lookups
		if queryType == aRecordType {
			r.sendDNSResponse(conn, addr, transactionID, domain, queryType, []byte{127, 0, 0, 1})
			return
		}

		r.sendErrorResponse(conn, addr, transactionID)
		return
	}

	slog.Debug("parsed packet", "type", packet.Type, "seq", packet.Sequence, "conv_id", packet.ConversationId)

	// Handle packet based on type
	var responseData []byte
	switch packet.Type {
	case dnspb.PacketType_PACKET_TYPE_INIT:
		responseData, err = r.handleInitPacket(packet)
	case dnspb.PacketType_PACKET_TYPE_DATA:
		responseData, err = r.handleDataPacket(ctx, upstream, packet, queryType)
	case dnspb.PacketType_PACKET_TYPE_FETCH:
		responseData, err = r.handleFetchPacket(packet)
	default:
		err = fmt.Errorf("unknown packet type: %d", packet.Type)
	}

	if err != nil {
		// Log as WARN since conversation-not-found is expected with UDP packet loss
		slog.Warn("packet handling failed", "type", packet.Type, "conv_id", packet.ConversationId, "error", err)
		r.sendErrorResponse(conn, addr, transactionID)
		return
	}

	r.sendDNSResponse(conn, addr, transactionID, domain, queryType, responseData)
}

func (r *Redirector) extractSubdomain(domain string) (string, error) {
	domainParts := strings.Split(domain, ".")

	for _, baseDomain := range r.baseDomains {
		baseDomainParts := strings.Split(baseDomain, ".")

		if len(domainParts) <= len(baseDomainParts) {
			continue
		}

		domainSuffix := domainParts[len(domainParts)-len(baseDomainParts):]
		matched := true
		for i, part := range baseDomainParts {
			if !strings.EqualFold(part, domainSuffix[i]) {
				matched = false
				break
			}
		}

		if matched {
			subdomainParts := domainParts[:len(domainParts)-len(baseDomainParts)]
			return strings.Join(subdomainParts, "."), nil
		}
	}

	return "", fmt.Errorf("no matching base domain")
}

// decodePacket decodes DNS packet from subdomain
// Subdomain format: <base32_encoded_protobuf_packet>.<base_domain>
// The entire protobuf packet is base32-encoded and split into 63-char labels
func (r *Redirector) decodePacket(subdomain string) (*dnspb.DNSPacket, error) {
	// Remove all dots to get continuous base32 string
	// Labels were split at 63-char boundaries, now rejoin them
	encodedData := strings.ReplaceAll(subdomain, ".", "")

	// Decode data using Base32 (case-insensitive, no padding)
	packetData, err := base32.StdEncoding.WithPadding(base32.NoPadding).DecodeString(strings.ToUpper(encodedData))
	if err != nil {
		return nil, fmt.Errorf("failed to decode Base32 data: %w", err)
	}

	// Unmarshal protobuf
	var packet dnspb.DNSPacket
	if err := proto.Unmarshal(packetData, &packet); err != nil {
		return nil, fmt.Errorf("failed to unmarshal protobuf: %w", err)
	}

	// Verify CRC for data packets
	if packet.Type == dnspb.PacketType_PACKET_TYPE_DATA && len(packet.Data) > 0 {
		actualCRC := crc32.ChecksumIEEE(packet.Data)
		if actualCRC != packet.Crc32 {
			return nil, fmt.Errorf("CRC mismatch: expected %d, got %d", packet.Crc32, actualCRC)
		}
	}

	return &packet, nil
}

// handleInitPacket processes INIT packet
func (r *Redirector) handleInitPacket(packet *dnspb.DNSPacket) ([]byte, error) {
	// Atomically check and increment conversation count
	// Loop until we successfully increment or hit the limit
	for {
		current := atomic.LoadInt32(&r.conversationCount)
		if current >= MaxActiveConversations {
			return nil, fmt.Errorf("max active conversations reached: %d", current)
		}
		// Try to increment atomically
		if atomic.CompareAndSwapInt32(&r.conversationCount, current, current+1) {
			// Successfully incremented, break out
			break
		}
		// CAS failed (another goroutine modified the value), retry
	}

	// Unmarshal init payload
	var initPayload dnspb.InitPayload
	if err := proto.Unmarshal(packet.Data, &initPayload); err != nil {
		// Decrement on error since we already incremented
		atomic.AddInt32(&r.conversationCount, -1)
		return nil, fmt.Errorf("failed to unmarshal init payload: %w", err)
	}

	// Validate file size from client
	if initPayload.FileSize > MaxDataSize {
		atomic.AddInt32(&r.conversationCount, -1)
		return nil, fmt.Errorf("data size exceeds maximum: %d > %d bytes", initPayload.FileSize, MaxDataSize)
	}

	// Validate that FileSize is set (protobuf default is 0)
	if initPayload.FileSize == 0 && initPayload.TotalChunks > 0 {
		slog.Warn("INIT packet missing file_size field", "conv_id", packet.ConversationId, "total_chunks", initPayload.TotalChunks)
	}

	slog.Debug("creating conversation", "conv_id", packet.ConversationId, "method", initPayload.MethodCode,
		"total_chunks", initPayload.TotalChunks, "file_size", initPayload.FileSize, "crc32", initPayload.DataCrc32)

	// Create conversation
	conv := &Conversation{
		ID:               packet.ConversationId,
		MethodPath:       initPayload.MethodCode,
		TotalChunks:      initPayload.TotalChunks,
		ExpectedCRC:      initPayload.DataCrc32,
		ExpectedDataSize: initPayload.FileSize,
		Chunks:           make(map[uint32][]byte),
		LastActivity:     time.Now(),
		Completed:        false,
	}

	r.conversations.Store(packet.ConversationId, conv)

	slog.Info("C2 conversation started", "conv_id", conv.ID, "method", conv.MethodPath,
		"total_chunks", conv.TotalChunks, "data_size", initPayload.FileSize)

	// Return empty STATUS packet (no ACKs/NACKs yet) to look like legitimate DNS data
	// Don't return plain text "ok" which could trigger Cloudflare filters
	statusPacket := &dnspb.DNSPacket{
		Type:           dnspb.PacketType_PACKET_TYPE_STATUS,
		ConversationId: packet.ConversationId,
		Acks:           []*dnspb.AckRange{},
		Nacks:          []uint32{},
	}
	statusData, err := proto.Marshal(statusPacket)
	if err != nil {
		atomic.AddInt32(&r.conversationCount, -1)
		r.conversations.Delete(packet.ConversationId)
		return nil, fmt.Errorf("failed to marshal init status: %w", err)
	}
	return statusData, nil
}

// handleDataPacket processes DATA packet
func (r *Redirector) handleDataPacket(ctx context.Context, upstream *grpc.ClientConn, packet *dnspb.DNSPacket, queryType uint16) ([]byte, error) {
	val, ok := r.conversations.Load(packet.ConversationId)
	if !ok {
		// Log at debug - this is normal with UDP packet loss/reordering (INIT may arrive later)
		slog.Debug("DATA packet for unknown conversation (INIT may be lost/delayed)",
			"conv_id", packet.ConversationId, "seq", packet.Sequence)
		return nil, fmt.Errorf("conversation not found: %s", packet.ConversationId)
	}

	conv := val.(*Conversation)
	conv.mu.Lock()
	defer conv.mu.Unlock()

	// Validate sequence number
	if packet.Sequence < 1 || packet.Sequence > conv.TotalChunks {
		return nil, fmt.Errorf("sequence out of bounds: %d (expected 1-%d)", packet.Sequence, conv.TotalChunks)
	}

	// Store chunk (sequence is 1-indexed, overwrites duplicates safely)
	conv.Chunks[packet.Sequence] = packet.Data
	conv.LastActivity = time.Now()

	slog.Debug("received chunk", "conv_id", conv.ID, "seq", packet.Sequence, "size", len(packet.Data), "total", len(conv.Chunks))

	// Check if conversation is complete and auto-process
	if uint32(len(conv.Chunks)) == conv.TotalChunks && !conv.Completed {
		conv.Completed = true
		slog.Info("C2 request complete, forwarding to upstream", "conv_id", conv.ID,
			"method", conv.MethodPath, "total_chunks", conv.TotalChunks, "data_size", conv.ExpectedDataSize)

		// Unlock before calling processCompletedConversation (it will re-lock)
		conv.mu.Unlock()
		if err := r.processCompletedConversation(ctx, upstream, conv, queryType); err != nil {
			slog.Error("failed to process completed conversation", "conv_id", conv.ID, "error", err)
		}
		conv.mu.Lock()
	}

	// Build ACK/NACK response (STATUS packet)
	acks, nacks := r.computeAcksNacks(conv)

	statusPacket := &dnspb.DNSPacket{
		Type:           dnspb.PacketType_PACKET_TYPE_STATUS,
		ConversationId: packet.ConversationId,
		Acks:           acks,
		Nacks:          nacks,
	}

	// Marshal STATUS packet to return as response
	statusData, err := proto.Marshal(statusPacket)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal status packet: %w", err)
	}

	return statusData, nil
}

// processCompletedConversation reassembles data, verifies CRC, forwards to upstream, and stores response
func (r *Redirector) processCompletedConversation(ctx context.Context, upstream *grpc.ClientConn, conv *Conversation, queryType uint16) error {
	conv.mu.Lock()
	defer conv.mu.Unlock()

	// Reassemble data (chunks are 1-indexed)
	var fullData []byte
	for i := uint32(1); i <= conv.TotalChunks; i++ {
		chunk, ok := conv.Chunks[i]
		if !ok {
			return fmt.Errorf("missing chunk %d", i)
		}
		fullData = append(fullData, chunk...)
	}

	// Verify CRC
	actualCRC := crc32.ChecksumIEEE(fullData)
	if actualCRC != conv.ExpectedCRC {
		// Clean up on fatal error
		r.conversations.Delete(conv.ID)
		atomic.AddInt32(&r.conversationCount, -1)
		return fmt.Errorf("data CRC mismatch: expected %d, got %d", conv.ExpectedCRC, actualCRC)
	}

	slog.Debug("reassembled data", "conv_id", conv.ID, "size", len(fullData), "method", conv.MethodPath)

	// Validate reassembled size matches client-provided data size (if provided)
	if conv.ExpectedDataSize > 0 && uint32(len(fullData)) != conv.ExpectedDataSize {
		// Clean up on fatal error
		r.conversations.Delete(conv.ID)
		atomic.AddInt32(&r.conversationCount, -1)
		return fmt.Errorf("reassembled data size mismatch: expected %d bytes, got %d bytes", conv.ExpectedDataSize, len(fullData))
	}

	// Forward to upstream gRPC server
	responseData, err := r.forwardToUpstream(ctx, upstream, conv.MethodPath, fullData)
	if err != nil {
		// Clean up on fatal error
		r.conversations.Delete(conv.ID)
		atomic.AddInt32(&r.conversationCount, -1)
		return fmt.Errorf("failed to forward to upstream: %w", err)
	}

	// Determine max response size based on record type
	var maxSize int
	switch queryType {
	case txtRecordType:
		maxSize = 400
	case aRecordType:
		maxSize = 64
	case aaaaRecordType:
		maxSize = 128
	default:
		maxSize = 400
	}

	// Check if response needs chunking
	if len(responseData) > maxSize {
		// Calculate CRC for full response
		conv.ResponseCRC = crc32.ChecksumIEEE(responseData)
		conv.ResponseData = responseData

		// Split into chunks
		conv.ResponseChunks = nil
		for i := 0; i < len(responseData); i += maxSize {
			end := i + maxSize
			if end > len(responseData) {
				end = len(responseData)
			}
			conv.ResponseChunks = append(conv.ResponseChunks, responseData[i:end])
		}

		conv.LastActivity = time.Now()

		slog.Debug("response chunked", "conv_id", conv.ID, "total_size", len(responseData),
			"chunks", len(conv.ResponseChunks), "crc32", conv.ResponseCRC)
	} else {
		// Response fits in single packet
		conv.ResponseData = responseData
		conv.LastActivity = time.Now()

		slog.Debug("stored response", "conv_id", conv.ID, "size", len(responseData))
	}

	return nil
}

// computeAcksNacks computes ACK ranges and NACK list for a conversation
// Must be called with conv.mu locked
func (r *Redirector) computeAcksNacks(conv *Conversation) ([]*dnspb.AckRange, []uint32) {
	// Build sorted list of received sequences
	received := make([]uint32, 0, len(conv.Chunks))
	for seq := range conv.Chunks {
		received = append(received, seq)
	}
	sort.Slice(received, func(i, j int) bool { return received[i] < received[j] })

	// Compute ACK ranges (contiguous blocks)
	acks := []*dnspb.AckRange{}
	if len(received) > 0 {
		start := received[0]
		end := received[0]

		for i := 1; i < len(received); i++ {
			if received[i] == end+1 {
				end = received[i]
			} else {
				acks = append(acks, &dnspb.AckRange{StartSeq: start, EndSeq: end})
				start = received[i]
				end = received[i]
			}
		}
		acks = append(acks, &dnspb.AckRange{StartSeq: start, EndSeq: end})
	}

	// Limit ACK ranges
	if len(acks) > MaxAckRangesInResponse {
		acks = acks[:MaxAckRangesInResponse]
	}

	// Compute NACKs (missing sequences in gaps)
	nacks := []uint32{}

	if len(received) > 0 {
		// Find gaps between first and last received
		minReceived := received[0]
		maxReceived := received[len(received)-1]

		receivedSet := make(map[uint32]bool)
		for _, seq := range received {
			receivedSet[seq] = true
		}

		for seq := minReceived; seq <= maxReceived; seq++ {
			if !receivedSet[seq] {
				nacks = append(nacks, seq)
				if len(nacks) >= MaxNacksInResponse {
					break
				}
			}
		}
	}

	return acks, nacks
}

// handleFetchPacket processes FETCH packet
func (r *Redirector) handleFetchPacket(packet *dnspb.DNSPacket) ([]byte, error) {
	val, ok := r.conversations.Load(packet.ConversationId)
	if !ok {
		return nil, fmt.Errorf("conversation not found: %s", packet.ConversationId)
	}

	conv := val.(*Conversation)
	conv.mu.Lock()
	defer conv.mu.Unlock()

	if conv.ResponseData == nil {
		return nil, fmt.Errorf("no response data available")
	}

	conv.LastActivity = time.Now()

	// Check if response was chunked
	if len(conv.ResponseChunks) > 0 {
		// Empty data = metadata request
		// Non-empty data = FetchPayload with 1-based chunk_index
		if len(packet.Data) == 0 {
			// Return ResponseMetadata
			metadata := &dnspb.ResponseMetadata{
				TotalChunks: uint32(len(conv.ResponseChunks)),
				DataCrc32:   conv.ResponseCRC,
				ChunkSize:   uint32(len(conv.ResponseChunks[0])),
			}
			metadataBytes, err := proto.Marshal(metadata)
			if err != nil {
				return nil, fmt.Errorf("failed to marshal metadata: %w", err)
			}

			slog.Debug("returning response metadata", "conv_id", conv.ID, "total_chunks", len(conv.ResponseChunks),
				"total_size", len(conv.ResponseData), "crc32", conv.ResponseCRC)

			return metadataBytes, nil
		}

		// Parse FetchPayload - chunk_index is 1-based
		var fetchPayload dnspb.FetchPayload
		if err := proto.Unmarshal(packet.Data, &fetchPayload); err != nil {
			return nil, fmt.Errorf("failed to unmarshal fetch payload: %w", err)
		}

		// Convert 1-based to 0-based array index
		chunkIndex := int(fetchPayload.ChunkIndex) - 1

		if chunkIndex < 0 || chunkIndex >= len(conv.ResponseChunks) {
			return nil, fmt.Errorf("invalid chunk index: %d (expected 1-%d)", fetchPayload.ChunkIndex, len(conv.ResponseChunks))
		}

		slog.Debug("returning response chunk", "conv_id", conv.ID, "chunk", fetchPayload.ChunkIndex,
			"size", len(conv.ResponseChunks[chunkIndex]), "total_chunks", len(conv.ResponseChunks))

		return conv.ResponseChunks[chunkIndex], nil
	}

	// Single response (not chunked)
	// Don't delete immediately - rely on timeout-based cleanup

	slog.Debug("returning response", "conv_id", conv.ID, "size", len(conv.ResponseData))

	return conv.ResponseData, nil
}

// forwardToUpstream sends request to gRPC server and returns response
func (r *Redirector) forwardToUpstream(ctx context.Context, upstream *grpc.ClientConn, methodPath string, requestData []byte) ([]byte, error) {
	// Create gRPC stream with the raw codec
	md := metadata.New(map[string]string{})
	ctx = metadata.NewOutgoingContext(ctx, md)

	// Determine if this is a streaming method
	isClientStreaming := methodPath == "/c2.C2/ReportFile"
	isServerStreaming := methodPath == "/c2.C2/FetchAsset"

	stream, err := upstream.NewStream(ctx, &grpc.StreamDesc{
		StreamName:    methodPath,
		ServerStreams: isServerStreaming,
		ClientStreams: isClientStreaming,
	}, methodPath, grpc.CallContentSubtype("raw"))
	if err != nil {
		return nil, fmt.Errorf("failed to create stream: %w", err)
	}

	// Send request
	if isClientStreaming {
		// For client streaming (ReportFile), parse length-prefixed chunks and send individually
		offset := 0
		chunkCount := 0
		for offset < len(requestData) {
			if offset+4 > len(requestData) {
				break
			}

			// Read 4-byte length prefix
			msgLen := binary.BigEndian.Uint32(requestData[offset : offset+4])
			offset += 4

			if offset+int(msgLen) > len(requestData) {
				return nil, fmt.Errorf("invalid chunk length: %d bytes at offset %d", msgLen, offset)
			}

			// Send individual chunk (already encrypted)
			chunk := requestData[offset : offset+int(msgLen)]
			if err := stream.SendMsg(chunk); err != nil {
				return nil, fmt.Errorf("failed to send chunk %d: %w", chunkCount, err)
			}

			offset += int(msgLen)
			chunkCount++
		}

		slog.Debug("sent client streaming chunks", "method", methodPath, "chunks", chunkCount)
	} else {
		// For unary/server-streaming, send the request as-is
		if err := stream.SendMsg(requestData); err != nil {
			return nil, fmt.Errorf("failed to send request: %w", err)
		}
	}

	if err := stream.CloseSend(); err != nil {
		return nil, fmt.Errorf("failed to close send: %w", err)
	}

	// Receive response(s)
	var responseData []byte
	if isServerStreaming {
		// For server streaming (FetchAsset), receive multiple chunks with length prefixes
		responseCount := 0
		for {
			var msg []byte
			err := stream.RecvMsg(&msg)
			if err != nil {
				// Check for EOF (normal end of stream)
				if errors.Is(err, io.EOF) {
					break
				}
				return nil, fmt.Errorf("failed to receive message: %w", err)
			}

			if len(msg) > 0 {
				// Add 4-byte length prefix before each response chunk
				lengthPrefix := make([]byte, 4)
				binary.BigEndian.PutUint32(lengthPrefix, uint32(len(msg)))
				responseData = append(responseData, lengthPrefix...)
				responseData = append(responseData, msg...)
				responseCount++
			}
		}
		slog.Debug("received server streaming responses", "method", methodPath, "count", responseCount)
	} else {
		// For unary, receive single response
		if err := stream.RecvMsg(&responseData); err != nil {
			return nil, fmt.Errorf("failed to receive response: %w", err)
		}
	}

	return responseData, nil
}

// parseDomainNameAndType extracts domain name and query type
func (r *Redirector) parseDomainNameAndType(data []byte) (string, uint16, error) {
	var labels []string
	offset := 0

	for offset < len(data) {
		length := int(data[offset])
		if length == 0 {
			break
		}
		offset++

		if offset+length > len(data) {
			return "", 0, fmt.Errorf("invalid domain name")
		}

		label := string(data[offset : offset+length])
		labels = append(labels, label)
		offset += length
	}

	// Skip the null terminator (0x00)
	offset++

	if offset+2 > len(data) {
		return "", 0, fmt.Errorf("query too short for type field")
	}

	queryType := binary.BigEndian.Uint16(data[offset : offset+2])
	domain := strings.Join(labels, ".")

	return domain, queryType, nil
}

// sendDNSResponse sends a DNS response with appropriate record type (TXT/A/AAAA)
// For A/AAAA records with data larger than 4/16 bytes, multiple answer records are sent
func (r *Redirector) sendDNSResponse(conn *net.UDPConn, addr *net.UDPAddr, transactionID uint16, domain string, queryType uint16, data []byte) {
	// For A/AAAA records, base32-encode data first (client expects to decode it)
	if queryType == aRecordType || queryType == aaaaRecordType {
		encoded := base32.StdEncoding.WithPadding(base32.NoPadding).EncodeToString(data)
		data = []byte(encoded)
	}

	// Determine chunk size and number of answer records needed
	var recordSize int
	var answerCount uint16

	switch queryType {
	case txtRecordType:
		// TXT can handle all data in one record (with internal chunking)
		recordSize = 0 // Special case - handled separately
		answerCount = 1
	case aRecordType:
		recordSize = 4
		answerCount = uint16((len(data) + recordSize - 1) / recordSize)
		if answerCount == 0 {
			answerCount = 1
		}
	case aaaaRecordType:
		recordSize = 16
		answerCount = uint16((len(data) + recordSize - 1) / recordSize)
		if answerCount == 0 {
			answerCount = 1
		}
	default:
		// Unknown type - single empty record
		recordSize = 0
		answerCount = 1
	}

	response := make([]byte, 0, 512)

	// DNS Header
	response = append(response, byte(transactionID>>8), byte(transactionID))
	response = append(response, byte(dnsResponseFlags>>8), byte(dnsResponseFlags&0xFF))
	response = append(response, 0x00, 0x01)                                   // Questions: 1
	response = append(response, byte(answerCount>>8), byte(answerCount&0xFF)) // Answers: multiple for A/AAAA
	response = append(response, 0x00, 0x00)                                   // Authority RRs: 0
	response = append(response, 0x00, 0x00)                                   // Additional RRs: 0

	// Question section - echo back the original query type
	for _, label := range strings.Split(domain, ".") {
		if len(label) == 0 {
			continue
		}
		response = append(response, byte(len(label)))
		response = append(response, []byte(label)...)
	}
	response = append(response, 0x00)                                     // End of domain
	response = append(response, byte(queryType>>8), byte(queryType&0xFF)) // Type: original query type
	response = append(response, 0x00, byte(dnsClassIN))                   // Class: IN

	// Answer section - build multiple records for A/AAAA
	switch queryType {
	case txtRecordType:
		// TXT record: single record with length-prefixed strings (split into 255-byte chunks)
		response = append(response, byte(dnsPointer>>8), byte(dnsPointer&0xFF)) // Name pointer
		response = append(response, byte(queryType>>8), byte(queryType&0xFF))   // Type: TXT
		response = append(response, 0x00, byte(dnsClassIN))                     // Class: IN
		response = append(response, 0x00, 0x00, 0x00, byte(dnsTTLSeconds))      // TTL

		var rdata []byte
		if len(data) == 0 {
			rdata = []byte{0x00} // Empty TXT string
		} else {
			// Split into 255-byte chunks
			tempData := data
			for len(tempData) > 0 {
				chunkSize := len(tempData)
				if chunkSize > txtMaxChunkSize {
					chunkSize = txtMaxChunkSize
				}
				rdata = append(rdata, byte(chunkSize))
				rdata = append(rdata, tempData[:chunkSize]...)
				tempData = tempData[chunkSize:]
			}
		}

		// RDLENGTH and RDATA
		response = append(response, byte(len(rdata)>>8), byte(len(rdata)))
		response = append(response, rdata...)

	case aRecordType:
		// Multiple A records - 4 bytes each
		for i := uint16(0); i < answerCount; i++ {
			response = append(response, byte(dnsPointer>>8), byte(dnsPointer&0xFF)) // Name pointer
			response = append(response, 0x00, byte(aRecordType))                    // Type: A
			response = append(response, 0x00, byte(dnsClassIN))                     // Class: IN
			response = append(response, 0x00, 0x00, 0x00, byte(dnsTTLSeconds))      // TTL

			// RDLENGTH: always 4 for A records
			response = append(response, 0x00, 0x04)

			// RDATA: 4 bytes from data, padded with zeros if needed
			start := int(i) * recordSize
			end := start + recordSize
			rdata := make([]byte, 4)
			if start < len(data) {
				copyEnd := end
				if copyEnd > len(data) {
					copyEnd = len(data)
				}
				copy(rdata, data[start:copyEnd])
			}
			response = append(response, rdata...)
		}

	case aaaaRecordType:
		// Multiple AAAA records - 16 bytes each
		for i := uint16(0); i < answerCount; i++ {
			response = append(response, byte(dnsPointer>>8), byte(dnsPointer&0xFF)) // Name pointer
			response = append(response, 0x00, byte(aaaaRecordType))                 // Type: AAAA
			response = append(response, 0x00, byte(dnsClassIN))                     // Class: IN
			response = append(response, 0x00, 0x00, 0x00, byte(dnsTTLSeconds))      // TTL

			// RDLENGTH: always 16 for AAAA records
			response = append(response, 0x00, 0x10)

			// RDATA: 16 bytes from data, padded with zeros if needed
			start := int(i) * recordSize
			end := start + recordSize
			rdata := make([]byte, 16)
			if start < len(data) {
				copyEnd := end
				if copyEnd > len(data) {
					copyEnd = len(data)
				}
				copy(rdata, data[start:copyEnd])
			}
			response = append(response, rdata...)
		}

	default:
		// Unknown type - single empty record
		response = append(response, byte(dnsPointer>>8), byte(dnsPointer&0xFF)) // Name pointer
		response = append(response, byte(queryType>>8), byte(queryType&0xFF))   // Type: match query
		response = append(response, 0x00, byte(dnsClassIN))                     // Class: IN
		response = append(response, 0x00, 0x00, 0x00, byte(dnsTTLSeconds))      // TTL
		response = append(response, 0x00, 0x00)                                 // RDLENGTH: 0
	}

	conn.WriteToUDP(response, addr)
}

// sendErrorResponse sends a DNS error response
func (r *Redirector) sendErrorResponse(conn *net.UDPConn, addr *net.UDPAddr, transactionID uint16) {
	response := make([]byte, dnsHeaderSize)
	binary.BigEndian.PutUint16(response[0:2], transactionID)
	response[2] = byte(dnsErrorFlags >> 8)
	response[3] = byte(dnsErrorFlags & 0xFF)

	conn.WriteToUDP(response, addr)
}
