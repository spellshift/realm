package conversation

import (
	"context"
	"fmt"
	"hash/crc32"
	"sort"
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/proto"
	"realm.pub/tavern/internal/c2/convpb"
)

func newTestManager() *Manager {
	return NewManager(200_000, 5*time.Minute)
}

// TestComputeAcksNacks tests ACK range and NACK computation.
func TestComputeAcksNacks(t *testing.T) {
	tests := []struct {
		name          string
		chunks        map[uint32][]byte
		expectedAcks  []*convpb.AckRange
		expectedNacks []uint32
	}{
		{
			name:          "empty chunks",
			chunks:        map[uint32][]byte{},
			expectedAcks:  []*convpb.AckRange{},
			expectedNacks: []uint32{},
		},
		{
			name: "single chunk",
			chunks: map[uint32][]byte{
				1: {0x01},
			},
			expectedAcks: []*convpb.AckRange{
				{StartSeq: 1, EndSeq: 1},
			},
			expectedNacks: []uint32{},
		},
		{
			name: "consecutive chunks",
			chunks: map[uint32][]byte{
				1: {0x01},
				2: {0x02},
				3: {0x03},
			},
			expectedAcks: []*convpb.AckRange{
				{StartSeq: 1, EndSeq: 3},
			},
			expectedNacks: []uint32{},
		},
		{
			name: "gap in middle",
			chunks: map[uint32][]byte{
				1: {0x01},
				2: {0x02},
				5: {0x05},
				6: {0x06},
			},
			expectedAcks: []*convpb.AckRange{
				{StartSeq: 1, EndSeq: 2},
				{StartSeq: 5, EndSeq: 6},
			},
			expectedNacks: []uint32{3, 4},
		},
		{
			name: "multiple gaps",
			chunks: map[uint32][]byte{
				1:  {0x01},
				3:  {0x03},
				5:  {0x05},
				10: {0x0A},
			},
			expectedAcks: []*convpb.AckRange{
				{StartSeq: 1, EndSeq: 1},
				{StartSeq: 3, EndSeq: 3},
				{StartSeq: 5, EndSeq: 5},
				{StartSeq: 10, EndSeq: 10},
			},
			expectedNacks: []uint32{2, 4, 6, 7, 8, 9},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			conv := &Conversation{
				Chunks: tt.chunks,
			}
			conv.mu.Lock()
			acks, nacks := computeAcksNacks(conv)
			conv.mu.Unlock()

			sort.Slice(acks, func(i, j int) bool { return acks[i].StartSeq < acks[j].StartSeq })
			assert.Equal(t, tt.expectedAcks, acks)
			assert.Equal(t, tt.expectedNacks, nacks)
		})
	}
}

