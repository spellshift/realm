package conversation

import (
	"context"
	"encoding/binary"
	"errors"
	"fmt"
	"hash/crc32"
	"io"
	"log/slog"
	"sort"
	"sync"
	"time"

	"github.com/hashicorp/golang-lru/v2/expirable"
	"google.golang.org/grpc"
	"google.golang.org/grpc/metadata"
	"google.golang.org/protobuf/proto"
	"realm.pub/tavern/internal/c2/convpb"
	"realm.pub/tavern/internal/redirectors"
)

const (
	MaxAckRangesInResponse = 20
	MaxNacksInResponse     = 50
	MaxDataSize            = 50 * 1024 * 1024 // 50MB max data size
)

// Manager wraps an LRU cache and implements the conversation state machine.
// maxChunkSize is transport-specific (400 for DNS/TXT, 1400 for ICMP).
type Manager struct {
	mu    sync.Mutex
	cache *expirable.LRU[string, *Conversation]
}

// NewManager creates a Manager with the given capacity and TTL per conversation.
func NewManager(maxSize int, ttl time.Duration) *Manager {
	cache := expirable.NewLRU[string, *Conversation](maxSize, nil, ttl)
	return &Manager{cache: cache}
}

// HandleInit processes an INIT packet and returns a serialized STATUS response.
func (m *Manager) HandleInit(packet *convpb.ConvPacket) ([]byte, error) {
	var initPayload convpb.InitPayload
	if err := proto.Unmarshal(packet.Data, &initPayload); err != nil {
		return nil, fmt.Errorf("failed to unmarshal init payload: %w", err)
	}

	if initPayload.FileSize > MaxDataSize {
		return nil, fmt.Errorf("data size exceeds maximum: %d > %d bytes", initPayload.FileSize, MaxDataSize)
	}

	if initPayload.FileSize == 0 && initPayload.TotalChunks > 0 {
		slog.Warn("INIT packet missing file_size field", "conv_id", packet.ConversationId, "total_chunks", initPayload.TotalChunks)
	}

	conv := &Conversation{
		ID:               packet.ConversationId,
		MethodPath:       initPayload.MethodCode,
		TotalChunks:      initPayload.TotalChunks,
		ExpectedCRC:      initPayload.DataCrc32,
		ExpectedDataSize: initPayload.FileSize,
		Chunks:           make(map[uint32][]byte),
		Completed:        false,
	}

	// Atomically check-then-store to handle duplicate INITs idempotently.
	m.mu.Lock()
	if existing, ok := m.cache.Get(packet.ConversationId); ok {
		m.mu.Unlock()

		existing.mu.Lock()
		defer existing.mu.Unlock()

		slog.Debug("duplicate INIT for existing conversation", "conv_id", packet.ConversationId)

		acks, nacks := computeAcksNacks(existing)
		statusPacket := &convpb.ConvPacket{
			Type:           convpb.PacketType_PACKET_TYPE_STATUS,
			ConversationId: packet.ConversationId,
			Acks:           acks,
			Nacks:          nacks,
		}
		statusData, err := proto.Marshal(statusPacket)
		if err != nil {
			return nil, fmt.Errorf("failed to marshal duplicate init status: %w", err)
		}
		return statusData, nil
	}

	evicted := m.cache.Add(packet.ConversationId, conv)
	m.mu.Unlock()

	if evicted {
		slog.Debug("LRU evicted oldest conversation to make room", "conv_id", conv.ID)
	}

	slog.Debug("C2 conversation started", "conv_id", conv.ID, "method", conv.MethodPath,
		"total_chunks", conv.TotalChunks, "data_size", initPayload.FileSize)

	statusPacket := &convpb.ConvPacket{
		Type:           convpb.PacketType_PACKET_TYPE_STATUS,
		ConversationId: packet.ConversationId,
		Acks:           []*convpb.AckRange{},
		Nacks:          []uint32{},
	}
	statusData, err := proto.Marshal(statusPacket)
	if err != nil {
		m.cache.Remove(packet.ConversationId)
		return nil, fmt.Errorf("failed to marshal init status: %w", err)
	}
	return statusData, nil
}

