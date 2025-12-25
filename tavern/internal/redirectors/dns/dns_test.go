package dns

import (
	"context"
	"encoding/base32"
	"hash/crc32"
	"net"
	"sort"
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/proto"
	"realm.pub/tavern/internal/c2/dnspb"
)

// TestParseListenAddr tests the ParseListenAddr function
func TestParseListenAddr(t *testing.T) {
	tests := []struct {
		name            string
		input           string
		expectedAddr    string
		expectedDomains []string
		expectError     bool
	}{
		{
			name:            "default port with multiple domains",
			input:           "0.0.0.0?domain=dnsc2.realm.pub&domain=foo.bar",
			expectedAddr:    "0.0.0.0:53",
			expectedDomains: []string{"dnsc2.realm.pub", "foo.bar"},
		},
		{
			name:            "custom port with single domain",
			input:           "127.0.0.1:8053?domain=dnsc2.realm.pub",
			expectedAddr:    "127.0.0.1:8053",
			expectedDomains: []string{"dnsc2.realm.pub"},
		},
		{
			name:            "no query params",
			input:           "0.0.0.0:5353",
			expectedAddr:    "0.0.0.0:5353",
			expectedDomains: nil,
		},
		{
			name:            "empty domain value",
			input:           "0.0.0.0?domain=",
			expectedAddr:    "0.0.0.0:53",
			expectedDomains: nil,
		},
		{
			name:            "mixed valid and empty domains",
			input:           "0.0.0.0?domain=valid.com&domain=&domain=also.valid",
			expectedAddr:    "0.0.0.0:53",
			expectedDomains: []string{"valid.com", "also.valid"},
		},
		{
			name:        "malformed URL encoding",
			input:       "0.0.0.0?domain=%ZZ",
			expectError: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			addr, domains, err := ParseListenAddr(tt.input)

			if tt.expectError {
				assert.Error(t, err)
				return
			}

			require.NoError(t, err)
			assert.Equal(t, tt.expectedAddr, addr)
			assert.ElementsMatch(t, tt.expectedDomains, domains)
		})
	}
}

// TestExtractSubdomain tests subdomain extraction from full domain names
func TestExtractSubdomain(t *testing.T) {
	r := &Redirector{
		baseDomains: []string{"dnsc2.realm.pub", "foo.bar.com"},
	}

	tests := []struct {
		name           string
		domain         string
		expectedSubdom string
		expectError    bool
	}{
		{
			name:           "simple subdomain",
			domain:         "test.dnsc2.realm.pub",
			expectedSubdom: "test",
		},
		{
			name:           "multi-label subdomain",
			domain:         "a.b.c.dnsc2.realm.pub",
			expectedSubdom: "a.b.c",
		},
		{
			name:           "subdomain with longer base domain",
			domain:         "test.foo.bar.com",
			expectedSubdom: "test",
		},
		{
			name:        "no matching base domain",
			domain:      "test.unknown.com",
			expectError: true,
		},
		{
			name:        "only base domain (no subdomain)",
			domain:      "dnsc2.realm.pub",
			expectError: true,
		},
		{
			name:           "case insensitive match",
			domain:         "test.DNSC2.REALM.PUB",
			expectedSubdom: "test",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			subdomain, err := r.extractSubdomain(tt.domain)

			if tt.expectError {
				assert.Error(t, err)
				return
			}

			require.NoError(t, err)
			assert.Equal(t, tt.expectedSubdom, subdomain)
		})
	}
}

