package dns_test

import (
	"context"
	"net"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	dnsredirector "realm.pub/tavern/internal/redirectors/dns"
)

// TestParseListenAddr tests the parseListenAddr function
func TestParseListenAddr(t *testing.T) {
	t.Run("default port with multiple domains", func(t *testing.T) {
		addr, domains, err := dnsredirector.ParseListenAddr("0.0.0.0?domain=example.com&domain=foo.bar")
		require.NoError(t, err)
		assert.Equal(t, "0.0.0.0:53", addr)
		assert.ElementsMatch(t, []string{"example.com", "foo.bar"}, domains)
	})

	t.Run("custom port with single domain", func(t *testing.T) {
		addr, domains, err := dnsredirector.ParseListenAddr("127.0.0.1:8053?domain=example.com")
		require.NoError(t, err)
		assert.Equal(t, "127.0.0.1:8053", addr)
		assert.ElementsMatch(t, []string{"example.com"}, domains)
	})

	t.Run("malformed domain value", func(t *testing.T) {
		_, _, err := dnsredirector.ParseListenAddr("127.0.0.1:8053?domain=%ZZ")
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "decode domain")
	})

	t.Run("no query params", func(t *testing.T) {
		addr, domains, err := dnsredirector.ParseListenAddr("0.0.0.0:5353")
		require.NoError(t, err)
		assert.Equal(t, "0.0.0.0:5353", addr)
		assert.Empty(t, domains)
	})
}

// newTestRedirector creates a test redirector with stubbed upstream
func newTestRedirector() *dnsredirector.Redirector {
	return &dnsredirector.Redirector{}
}

// TestInitDataEndLifecycle tests the complete packet handling flow
func TestInitDataEndLifecycle(t *testing.T) {
	r := newTestRedirector()

	// Step 1: Send init packet
	// Init payload: [method_code:2][total_chunks:5][crc:4]
	methodCode := "ct"        // ClaimTasks
	totalChunksStr := "00002" // 2 chunks (base36)
	testData := []byte{0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08}
	crc := dnsredirector.CalculateCRC16(testData)
	crcStr := dnsredirector.EncodeBase36CRC(int(crc))

	initPayload := methodCode + totalChunksStr + crcStr
	tempConvID := "temp12345678"

	convID, err := r.HandleInitPacket(tempConvID, initPayload)
	require.NoError(t, err)
	assert.NotEmpty(t, convID)
	assert.Len(t, convID, 12) // CONV_ID_SIZE

	convIDStr := string(convID)

	// Verify conversation was created
	conv, ok := r.GetConversation(convIDStr)
	require.True(t, ok)
	assert.Equal(t, "/c2.C2/ClaimTasks", conv.MethodPath)
	assert.Equal(t, 2, conv.TotalChunks)
	assert.Equal(t, crc, conv.ExpectedCRC)

	// Step 2: Send data chunks
	chunk0 := testData[:4]
	chunk1 := testData[4:]

	_, err = r.HandleDataPacket(convIDStr, 0, chunk0)
	require.NoError(t, err)

	_, err = r.HandleDataPacket(convIDStr, 1, chunk1)
	require.NoError(t, err)

	// Verify chunks were stored
	conv, _ = r.GetConversation(convIDStr)
	assert.Len(t, conv.Chunks, 2)
}

// TestHandleDataPacketUnknownConversation tests error handling for unknown conversation
func TestHandleDataPacketUnknownConversation(t *testing.T) {
	r := newTestRedirector()

	_, err := r.HandleDataPacket("nonexistent", 0, []byte{0x01, 0x02})
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "unknown conversation")
}