// HandleData processes a DATA packet and returns a serialized STATUS response.
// maxChunkSize controls how the upstream response is split for FETCH (transport-specific).
// clientIP is the originating client address forwarded to upstream via x-redirected-for;
// pass redirectors.ExternalIPNoop when the client IP cannot be determined.
func (m *Manager) HandleData(ctx context.Context, upstream *grpc.ClientConn, packet *convpb.ConvPacket, maxChunkSize int, clientIP string) ([]byte, error) {
	conv, ok := m.cache.Get(packet.ConversationId)
	if !ok {
		return nil, fmt.Errorf("conversation not found: %s", packet.ConversationId)
	}

	// Re-Add to refresh TTL
	m.cache.Add(packet.ConversationId, conv)

	conv.mu.Lock()
	defer conv.mu.Unlock()

	// Once forwarded to upstream, return full ack range immediately.
	if conv.Completed {
		statusPacket := &convpb.ConvPacket{
			Type:           convpb.PacketType_PACKET_TYPE_STATUS,
			ConversationId: packet.ConversationId,
			Acks:           []*convpb.AckRange{{StartSeq: 1, EndSeq: conv.TotalChunks}},
			Nacks:          []uint32{},
		}
		statusData, err := proto.Marshal(statusPacket)
		if err != nil {
			return nil, fmt.Errorf("failed to marshal status packet: %w", err)
		}
		return statusData, nil
	}

	if packet.Sequence < 1 || packet.Sequence > conv.TotalChunks {
		return nil, fmt.Errorf("sequence out of bounds: %d (expected 1-%d)", packet.Sequence, conv.TotalChunks)
	}

	conv.Chunks[packet.Sequence] = packet.Data

	slog.Debug("received chunk", "conv_id", conv.ID, "seq", packet.Sequence, "size", len(packet.Data), "total", len(conv.Chunks))

	if uint32(len(conv.Chunks)) == conv.TotalChunks {
		conv.Completed = true
		slog.Debug("C2 request complete, forwarding to upstream", "conv_id", conv.ID,
			"method", conv.MethodPath, "total_chunks", conv.TotalChunks, "data_size", conv.ExpectedDataSize)

		conv.mu.Unlock()
		if err := processCompleted(ctx, upstream, m.cache, conv, maxChunkSize, clientIP); err != nil {
			slog.Error("upstream request failed, could not process completed conversation",
				"conv_id", conv.ID, "method", conv.MethodPath, "error", err)
		}
		conv.mu.Lock()
	}

	acks, nacks := computeAcksNacks(conv)

	statusPacket := &convpb.ConvPacket{
		Type:           convpb.PacketType_PACKET_TYPE_STATUS,
		ConversationId: packet.ConversationId,
		Acks:           acks,
		Nacks:          nacks,
	}
	statusData, err := proto.Marshal(statusPacket)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal status packet: %w", err)
	}
	return statusData, nil
}