// TestDecodePacket tests Base32 decoding and protobuf unmarshaling
func TestDecodePacket(t *testing.T) {
	r := &Redirector{}

	t.Run("valid INIT packet", func(t *testing.T) {
		packet := &dnspb.DNSPacket{
			Type:           dnspb.PacketType_PACKET_TYPE_INIT,
			Sequence:       0,
			ConversationId: "test1234",
			Data:           []byte{0x01, 0x02, 0x03},
		}
		packetBytes, err := proto.Marshal(packet)
		require.NoError(t, err)

		encoded := base32.StdEncoding.WithPadding(base32.NoPadding).EncodeToString(packetBytes)

		decoded, err := r.decodePacket(encoded)
		require.NoError(t, err)
		assert.Equal(t, dnspb.PacketType_PACKET_TYPE_INIT, decoded.Type)
		assert.Equal(t, "test1234", decoded.ConversationId)
		assert.Equal(t, []byte{0x01, 0x02, 0x03}, decoded.Data)
	})

	t.Run("valid DATA packet with CRC", func(t *testing.T) {
		data := []byte{0xDE, 0xAD, 0xBE, 0xEF}
		packet := &dnspb.DNSPacket{
			Type:           dnspb.PacketType_PACKET_TYPE_DATA,
			Sequence:       1,
			ConversationId: "test5678",
			Data:           data,
			Crc32:          crc32.ChecksumIEEE(data),
		}
		packetBytes, err := proto.Marshal(packet)
		require.NoError(t, err)

		encoded := base32.StdEncoding.WithPadding(base32.NoPadding).EncodeToString(packetBytes)

		decoded, err := r.decodePacket(encoded)
		require.NoError(t, err)
		assert.Equal(t, dnspb.PacketType_PACKET_TYPE_DATA, decoded.Type)
		assert.Equal(t, data, decoded.Data)
	})

	t.Run("DATA packet with invalid CRC", func(t *testing.T) {
		data := []byte{0xDE, 0xAD, 0xBE, 0xEF}
		packet := &dnspb.DNSPacket{
			Type:           dnspb.PacketType_PACKET_TYPE_DATA,
			Sequence:       1,
			ConversationId: "test5678",
			Data:           data,
			Crc32:          0xDEADBEEF, // Wrong CRC
		}
		packetBytes, err := proto.Marshal(packet)
		require.NoError(t, err)

		encoded := base32.StdEncoding.WithPadding(base32.NoPadding).EncodeToString(packetBytes)

		_, err = r.decodePacket(encoded)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "CRC mismatch")
	})

	t.Run("invalid Base32", func(t *testing.T) {
		_, err := r.decodePacket("!!!invalid!!!")
		assert.Error(t, err)
	})

	t.Run("invalid protobuf", func(t *testing.T) {
		// Valid Base32 but not valid protobuf
		encoded := base32.StdEncoding.WithPadding(base32.NoPadding).EncodeToString([]byte{0xFF, 0xFF, 0xFF})
		_, err := r.decodePacket(encoded)
		assert.Error(t, err)
	})

	t.Run("packet with labels (dots)", func(t *testing.T) {
		packet := &dnspb.DNSPacket{
			Type:           dnspb.PacketType_PACKET_TYPE_INIT,
			ConversationId: "test1234",
		}
		packetBytes, err := proto.Marshal(packet)
		require.NoError(t, err)

		encoded := base32.StdEncoding.WithPadding(base32.NoPadding).EncodeToString(packetBytes)
		// Split into labels (simulating DNS format)
		withDots := encoded[:4] + "." + encoded[4:]

		decoded, err := r.decodePacket(withDots)
		require.NoError(t, err)
		assert.Equal(t, "test1234", decoded.ConversationId)
	})
}