// TestManagerHandleInit tests INIT packet processing.
func TestManagerHandleInit(t *testing.T) {
	t.Run("valid init packet", func(t *testing.T) {
		m := newTestManager()

		initPayload := &convpb.InitPayload{
			MethodCode:  "/c2.C2/ClaimTasks",
			TotalChunks: 5,
			DataCrc32:   0x12345678,
			FileSize:    1024,
		}
		payloadBytes, err := proto.Marshal(initPayload)
		require.NoError(t, err)

		packet := &convpb.ConvPacket{
			Type:           convpb.PacketType_PACKET_TYPE_INIT,
			ConversationId: "conv1234",
			Data:           payloadBytes,
		}

		responseData, err := m.HandleInit(packet)
		require.NoError(t, err)
		require.NotNil(t, responseData)

		var statusPacket convpb.ConvPacket
		err = proto.Unmarshal(responseData, &statusPacket)
		require.NoError(t, err)
		assert.Equal(t, convpb.PacketType_PACKET_TYPE_STATUS, statusPacket.Type)
		assert.Equal(t, "conv1234", statusPacket.ConversationId)

		conv, ok := m.cache.Get("conv1234")
		require.True(t, ok)
		assert.Equal(t, "/c2.C2/ClaimTasks", conv.MethodPath)
		assert.Equal(t, uint32(5), conv.TotalChunks)
		assert.Equal(t, uint32(0x12345678), conv.ExpectedCRC)
		assert.Equal(t, uint32(1024), conv.ExpectedDataSize)
	})

	t.Run("invalid init payload", func(t *testing.T) {
		m := newTestManager()

		packet := &convpb.ConvPacket{
			Type:           convpb.PacketType_PACKET_TYPE_INIT,
			ConversationId: "conv1234",
			Data:           []byte{0xFF, 0xFF},
		}

		_, err := m.HandleInit(packet)
		assert.Error(t, err)
	})

	t.Run("data size exceeds maximum", func(t *testing.T) {
		m := newTestManager()

		initPayload := &convpb.InitPayload{
			MethodCode:  "/c2.C2/ClaimTasks",
			TotalChunks: 1,
			FileSize:    MaxDataSize + 1,
		}
		payloadBytes, err := proto.Marshal(initPayload)
		require.NoError(t, err)

		packet := &convpb.ConvPacket{
			Type:           convpb.PacketType_PACKET_TYPE_INIT,
			ConversationId: "conv1234",
			Data:           payloadBytes,
		}

		_, err = m.HandleInit(packet)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "exceeds maximum")
	})

	t.Run("max conversations triggers LRU eviction", func(t *testing.T) {
		m := NewManager(2, 5*time.Minute)

		initPayload := &convpb.InitPayload{
			MethodCode:  "/c2.C2/ClaimTasks",
			TotalChunks: 1,
		}
		payloadBytes, err := proto.Marshal(initPayload)
		require.NoError(t, err)

		for i := 0; i < 2; i++ {
			packet := &convpb.ConvPacket{
				Type:           convpb.PacketType_PACKET_TYPE_INIT,
				ConversationId: fmt.Sprintf("conv%d", i),
				Data:           payloadBytes,
			}
			_, err = m.HandleInit(packet)
			require.NoError(t, err)
		}

		assert.Equal(t, 2, m.cache.Len())

		// Third conversation should evict the oldest (conv0).
		packet := &convpb.ConvPacket{
			Type:           convpb.PacketType_PACKET_TYPE_INIT,
			ConversationId: "conv2",
			Data:           payloadBytes,
		}
		_, err = m.HandleInit(packet)
		require.NoError(t, err)

		assert.Equal(t, 2, m.cache.Len())
		_, ok := m.cache.Get("conv0")
		assert.False(t, ok, "oldest conversation should be evicted")
		_, ok = m.cache.Get("conv2")
		assert.True(t, ok)
	})

	t.Run("duplicate INIT returns status without leaking state", func(t *testing.T) {
		m := newTestManager()

		initPayload := &convpb.InitPayload{
			MethodCode:  "/c2.C2/ClaimTasks",
			TotalChunks: 3,
			DataCrc32:   0xDEADBEEF,
			FileSize:    512,
		}
		payloadBytes, err := proto.Marshal(initPayload)
		require.NoError(t, err)

		packet := &convpb.ConvPacket{
			Type:           convpb.PacketType_PACKET_TYPE_INIT,
			ConversationId: "dupinit1234",
			Data:           payloadBytes,
		}

		resp1, err := m.HandleInit(packet)
		require.NoError(t, err)
		require.NotNil(t, resp1)
		assert.Equal(t, 1, m.cache.Len())

		var status1 convpb.ConvPacket
		err = proto.Unmarshal(resp1, &status1)
		require.NoError(t, err)
		assert.Equal(t, convpb.PacketType_PACKET_TYPE_STATUS, status1.Type)

		resp2, err := m.HandleInit(packet)
		require.NoError(t, err)
		require.NotNil(t, resp2)

		assert.Equal(t, 1, m.cache.Len(), "duplicate INIT should not create new conversation")

		var status2 convpb.ConvPacket
		err = proto.Unmarshal(resp2, &status2)
		require.NoError(t, err)
		assert.Equal(t, convpb.PacketType_PACKET_TYPE_STATUS, status2.Type)
		assert.Equal(t, "dupinit1234", status2.ConversationId)

		conv, ok := m.cache.Get("dupinit1234")
		require.True(t, ok)
		assert.Equal(t, "/c2.C2/ClaimTasks", conv.MethodPath)
		assert.Equal(t, uint32(3), conv.TotalChunks)
	})

	t.Run("concurrent duplicate INITs", func(t *testing.T) {
		m := newTestManager()

		initPayload := &convpb.InitPayload{
			MethodCode:  "/c2.C2/ClaimTasks",
			TotalChunks: 5,
			DataCrc32:   0x12345678,
			FileSize:    1024,
		}
		payloadBytes, err := proto.Marshal(initPayload)
		require.NoError(t, err)

		packet := &convpb.ConvPacket{
			Type:           convpb.PacketType_PACKET_TYPE_INIT,
			ConversationId: "concurrent-init",
			Data:           payloadBytes,
		}

		var wg sync.WaitGroup
		for i := 0; i < 10; i++ {
			wg.Add(1)
			go func() {
				defer wg.Done()
				_, err := m.HandleInit(packet)
				assert.NoError(t, err)
			}()
		}
		wg.Wait()

		assert.Equal(t, 1, m.cache.Len(), "concurrent INITs should not create duplicates")
		_, ok := m.cache.Get("concurrent-init")
		assert.True(t, ok)
	})
}