// HandleFetch processes a FETCH packet and returns the response data (or metadata).
func (m *Manager) HandleFetch(packet *convpb.ConvPacket) ([]byte, error) {
	conv, ok := m.cache.Get(packet.ConversationId)
	if !ok {
		return nil, fmt.Errorf("conversation not found: %s", packet.ConversationId)
	}

	// Re-Add to refresh TTL
	m.cache.Add(packet.ConversationId, conv)

	conv.mu.Lock()
	defer conv.mu.Unlock()

	if conv.ResponseData == nil {
		slog.Debug("response not ready yet - upstream call in progress", "conv_id", conv.ID)
		return []byte{}, nil
	}

	if len(conv.ResponseChunks) > 0 {
		if len(packet.Data) == 0 {
			metadata := &convpb.ResponseMetadata{
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

		var fetchPayload convpb.FetchPayload
		if err := proto.Unmarshal(packet.Data, &fetchPayload); err != nil {
			return nil, fmt.Errorf("failed to unmarshal fetch payload: %w", err)
		}

		chunkIndex := int(fetchPayload.ChunkIndex) - 1
		if chunkIndex < 0 || chunkIndex >= len(conv.ResponseChunks) {
			return nil, fmt.Errorf("invalid chunk index: %d (expected 1-%d)", fetchPayload.ChunkIndex, len(conv.ResponseChunks))
		}

		slog.Debug("returning response chunk", "conv_id", conv.ID, "chunk", fetchPayload.ChunkIndex,
			"size", len(conv.ResponseChunks[chunkIndex]), "total_chunks", len(conv.ResponseChunks))

		return conv.ResponseChunks[chunkIndex], nil
	}

	slog.Debug("returning response", "conv_id", conv.ID, "size", len(conv.ResponseData))
	return conv.ResponseData, nil
}

// HandleComplete processes a COMPLETE packet and returns a serialized STATUS response.
func (m *Manager) HandleComplete(packet *convpb.ConvPacket) ([]byte, error) {
	statusPacket := &convpb.ConvPacket{
		Type:           convpb.PacketType_PACKET_TYPE_STATUS,
		ConversationId: packet.ConversationId,
		Acks:           []*convpb.AckRange{},
		Nacks:          []uint32{},
	}
	statusData, err := proto.Marshal(statusPacket)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal complete status: %w", err)
	}
	return statusData, nil
}

// computeAcksNacks computes ACK ranges and NACK list for a conversation.
// Must be called with conv.mu locked.
func computeAcksNacks(conv *Conversation) ([]*convpb.AckRange, []uint32) {
	received := make([]uint32, 0, len(conv.Chunks))
	for seq := range conv.Chunks {
		received = append(received, seq)
	}
	sort.Slice(received, func(i, j int) bool { return received[i] < received[j] })

	acks := []*convpb.AckRange{}
	if len(received) > 0 {
		start := received[0]
		end := received[0]
		for i := 1; i < len(received); i++ {
			if received[i] == end+1 {
				end = received[i]
			} else {
				acks = append(acks, &convpb.AckRange{StartSeq: start, EndSeq: end})
				start = received[i]
				end = received[i]
			}
		}
		acks = append(acks, &convpb.AckRange{StartSeq: start, EndSeq: end})
	}

	if len(acks) > MaxAckRangesInResponse {
		acks = acks[:MaxAckRangesInResponse]
	}

	nacks := []uint32{}
	if len(received) > 0 {
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

// processCompleted reassembles data, verifies CRC, forwards to upstream, and stores the response.
func processCompleted(ctx context.Context, upstream *grpc.ClientConn, cache *expirable.LRU[string, *Conversation], conv *Conversation, maxChunkSize int, clientIP string) error {
	// Phase 1: reassemble and verify under the lock, then free chunk memory.
	conv.mu.Lock()
	var fullData []byte
	if conv.ExpectedDataSize > 0 {
		fullData = make([]byte, 0, conv.ExpectedDataSize)
	}
	for i := uint32(1); i <= conv.TotalChunks; i++ {
		chunk, ok := conv.Chunks[i]
		if !ok {
			conv.mu.Unlock()
			return fmt.Errorf("missing chunk %d", i)
		}
		fullData = append(fullData, chunk...)
	}
	for seq := range conv.Chunks {
		conv.Chunks[seq] = nil
	}

	actualCRC := crc32.ChecksumIEEE(fullData)
	if actualCRC != conv.ExpectedCRC {
		conv.mu.Unlock()
		cache.Remove(conv.ID)
		return fmt.Errorf("data CRC mismatch: expected %d, got %d", conv.ExpectedCRC, actualCRC)
	}

	if conv.ExpectedDataSize > 0 && uint32(len(fullData)) != conv.ExpectedDataSize {
		conv.mu.Unlock()
		cache.Remove(conv.ID)
		return fmt.Errorf("reassembled data size mismatch: expected %d bytes, got %d bytes", conv.ExpectedDataSize, len(fullData))
	}

	methodPath := conv.MethodPath
	convID := conv.ID
	conv.mu.Unlock()

	slog.Debug("reassembled data", "conv_id", convID, "size", len(fullData), "method", methodPath)

	// Phase 2: forward to upstream without holding the lock so HandleFetch can proceed.
	// HandleFetch returns an empty response while ResponseData == nil, and the agent retries.
	responseData, err := forwardToUpstream(ctx, upstream, methodPath, fullData, clientIP)

	// Phase 3: store the response (or a terminal empty slice on error) under the lock.
	conv.mu.Lock()
	defer conv.mu.Unlock()

	if err != nil {
		// Keep the conversation so FETCH receives a terminal empty response instead of
		// "conversation not found", which would cause transport-level timeouts.
		conv.ResponseData = []byte{}
		conv.ResponseChunks = nil
		return fmt.Errorf("failed to forward to upstream: %w", err)
	}

	if len(responseData) > maxChunkSize {
		conv.ResponseCRC = crc32.ChecksumIEEE(responseData)
		conv.ResponseData = responseData
		conv.ResponseChunks = nil
		for i := 0; i < len(responseData); i += maxChunkSize {
			end := i + maxChunkSize
			if end > len(responseData) {
				end = len(responseData)
			}
			conv.ResponseChunks = append(conv.ResponseChunks, responseData[i:end])
		}
		slog.Debug("response chunked", "conv_id", convID, "total_size", len(responseData),
			"chunks", len(conv.ResponseChunks), "crc32", conv.ResponseCRC)
	} else {
		conv.ResponseData = responseData
		slog.Debug("stored response", "conv_id", convID, "size", len(responseData))
	}

	return nil
}

// forwardToUpstream sends the request to the gRPC upstream and returns the response bytes.
func forwardToUpstream(ctx context.Context, upstream *grpc.ClientConn, methodPath string, requestData []byte, clientIP string) ([]byte, error) {
	md := metadata.New(map[string]string{})
	ctx = metadata.NewOutgoingContext(ctx, md)
	ctx = redirectors.SetRedirectedForHeader(ctx, clientIP)

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

	if isClientStreaming {
		offset := 0
		chunkCount := 0
		for offset < len(requestData) {
			if offset+4 > len(requestData) {
				break
			}
			msgLen := binary.BigEndian.Uint32(requestData[offset : offset+4])
			offset += 4
			if offset+int(msgLen) > len(requestData) {
				return nil, fmt.Errorf("invalid chunk length: %d bytes at offset %d", msgLen, offset)
			}
			chunk := requestData[offset : offset+int(msgLen)]
			if err := stream.SendMsg(chunk); err != nil {
				return nil, fmt.Errorf("failed to send chunk %d: %w", chunkCount, err)
			}
			offset += int(msgLen)
			chunkCount++
		}
		slog.Debug("sent client streaming chunks", "method", methodPath, "chunks", chunkCount)
	} else {
		if err := stream.SendMsg(requestData); err != nil {
			return nil, fmt.Errorf("failed to send request: %w", err)
		}
	}

	if err := stream.CloseSend(); err != nil {
		return nil, fmt.Errorf("failed to close send: %w", err)
	}

	var responseData []byte
	if isServerStreaming {
		responseCount := 0
		for {
			var msg []byte
			err := stream.RecvMsg(&msg)
			if err != nil {
				if errors.Is(err, io.EOF) {
					break
				}
				return nil, fmt.Errorf("failed to receive message: %w", err)
			}
			if len(msg) > 0 {
				lengthPrefix := make([]byte, 4)
				binary.BigEndian.PutUint32(lengthPrefix, uint32(len(msg)))
				responseData = append(responseData, lengthPrefix...)
				responseData = append(responseData, msg...)
				responseCount++
			}
		}
		slog.Debug("received server streaming responses", "method", methodPath, "count", responseCount)
	} else {
		if err := stream.RecvMsg(&responseData); err != nil {
			return nil, fmt.Errorf("failed to receive response: %w", err)
		}
	}

	return responseData, nil
}