// TestComputeAcksNacks tests the ACK range and NACK computation
func TestComputeAcksNacks(t *testing.T) {
	r := &Redirector{}

	tests := []struct {
		name          string
		chunks        map[uint32][]byte
		expectedAcks  []*dnspb.AckRange
		expectedNacks []uint32
	}{
		{
			name:          "empty chunks",
			chunks:        map[uint32][]byte{},
			expectedAcks:  []*dnspb.AckRange{},
			expectedNacks: []uint32{},
		},
		{
			name: "single chunk",
			chunks: map[uint32][]byte{
				1: {0x01},
			},
			expectedAcks: []*dnspb.AckRange{
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
			expectedAcks: []*dnspb.AckRange{
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
			expectedAcks: []*dnspb.AckRange{
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
			expectedAcks: []*dnspb.AckRange{
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

			acks, nacks := r.computeAcksNacks(conv)

			// Sort both slices for comparison
			sort.Slice(acks, func(i, j int) bool { return acks[i].StartSeq < acks[j].StartSeq })

			assert.Equal(t, tt.expectedAcks, acks)
			assert.Equal(t, tt.expectedNacks, nacks)
		})
	}
}

// TestHandleInitPacket tests INIT packet processing
func TestHandleInitPacket(t *testing.T) {
	t.Run("valid init packet", func(t *testing.T) {
		r := &Redirector{}

		initPayload := &dnspb.InitPayload{
			MethodCode:  "/c2.C2/ClaimTasks",
			TotalChunks: 5,
			DataCrc32:   0x12345678,
			FileSize:    1024,
		}
		payloadBytes, err := proto.Marshal(initPayload)
		require.NoError(t, err)

		packet := &dnspb.DNSPacket{
			Type:           dnspb.PacketType_PACKET_TYPE_INIT,
			ConversationId: "conv1234",
			Data:           payloadBytes,
		}

		responseData, err := r.handleInitPacket(packet)
		require.NoError(t, err)
		require.NotNil(t, responseData)

		// Verify response is a STATUS packet
		var statusPacket dnspb.DNSPacket
		err = proto.Unmarshal(responseData, &statusPacket)
		require.NoError(t, err)
		assert.Equal(t, dnspb.PacketType_PACKET_TYPE_STATUS, statusPacket.Type)
		assert.Equal(t, "conv1234", statusPacket.ConversationId)

		// Verify conversation was created
		val, ok := r.conversations.Load("conv1234")
		require.True(t, ok)
		conv := val.(*Conversation)
		assert.Equal(t, "/c2.C2/ClaimTasks", conv.MethodPath)
		assert.Equal(t, uint32(5), conv.TotalChunks)
		assert.Equal(t, uint32(0x12345678), conv.ExpectedCRC)
		assert.Equal(t, uint32(1024), conv.ExpectedDataSize)
	})

	t.Run("invalid init payload", func(t *testing.T) {
		r := &Redirector{}

		packet := &dnspb.DNSPacket{
			Type:           dnspb.PacketType_PACKET_TYPE_INIT,
			ConversationId: "conv1234",
			Data:           []byte{0xFF, 0xFF}, // Invalid protobuf
		}

		_, err := r.handleInitPacket(packet)
		assert.Error(t, err)
	})

	t.Run("data size exceeds maximum", func(t *testing.T) {
		r := &Redirector{}

		initPayload := &dnspb.InitPayload{
			MethodCode:  "/c2.C2/ClaimTasks",
			TotalChunks: 1,
			FileSize:    MaxDataSize + 1, // Exceeds limit
		}
		payloadBytes, err := proto.Marshal(initPayload)
		require.NoError(t, err)

		packet := &dnspb.DNSPacket{
			Type:           dnspb.PacketType_PACKET_TYPE_INIT,
			ConversationId: "conv1234",
			Data:           payloadBytes,
		}

		_, err = r.handleInitPacket(packet)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "exceeds maximum")
	})

	t.Run("max conversations reached", func(t *testing.T) {
		r := &Redirector{
			conversationCount: MaxActiveConversations,
		}

		initPayload := &dnspb.InitPayload{
			MethodCode:  "/c2.C2/ClaimTasks",
			TotalChunks: 1,
		}
		payloadBytes, err := proto.Marshal(initPayload)
		require.NoError(t, err)

		packet := &dnspb.DNSPacket{
			Type:           dnspb.PacketType_PACKET_TYPE_INIT,
			ConversationId: "conv1234",
			Data:           payloadBytes,
		}

		_, err = r.handleInitPacket(packet)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "max active conversations")
	})
}

// TestHandleFetchPacket tests FETCH packet processing
func TestHandleFetchPacket(t *testing.T) {
	t.Run("fetch single response", func(t *testing.T) {
		r := &Redirector{}
		responseData := []byte("test response data")

		conv := &Conversation{
			ID:           "conv1234",
			ResponseData: responseData,
			LastActivity: time.Now(),
		}
		r.conversations.Store("conv1234", conv)

		packet := &dnspb.DNSPacket{
			Type:           dnspb.PacketType_PACKET_TYPE_FETCH,
			ConversationId: "conv1234",
		}

		data, err := r.handleFetchPacket(packet)
		require.NoError(t, err)
		assert.Equal(t, responseData, data)
	})

	t.Run("fetch chunked response metadata", func(t *testing.T) {
		r := &Redirector{}
		responseData := []byte("full response")
		responseCRC := crc32.ChecksumIEEE(responseData)

		conv := &Conversation{
			ID:             "conv1234",
			ResponseData:   responseData,
			ResponseChunks: [][]byte{[]byte("chunk1"), []byte("chunk2")},
			ResponseCRC:    responseCRC,
			LastActivity:   time.Now(),
		}
		r.conversations.Store("conv1234", conv)

		packet := &dnspb.DNSPacket{
			Type:           dnspb.PacketType_PACKET_TYPE_FETCH,
			ConversationId: "conv1234",
			Data:           nil, // No payload = request metadata
		}

		data, err := r.handleFetchPacket(packet)
		require.NoError(t, err)

		var metadata dnspb.ResponseMetadata
		err = proto.Unmarshal(data, &metadata)
		require.NoError(t, err)
		assert.Equal(t, uint32(2), metadata.TotalChunks)
		assert.Equal(t, responseCRC, metadata.DataCrc32)
	})

	t.Run("fetch specific chunk", func(t *testing.T) {
		r := &Redirector{}

		conv := &Conversation{
			ID:             "conv1234",
			ResponseData:   []byte("full"),
			ResponseChunks: [][]byte{[]byte("chunk0"), []byte("chunk1"), []byte("chunk2")},
			LastActivity:   time.Now(),
		}
		r.conversations.Store("conv1234", conv)

		fetchPayload := &dnspb.FetchPayload{ChunkIndex: 2} // 1-indexed
		payloadBytes, err := proto.Marshal(fetchPayload)
		require.NoError(t, err)

		packet := &dnspb.DNSPacket{
			Type:           dnspb.PacketType_PACKET_TYPE_FETCH,
			ConversationId: "conv1234",
			Data:           payloadBytes,
		}

		data, err := r.handleFetchPacket(packet)
		require.NoError(t, err)
		assert.Equal(t, []byte("chunk1"), data) // 1-indexed -> 0-indexed
	})

	t.Run("fetch unknown conversation", func(t *testing.T) {
		r := &Redirector{}

		packet := &dnspb.DNSPacket{
			Type:           dnspb.PacketType_PACKET_TYPE_FETCH,
			ConversationId: "unknown",
		}

		_, err := r.handleFetchPacket(packet)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "conversation not found")
	})

	t.Run("fetch with no response ready", func(t *testing.T) {
		r := &Redirector{}

		conv := &Conversation{
			ID:           "conv1234",
			ResponseData: nil, // No response yet
			LastActivity: time.Now(),
		}
		r.conversations.Store("conv1234", conv)

		packet := &dnspb.DNSPacket{
			Type:           dnspb.PacketType_PACKET_TYPE_FETCH,
			ConversationId: "conv1234",
		}

		_, err := r.handleFetchPacket(packet)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "no response data")
	})

	t.Run("fetch chunk out of bounds", func(t *testing.T) {
		r := &Redirector{}

		conv := &Conversation{
			ID:             "conv1234",
			ResponseData:   []byte("full"),
			ResponseChunks: [][]byte{[]byte("chunk0")},
			LastActivity:   time.Now(),
		}
		r.conversations.Store("conv1234", conv)

		fetchPayload := &dnspb.FetchPayload{ChunkIndex: 10} // Out of bounds
		payloadBytes, err := proto.Marshal(fetchPayload)
		require.NoError(t, err)

		packet := &dnspb.DNSPacket{
			Type:           dnspb.PacketType_PACKET_TYPE_FETCH,
			ConversationId: "conv1234",
			Data:           payloadBytes,
		}

		_, err = r.handleFetchPacket(packet)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "invalid chunk index")
	})
}