// TestManagerHandleData tests DATA packet processing and chunk storage.
func TestManagerHandleData(t *testing.T) {
	t.Run("store single chunk", func(t *testing.T) {
		m := newTestManager()
		ctx := context.Background()

		initPayload := &convpb.InitPayload{
			MethodCode:  "/c2.C2/ClaimTasks",
			TotalChunks: 2, // Prevent completion on first chunk.
			DataCrc32:   crc32.ChecksumIEEE([]byte{0x01, 0x02}),
			FileSize:    2,
		}
		payloadBytes, err := proto.Marshal(initPayload)
		require.NoError(t, err)

		initPacket := &convpb.ConvPacket{
			Type:           convpb.PacketType_PACKET_TYPE_INIT,
			ConversationId: "data1234",
			Data:           payloadBytes,
		}
		_, err = m.HandleInit(initPacket)
		require.NoError(t, err)

		dataPacket := &convpb.ConvPacket{
			Type:           convpb.PacketType_PACKET_TYPE_DATA,
			ConversationId: "data1234",
			Sequence:       1,
			Data:           []byte{0x01},
		}

		statusData, err := m.HandleData(ctx, nil, dataPacket, 400, "")
		require.NoError(t, err)

		var statusPacket convpb.ConvPacket
		err = proto.Unmarshal(statusData, &statusPacket)
		require.NoError(t, err)
		assert.Equal(t, convpb.PacketType_PACKET_TYPE_STATUS, statusPacket.Type)
		assert.Equal(t, "data1234", statusPacket.ConversationId)

		conv, ok := m.cache.Get("data1234")
		require.True(t, ok)
		assert.Len(t, conv.Chunks, 1)
		assert.Equal(t, []byte{0x01}, conv.Chunks[1])
	})

	t.Run("store multiple chunks with gaps", func(t *testing.T) {
		m := newTestManager()
		ctx := context.Background()

		initPayload := &convpb.InitPayload{
			MethodCode:  "/c2.C2/ClaimTasks",
			TotalChunks: 5,
		}
		payloadBytes, err := proto.Marshal(initPayload)
		require.NoError(t, err)

		initPacket := &convpb.ConvPacket{
			Type:           convpb.PacketType_PACKET_TYPE_INIT,
			ConversationId: "gaps1234",
			Data:           payloadBytes,
		}
		_, err = m.HandleInit(initPacket)
		require.NoError(t, err)

		// Send chunks 1, 3, 5 (gaps at 2, 4).
		for _, seq := range []uint32{1, 3, 5} {
			dataPacket := &convpb.ConvPacket{
				Type:           convpb.PacketType_PACKET_TYPE_DATA,
				ConversationId: "gaps1234",
				Sequence:       seq,
				Data:           []byte{byte(seq)},
			}
			statusData, err := m.HandleData(ctx, nil, dataPacket, 400, "")
			require.NoError(t, err)

			var statusPacket convpb.ConvPacket
			err = proto.Unmarshal(statusData, &statusPacket)
			require.NoError(t, err)
			assert.NotEmpty(t, statusPacket.Acks)
		}

		conv, ok := m.cache.Get("gaps1234")
		require.True(t, ok)
		assert.Len(t, conv.Chunks, 3)
		assert.Equal(t, []byte{1}, conv.Chunks[1])
		assert.Equal(t, []byte{3}, conv.Chunks[3])
		assert.Equal(t, []byte{5}, conv.Chunks[5])
		assert.False(t, conv.Completed)
	})

	t.Run("unknown conversation", func(t *testing.T) {
		m := newTestManager()
		ctx := context.Background()

		dataPacket := &convpb.ConvPacket{
			Type:           convpb.PacketType_PACKET_TYPE_DATA,
			ConversationId: "unknown",
			Sequence:       1,
			Data:           []byte{0x01},
		}

		_, err := m.HandleData(ctx, nil, dataPacket, 400, "")
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "conversation not found")
	})

	t.Run("sequence out of bounds", func(t *testing.T) {
		m := newTestManager()
		ctx := context.Background()

		initPayload := &convpb.InitPayload{
			MethodCode:  "/c2.C2/ClaimTasks",
			TotalChunks: 3,
		}
		payloadBytes, err := proto.Marshal(initPayload)
		require.NoError(t, err)

		initPacket := &convpb.ConvPacket{
			Type:           convpb.PacketType_PACKET_TYPE_INIT,
			ConversationId: "bounds1234",
			Data:           payloadBytes,
		}
		_, err = m.HandleInit(initPacket)
		require.NoError(t, err)

		dataPacket := &convpb.ConvPacket{
			Type:           convpb.PacketType_PACKET_TYPE_DATA,
			ConversationId: "bounds1234",
			Sequence:       10,
			Data:           []byte{0x01},
		}

		_, err = m.HandleData(ctx, nil, dataPacket, 400, "")
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "sequence out of bounds")
	})

	t.Run("short-circuit for completed conversation", func(t *testing.T) {
		m := newTestManager()
		ctx := context.Background()

		conv := &Conversation{
			ID:          "completed1",
			TotalChunks: 3,
			Completed:   true,
			Chunks: map[uint32][]byte{
				1: {0x01},
				2: {0x02},
				3: {0x03},
			},
		}
		m.cache.Add("completed1", conv)

		dataPacket := &convpb.ConvPacket{
			Type:           convpb.PacketType_PACKET_TYPE_DATA,
			ConversationId: "completed1",
			Sequence:       1,
			Data:           []byte{0xFF}, // Different data — should not overwrite.
		}

		statusData, err := m.HandleData(ctx, nil, dataPacket, 400, "")
		require.NoError(t, err)

		var statusPacket convpb.ConvPacket
		err = proto.Unmarshal(statusData, &statusPacket)
		require.NoError(t, err)
		assert.Equal(t, convpb.PacketType_PACKET_TYPE_STATUS, statusPacket.Type)
		require.Len(t, statusPacket.Acks, 1)
		assert.Equal(t, uint32(1), statusPacket.Acks[0].StartSeq)
		assert.Equal(t, uint32(3), statusPacket.Acks[0].EndSeq)
		assert.Empty(t, statusPacket.Nacks)

		// Original chunk data must not be overwritten.
		assert.Equal(t, []byte{0x01}, conv.Chunks[1])
	})

	t.Run("data for missing conversation returns not found", func(t *testing.T) {
		m := newTestManager()
		ctx := context.Background()

		dataPacket := &convpb.ConvPacket{
			Type:           convpb.PacketType_PACKET_TYPE_DATA,
			ConversationId: "missing123",
			Sequence:       1,
			Data:           []byte{0x01},
		}

		_, err := m.HandleData(ctx, nil, dataPacket, 400, "")
		require.Error(t, err)
		assert.Contains(t, err.Error(), "conversation not found")
		assert.Contains(t, err.Error(), "missing123")
	})
}