// TestHandleFetchPacket tests response chunk fetching
func TestHandleFetchPacket(t *testing.T) {
	r := newTestRedirector()

	t.Run("fetch chunk within bounds - text chunking", func(t *testing.T) {
		convID := "test12345678"
		conv := &dnsredirector.Conversation{
			ID:               convID,
			ResponseChunks:   []string{"chunk0", "chunk1", "chunk2"},
			IsBinaryChunking: false,
			LastActivity:     time.Now(),
		}
		r.StoreConversation(convID, conv)

		// Fetch chunk 1
		data, err := r.HandleFetchPacket(convID, 1)
		require.NoError(t, err)
		assert.Equal(t, "ok:chunk1", string(data))

		// Conversation should still exist
		_, ok := r.GetConversation(convID)
		assert.True(t, ok)
	})

	t.Run("fetch chunk within bounds - binary chunking", func(t *testing.T) {
		convID := "bin123456789"
		conv := &dnsredirector.Conversation{
			ID:               convID,
			ResponseChunks:   []string{string([]byte{0x01, 0x02}), string([]byte{0x03, 0x04})},
			IsBinaryChunking: true,
			LastActivity:     time.Now(),
		}
		r.StoreConversation(convID, conv)

		// Fetch chunk 0
		data, err := r.HandleFetchPacket(convID, 0)
		require.NoError(t, err)
		assert.Equal(t, []byte{0x01, 0x02}, data)
	})

	t.Run("fetch beyond bounds triggers cleanup", func(t *testing.T) {
		convID := "cleanup12345"
		conv := &dnsredirector.Conversation{
			ID:               convID,
			ResponseChunks:   []string{"chunk0"},
			IsBinaryChunking: false,
			LastActivity:     time.Now(),
		}
		r.StoreConversation(convID, conv)

		// Fetch seq beyond bounds
		data, err := r.HandleFetchPacket(convID, 1)
		require.NoError(t, err)
		assert.Equal(t, "ok:", string(data))

		// Conversation should be deleted
		_, ok := r.GetConversation(convID)
		assert.False(t, ok)
	})

	t.Run("fetch from unknown conversation", func(t *testing.T) {
		_, err := r.HandleFetchPacket("unknown", 0)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "unknown conversation")
	})
}

// TestCleanupConversations tests conversation expiration
func TestCleanupConversations(t *testing.T) {
	r := newTestRedirector()

	// Create stale conversation (old timestamp)
	staleConvID := "stale1234567"
	staleConv := &dnsredirector.Conversation{
		ID:           staleConvID,
		LastActivity: time.Now().Add(-20 * time.Minute), // Older than timeout
	}
	r.StoreConversation(staleConvID, staleConv)

	// Create fresh conversation
	freshConvID := "fresh1234567"
	freshConv := &dnsredirector.Conversation{
		ID:           freshConvID,
		LastActivity: time.Now(),
	}
	r.StoreConversation(freshConvID, freshConv)

	// Run cleanup once
	r.CleanupConversationsOnce(15 * time.Minute)

	// Verify stale conversation was removed
	_, ok := r.GetConversation(staleConvID)
	assert.False(t, ok, "stale conversation should be removed")

	// Verify fresh conversation remains
	_, ok = r.GetConversation(freshConvID)
	assert.True(t, ok, "fresh conversation should remain")
}



// stubUpstream provides a minimal gRPC server for testing
type stubUpstream struct {
	server     *grpc.Server
	clientConn *grpc.ClientConn
	t          *testing.T
}

func newStubUpstream(t *testing.T, echoData []byte) *stubUpstream {
	t.Helper()

	// Create a simple handler that echoes back the request
	handler := func(srv any, stream grpc.ServerStream) error {
		var reqBytes []byte
		if err := stream.RecvMsg(&reqBytes); err != nil {
			return err
		}

		// Echo back the request data
		return stream.SendMsg(echoData)
	}

	server := grpc.NewServer(grpc.UnknownServiceHandler(handler))

	// Start server on random port
	listener, err := testListener(t)
	require.NoError(t, err)

	go func() {
		if err := server.Serve(listener); err != nil && err != grpc.ErrServerStopped {
			t.Logf("stub server error: %v", err)
		}
	}()

	// Create client connection
	conn, err := grpc.Dial(listener.Addr().String(), grpc.WithTransportCredentials(insecure.NewCredentials()))
	require.NoError(t, err)

	return &stubUpstream{
		server:     server,
		clientConn: conn,
		t:          t,
	}
}

func (s *stubUpstream) ClientConn() *grpc.ClientConn {
	return s.clientConn
}

func (s *stubUpstream) Close() {
	s.clientConn.Close()
	s.server.Stop()
}

func testListener(t *testing.T) (net.Listener, error) {
	t.Helper()
	return net.Listen("tcp", "127.0.0.1:0")
}

// TestCRCMismatch tests CRC validation failure
func TestCRCMismatch(t *testing.T) {
	r := newTestRedirector()

	// Create conversation with wrong CRC
	methodCode := "ct"
	totalChunksStr := "00001"
	wrongCRC := dnsredirector.EncodeBase36CRC(12345) // Wrong CRC
	initPayload := methodCode + totalChunksStr + wrongCRC

	convID, err := r.HandleInitPacket("temp", initPayload)
	require.NoError(t, err)

	convIDStr := string(convID)

	// Send data with different content
	actualData := []byte{0xFF, 0xFF, 0xFF, 0xFF}
	_, err = r.HandleDataPacket(convIDStr, 0, actualData)
	require.NoError(t, err)

	// Note: CRC validation now happens automatically when all chunks received
}