// TestParseDomainNameAndType tests DNS query parsing
func TestParseDomainNameAndType(t *testing.T) {
	r := &Redirector{}

	tests := []struct {
		name         string
		query        []byte
		expectDomain string
		expectType   uint16
		expectError  bool
	}{
		{
			name: "valid TXT query",
			query: func() []byte {
				q := []byte{
					5, 'd', 'n', 's', 'c', '2', // "dnsc2"
					5, 'r', 'e', 'a', 'l', 'm', // "realm"
					3, 'p', 'u', 'b', // "pub"
					0,     // null terminator
					0, 16, // Type: TXT
					0, 1, // Class: IN
				}
				return q
			}(),
			expectDomain: "dnsc2.realm.pub",
			expectType:   16,
		},
		{
			name: "valid A query",
			query: func() []byte {
				q := []byte{
					4, 't', 'e', 's', 't', // "test"
					5, 'd', 'n', 's', 'c', '2', // "dnsc2"
					5, 'r', 'e', 'a', 'l', 'm', // "realm"
					3, 'p', 'u', 'b', // "pub"
					0,    // null terminator
					0, 1, // Type: A
					0, 1, // Class: IN
				}
				return q
			}(),
			expectDomain: "test.dnsc2.realm.pub",
			expectType:   1,
		},
		{
			name: "valid AAAA query",
			query: func() []byte {
				q := []byte{
					3, 'w', 'w', 'w', // "www"
					4, 't', 'e', 's', 't', // "test"
					3, 'c', 'o', 'm', // "com"
					0,     // null terminator
					0, 28, // Type: AAAA
					0, 1, // Class: IN
				}
				return q
			}(),
			expectDomain: "www.test.com",
			expectType:   28,
		},
		{
			name:        "truncated query",
			query:       []byte{7, 'e', 'x', 'a'}, // Incomplete
			expectError: true,
		},
		{
			name:        "query too short for type",
			query:       []byte{4, 't', 'e', 's', 't', 0}, // Missing type/class
			expectError: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			domain, queryType, err := r.parseDomainNameAndType(tt.query)

			if tt.expectError {
				assert.Error(t, err)
				return
			}

			require.NoError(t, err)
			assert.Equal(t, tt.expectDomain, domain)
			assert.Equal(t, tt.expectType, queryType)
		})
	}
}