// TestManagerHandleFetch tests FETCH packet processing.
func TestManagerHandleFetch(t *testing.T) {
	t.Run("fetch single response", func(t *testing.T) {
		m := newTestManager()
		responseData := []byte("test response data")

		conv := &Conversation{
			ID:           "conv1234",
			ResponseData: responseData,
		}
		m.cache.Add("conv1234", conv)

		packet := &convpb.ConvPacket{
			Type:           convpb.PacketType_PACKET_TYPE_FETCH,
			ConversationId: "conv1234",
		}

		data, err := m.HandleFetch(packet)
		require.NoError(t, err)
		assert.Equal(t, responseData, data)
	})

	t.Run("fetch chunked response metadata", func(t *testing.T) {
		m := newTestManager()
		responseData := []byte("full response")
		responseCRC := crc32.ChecksumIEEE(responseData)

		conv := &Conversation{
			ID:             "conv1234",
			ResponseData:   responseData,
			ResponseChunks: [][]byte{[]byte("chunk1"), []byte("chunk2")},
			ResponseCRC:    responseCRC,
		}
		m.cache.Add("conv1234", conv)

		packet := &convpb.ConvPacket{
			Type:           convpb.PacketType_PACKET_TYPE_FETCH,
			ConversationId: "conv1234",
			Data:           nil, // No payload = request metadata.
		}

		data, err := m.HandleFetch(packet)
		require.NoError(t, err)

		var metadata convpb.ResponseMetadata
		err = proto.Unmarshal(data, &metadata)
		require.NoError(t, err)
		assert.Equal(t, uint32(2), metadata.TotalChunks)
		assert.Equal(t, responseCRC, metadata.DataCrc32)
	})

	t.Run("fetch specific chunk", func(t *testing.T) {
		m := newTestManager()

		conv := &Conversation{
			ID:             "conv1234",
			ResponseData:   []byte("full"),
			ResponseChunks: [][]byte{[]byte("chunk0"), []byte("chunk1"), []byte("chunk2")},
		}
		m.cache.Add("conv1234", conv)

		fetchPayload := &convpb.FetchPayload{ChunkIndex: 2} // 1-indexed
		payloadBytes, err := proto.Marshal(fetchPayload)
		require.NoError(t, err)

		packet := &convpb.ConvPacket{
			Type:           convpb.PacketType_PACKET_TYPE_FETCH,
			ConversationId: "conv1234",
			Data:           payloadBytes,
		}

		data, err := m.HandleFetch(packet)
		require.NoError(t, err)
		assert.Equal(t, []byte("chunk1"), data) // 1-indexed → 0-indexed
	})

	t.Run("fetch unknown conversation", func(t *testing.T) {
		m := newTestManager()

		packet := &convpb.ConvPacket{
			Type:           convpb.PacketType_PACKET_TYPE_FETCH,
			ConversationId: "unknown",
		}

		_, err := m.HandleFetch(packet)
		require.Error(t, err)
		assert.Contains(t, err.Error(), "conversation not found")
		assert.Contains(t, err.Error(), "unknown")
	})

	t.Run("fetch with no response ready returns empty", func(t *testing.T) {
		m := newTestManager()

		conv := &Conversation{
			ID:           "conv1234",
			ResponseData: nil, // Upstream call still in progress.
		}
		m.cache.Add("conv1234", conv)

		packet := &convpb.ConvPacket{
			Type:           convpb.PacketType_PACKET_TYPE_FETCH,
			ConversationId: "conv1234",
		}

		data, err := m.HandleFetch(packet)
		require.NoError(t, err)
		assert.Equal(t, []byte{}, data)
	})

	t.Run("fetch chunk out of bounds", func(t *testing.T) {
		m := newTestManager()

		conv := &Conversation{
			ID:             "conv1234",
			ResponseData:   []byte("full"),
			ResponseChunks: [][]byte{[]byte("chunk0")},
		}
		m.cache.Add("conv1234", conv)

		fetchPayload := &convpb.FetchPayload{ChunkIndex: 10} // Out of bounds.
		payloadBytes, err := proto.Marshal(fetchPayload)
		require.NoError(t, err)

		packet := &convpb.ConvPacket{
			Type:           convpb.PacketType_PACKET_TYPE_FETCH,
			ConversationId: "conv1234",
			Data:           payloadBytes,
		}

		_, err = m.HandleFetch(packet)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "invalid chunk index")
	})
}