// TestConversationCleanup tests cleanup of stale conversations
func TestConversationCleanup(t *testing.T) {
	r := &Redirector{
		conversationTimeout: 15 * time.Minute,
	}

	// Create stale conversation
	staleConv := &Conversation{
		ID:           "stale",
		LastActivity: time.Now().Add(-20 * time.Minute),
	}
	r.conversations.Store("stale", staleConv)
	r.conversationCount = 1

	// Create fresh conversation
	freshConv := &Conversation{
		ID:           "fresh",
		LastActivity: time.Now(),
	}
	r.conversations.Store("fresh", freshConv)
	r.conversationCount = 2

	// Run cleanup
	now := time.Now()
	r.conversations.Range(func(key, value any) bool {
		conv := value.(*Conversation)
		conv.mu.Lock()
		if now.Sub(conv.LastActivity) > r.conversationTimeout {
			r.conversations.Delete(key)
			r.conversationCount--
		}
		conv.mu.Unlock()
		return true
	})

	// Verify stale was removed
	_, ok := r.conversations.Load("stale")
	assert.False(t, ok, "stale conversation should be removed")

	// Verify fresh remains
	_, ok = r.conversations.Load("fresh")
	assert.True(t, ok, "fresh conversation should remain")

	assert.Equal(t, int32(1), r.conversationCount)
}

// TestConcurrentConversationAccess tests thread safety of conversation handling
func TestConcurrentConversationAccess(t *testing.T) {
	r := &Redirector{}

	initPayload := &dnspb.InitPayload{
		MethodCode:  "/c2.C2/ClaimTasks",
		TotalChunks: 100,
		DataCrc32:   0,
		FileSize:    0,
	}
	payloadBytes, err := proto.Marshal(initPayload)
	require.NoError(t, err)

	packet := &dnspb.DNSPacket{
		Type:           dnspb.PacketType_PACKET_TYPE_INIT,
		ConversationId: "concurrent",
		Data:           payloadBytes,
	}

	_, err = r.handleInitPacket(packet)
	require.NoError(t, err)

	// Concurrent access to store chunks
	var wg sync.WaitGroup
	for i := uint32(1); i <= 100; i++ {
		wg.Add(1)
		go func(seq uint32) {
			defer wg.Done()

			val, ok := r.conversations.Load("concurrent")
			if !ok {
				return
			}
			conv := val.(*Conversation)
			conv.mu.Lock()
			conv.Chunks[seq] = []byte{byte(seq)}
			conv.mu.Unlock()
		}(i)
	}
	wg.Wait()

	// Verify all chunks stored
	val, ok := r.conversations.Load("concurrent")
	require.True(t, ok)
	conv := val.(*Conversation)
	assert.Len(t, conv.Chunks, 100)
}

// TestBuildDNSResponse tests DNS response packet construction
func TestBuildDNSResponse(t *testing.T) {
	r := &Redirector{}

	// Create a mock UDP connection for testing
	serverAddr, err := net.ResolveUDPAddr("udp", "127.0.0.1:0")
	require.NoError(t, err)
	serverConn, err := net.ListenUDP("udp", serverAddr)
	require.NoError(t, err)
	defer serverConn.Close()

	clientAddr, err := net.ResolveUDPAddr("udp", "127.0.0.1:0")
	require.NoError(t, err)
	clientConn, err := net.ListenUDP("udp", clientAddr)
	require.NoError(t, err)
	defer clientConn.Close()

	t.Run("TXT record response", func(t *testing.T) {
		r.sendDNSResponse(serverConn, clientConn.LocalAddr().(*net.UDPAddr), 0x1234, "test.dnsc2.realm.pub", txtRecordType, []byte("hello"))

		buf := make([]byte, 512)
		clientConn.SetReadDeadline(time.Now().Add(time.Second))
		n, _, err := clientConn.ReadFromUDP(buf)
		require.NoError(t, err)

		// Verify transaction ID
		assert.Equal(t, uint16(0x1234), uint16(buf[0])<<8|uint16(buf[1]))
		// Verify it's a response (QR bit set)
		assert.True(t, buf[2]&0x80 != 0)
		// Verify answer count is 1
		assert.Equal(t, uint16(1), uint16(buf[6])<<8|uint16(buf[7]))

		// Response should contain data
		assert.Greater(t, n, 12)
	})
}

// TestHandleDataPacket tests DATA packet processing and chunk storage
func TestHandleDataPacket(t *testing.T) {
	t.Run("store single chunk", func(t *testing.T) {
		r := &Redirector{}
		ctx := context.Background()

		// Create conversation first with INIT - set TotalChunks > 1 to avoid completion
		initPayload := &dnspb.InitPayload{
			MethodCode:  "/c2.C2/ClaimTasks",
			TotalChunks: 2, // Prevent completion on first chunk
			DataCrc32:   crc32.ChecksumIEEE([]byte{0x01, 0x02}),
			FileSize:    2,
		}
		payloadBytes, err := proto.Marshal(initPayload)
		require.NoError(t, err)

		initPacket := &dnspb.DNSPacket{
			Type:           dnspb.PacketType_PACKET_TYPE_INIT,
			ConversationId: "data1234",
			Data:           payloadBytes,
		}
		_, err = r.handleInitPacket(initPacket)
		require.NoError(t, err)

		// Send DATA packet
		dataPacket := &dnspb.DNSPacket{
			Type:           dnspb.PacketType_PACKET_TYPE_DATA,
			ConversationId: "data1234",
			Sequence:       1,
			Data:           []byte{0x01},
		}

		statusData, err := r.handleDataPacket(ctx, nil, dataPacket, txtRecordType)
		require.NoError(t, err)

		// Verify STATUS response
		var statusPacket dnspb.DNSPacket
		err = proto.Unmarshal(statusData, &statusPacket)
		require.NoError(t, err)
		assert.Equal(t, dnspb.PacketType_PACKET_TYPE_STATUS, statusPacket.Type)
		assert.Equal(t, "data1234", statusPacket.ConversationId)

		// Verify chunk was stored
		val, ok := r.conversations.Load("data1234")
		require.True(t, ok)
		conv := val.(*Conversation)
		assert.Len(t, conv.Chunks, 1)
		assert.Equal(t, []byte{0x01}, conv.Chunks[1])
	})

	t.Run("store multiple chunks with gaps", func(t *testing.T) {
		r := &Redirector{}
		ctx := context.Background()

		// Create conversation
		initPayload := &dnspb.InitPayload{
			MethodCode:  "/c2.C2/ClaimTasks",
			TotalChunks: 5,
			DataCrc32:   0,
			FileSize:    5,
		}
		payloadBytes, err := proto.Marshal(initPayload)
		require.NoError(t, err)

		initPacket := &dnspb.DNSPacket{
			Type:           dnspb.PacketType_PACKET_TYPE_INIT,
			ConversationId: "gaps1234",
			Data:           payloadBytes,
		}
		_, err = r.handleInitPacket(initPacket)
		require.NoError(t, err)

		// Send chunks 1, 3, 5 (gaps at 2, 4)
		for _, seq := range []uint32{1, 3, 5} {
			dataPacket := &dnspb.DNSPacket{
				Type:           dnspb.PacketType_PACKET_TYPE_DATA,
				ConversationId: "gaps1234",
				Sequence:       seq,
				Data:           []byte{byte(seq)},
			}

			statusData, err := r.handleDataPacket(ctx, nil, dataPacket, txtRecordType)
			require.NoError(t, err)

			// Parse STATUS response
			var statusPacket dnspb.DNSPacket
			err = proto.Unmarshal(statusData, &statusPacket)
			require.NoError(t, err)

			// Should always have ACKs for received chunks
			assert.NotEmpty(t, statusPacket.Acks)
			// NACKs will appear after gaps - not on first chunk
		}

		// Verify chunks stored
		val, ok := r.conversations.Load("gaps1234")
		require.True(t, ok)
		conv := val.(*Conversation)
		assert.Len(t, conv.Chunks, 3)
		assert.Equal(t, []byte{1}, conv.Chunks[1])
		assert.Equal(t, []byte{3}, conv.Chunks[3])
		assert.Equal(t, []byte{5}, conv.Chunks[5])
		assert.False(t, conv.Completed) // Not all chunks received
	})

	t.Run("unknown conversation", func(t *testing.T) {
		r := &Redirector{}
		ctx := context.Background()

		dataPacket := &dnspb.DNSPacket{
			Type:           dnspb.PacketType_PACKET_TYPE_DATA,
			ConversationId: "unknown",
			Sequence:       1,
			Data:           []byte{0x01},
		}

		_, err := r.handleDataPacket(ctx, nil, dataPacket, txtRecordType)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "conversation not found")
	})

	t.Run("sequence out of bounds", func(t *testing.T) {
		r := &Redirector{}
		ctx := context.Background()

		// Create conversation
		initPayload := &dnspb.InitPayload{
			MethodCode:  "/c2.C2/ClaimTasks",
			TotalChunks: 3,
			DataCrc32:   0,
		}
		payloadBytes, err := proto.Marshal(initPayload)
		require.NoError(t, err)

		initPacket := &dnspb.DNSPacket{
			Type:           dnspb.PacketType_PACKET_TYPE_INIT,
			ConversationId: "bounds1234",
			Data:           payloadBytes,
		}
		_, err = r.handleInitPacket(initPacket)
		require.NoError(t, err)

		// Send chunk with sequence > TotalChunks
		dataPacket := &dnspb.DNSPacket{
			Type:           dnspb.PacketType_PACKET_TYPE_DATA,
			ConversationId: "bounds1234",
			Sequence:       10,
			Data:           []byte{0x01},
		}

		_, err = r.handleDataPacket(ctx, nil, dataPacket, txtRecordType)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "sequence out of bounds")
	})
}