// TestManagerHandleComplete tests COMPLETE packet processing.
func TestManagerHandleComplete(t *testing.T) {
	t.Run("complete returns status", func(t *testing.T) {
		m := newTestManager()

		packet := &convpb.ConvPacket{
			Type:           convpb.PacketType_PACKET_TYPE_COMPLETE,
			ConversationId: "complete1234",
		}

		responseData, err := m.HandleComplete(packet)
		require.NoError(t, err)
		require.NotNil(t, responseData)

		var statusPacket convpb.ConvPacket
		err = proto.Unmarshal(responseData, &statusPacket)
		require.NoError(t, err)
		assert.Equal(t, convpb.PacketType_PACKET_TYPE_STATUS, statusPacket.Type)
		assert.Equal(t, "complete1234", statusPacket.ConversationId)
	})

	t.Run("duplicate COMPLETE is idempotent", func(t *testing.T) {
		m := newTestManager()

		packet := &convpb.ConvPacket{
			Type:           convpb.PacketType_PACKET_TYPE_COMPLETE,
			ConversationId: "dupcomp1234",
		}

		resp1, err := m.HandleComplete(packet)
		require.NoError(t, err)
		require.NotNil(t, resp1)

		resp2, err := m.HandleComplete(packet)
		require.NoError(t, err, "duplicate COMPLETE should not error")
		require.NotNil(t, resp2)

		var status convpb.ConvPacket
		err = proto.Unmarshal(resp2, &status)
		require.NoError(t, err)
		assert.Equal(t, convpb.PacketType_PACKET_TYPE_STATUS, status.Type)
	})

	t.Run("COMPLETE for nonexistent conversation returns status", func(t *testing.T) {
		m := newTestManager()

		packet := &convpb.ConvPacket{
			Type:           convpb.PacketType_PACKET_TYPE_COMPLETE,
			ConversationId: "nonexistent",
		}

		responseData, err := m.HandleComplete(packet)
		require.NoError(t, err)
		require.NotNil(t, responseData)

		var statusPacket convpb.ConvPacket
		err = proto.Unmarshal(responseData, &statusPacket)
		require.NoError(t, err)
		assert.Equal(t, convpb.PacketType_PACKET_TYPE_STATUS, statusPacket.Type)
	})

	t.Run("concurrent COMPLETEs all succeed", func(t *testing.T) {
		m := newTestManager()

		packet := &convpb.ConvPacket{
			Type:           convpb.PacketType_PACKET_TYPE_COMPLETE,
			ConversationId: "concurrent-complete",
		}

		var wg sync.WaitGroup
		for i := 0; i < 10; i++ {
			wg.Add(1)
			go func() {
				defer wg.Done()
				_, err := m.HandleComplete(packet)
				assert.NoError(t, err)
			}()
		}
		wg.Wait()
	})
}