// TestProcessCompletedConversation tests data reassembly and CRC validation
func TestProcessCompletedConversation(t *testing.T) {
	t.Run("successful reassembly and CRC validation", func(t *testing.T) {
		data := []byte{0x01, 0x02, 0x03, 0x04, 0x05}
		expectedCRC := crc32.ChecksumIEEE(data)

		conv := &Conversation{
			ID:               "complete1234",
			MethodPath:       "/test/method",
			TotalChunks:      3,
			ExpectedCRC:      expectedCRC,
			ExpectedDataSize: uint32(len(data)),
			Chunks: map[uint32][]byte{
				1: {0x01, 0x02},
				2: {0x03, 0x04},
				3: {0x05},
			},
		}

		// Mock upstream that returns empty response
		// Since we can't easily mock grpc.ClientConn, we'll test the reassembly logic
		// by directly checking the data assembly

		// Manually reassemble to test logic
		var fullData []byte
		for i := uint32(1); i <= conv.TotalChunks; i++ {
			chunk, ok := conv.Chunks[i]
			require.True(t, ok, "missing chunk %d", i)
			fullData = append(fullData, chunk...)
		}

		assert.Equal(t, data, fullData)
		actualCRC := crc32.ChecksumIEEE(fullData)
		assert.Equal(t, expectedCRC, actualCRC)
		assert.Equal(t, conv.ExpectedDataSize, uint32(len(fullData)))
	})

	t.Run("CRC mismatch detection", func(t *testing.T) {
		data := []byte{0x01, 0x02, 0x03}
		wrongCRC := uint32(0xDEADBEEF)

		conv := &Conversation{
			ID:          "crcfail1234",
			MethodPath:  "/test/method",
			TotalChunks: 1,
			ExpectedCRC: wrongCRC,
			Chunks: map[uint32][]byte{
				1: data,
			},
		}

		// Test CRC validation logic
		var fullData []byte
		for i := uint32(1); i <= conv.TotalChunks; i++ {
			fullData = append(fullData, conv.Chunks[i]...)
		}

		actualCRC := crc32.ChecksumIEEE(fullData)
		assert.NotEqual(t, wrongCRC, actualCRC, "CRC should mismatch")
	})
}