// TestManagerConcurrentAccess verifies thread safety of chunk storage.
func TestManagerConcurrentAccess(t *testing.T) {
	m := newTestManager()

	initPayload := &convpb.InitPayload{
		MethodCode:  "/c2.C2/ClaimTasks",
		TotalChunks: 100,
	}
	payloadBytes, err := proto.Marshal(initPayload)
	require.NoError(t, err)

	initPacket := &convpb.ConvPacket{
		Type:           convpb.PacketType_PACKET_TYPE_INIT,
		ConversationId: "concurrent",
		Data:           payloadBytes,
	}
	_, err = m.HandleInit(initPacket)
	require.NoError(t, err)

	var wg sync.WaitGroup
	for i := uint32(1); i <= 100; i++ {
		wg.Add(1)
		go func(seq uint32) {
			defer wg.Done()
			conv, ok := m.cache.Get("concurrent")
			if !ok {
				return
			}
			conv.mu.Lock()
			conv.Chunks[seq] = []byte{byte(seq)}
			conv.mu.Unlock()
		}(i)
	}
	wg.Wait()

	conv, ok := m.cache.Get("concurrent")
	require.True(t, ok)
	assert.Len(t, conv.Chunks, 100)
}

// TestManagerLRUEviction verifies the LRU cache evicts oldest conversations when full.
func TestManagerLRUEviction(t *testing.T) {
	t.Run("evicts oldest when at capacity", func(t *testing.T) {
		m := NewManager(3, 5*time.Minute)

		for i := 0; i < 3; i++ {
			m.cache.Add(fmt.Sprintf("conv%d", i), &Conversation{
				ID: fmt.Sprintf("conv%d", i),
			})
		}
		assert.Equal(t, 3, m.cache.Len())

		m.cache.Add("conv3", &Conversation{ID: "conv3"})
		assert.Equal(t, 3, m.cache.Len())

		_, ok := m.cache.Get("conv0")
		assert.False(t, ok, "oldest conversation should be evicted")
		_, ok = m.cache.Get("conv3")
		assert.True(t, ok, "newest conversation should exist")
	})

	t.Run("Get refreshes recency", func(t *testing.T) {
		m := NewManager(3, 5*time.Minute)

		for i := 0; i < 3; i++ {
			m.cache.Add(fmt.Sprintf("conv%d", i), &Conversation{
				ID: fmt.Sprintf("conv%d", i),
			})
		}

		// Access conv0 to refresh its recency.
		m.cache.Get("conv0")

		// Adding conv3 should evict conv1 (now the oldest) instead of conv0.
		m.cache.Add("conv3", &Conversation{ID: "conv3"})

		_, ok := m.cache.Get("conv0")
		assert.True(t, ok, "conv0 should survive due to recent Get")
		_, ok = m.cache.Get("conv1")
		assert.False(t, ok, "conv1 should be evicted as the oldest")
	})
}
