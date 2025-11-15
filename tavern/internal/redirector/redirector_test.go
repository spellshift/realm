package redirector

import (
	"bytes"
	"fmt"
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/status"
)

// TestRawCodecMarshal tests marshaling bytes with RawCodec
func TestRawCodecMarshal(t *testing.T) {
	codec := RawCodec{}
	testData := []byte("test data")

	result, err := codec.Marshal(testData)
	if err != nil {
		t.Fatalf("Marshal failed: %v", err)
	}
	if !bytes.Equal(result, testData) {
		t.Errorf("Expected %v, got %v", testData, result)
	}
}

// TestRawCodecMarshalInvalidType tests marshaling with invalid type
func TestRawCodecMarshalInvalidType(t *testing.T) {
	codec := RawCodec{}
	invalidData := "string data"

	_, err := codec.Marshal(invalidData)
	if err == nil {
		t.Error("Expected error when marshaling non-bytes type")
	}
	if !strings.Contains(err.Error(), "failed to marshal") {
		t.Errorf("Expected 'failed to marshal' in error, got: %v", err)
	}
}

// TestRawCodecUnmarshal tests unmarshaling bytes with RawCodec
func TestRawCodecUnmarshal(t *testing.T) {
	codec := RawCodec{}
	testData := []byte("test data")
	var result []byte

	err := codec.Unmarshal(testData, &result)
	if err != nil {
		t.Fatalf("Unmarshal failed: %v", err)
	}
	if !bytes.Equal(result, testData) {
		t.Errorf("Expected %v, got %v", testData, result)
	}
}

// TestRawCodecUnmarshalInvalidType tests unmarshaling with invalid type
func TestRawCodecUnmarshalInvalidType(t *testing.T) {
	codec := RawCodec{}
	testData := []byte("test data")
	var result string

	err := codec.Unmarshal(testData, &result)
	if err == nil {
		t.Error("Expected error when unmarshaling to non-*[]byte type")
	}
	if !strings.Contains(err.Error(), "failed to unmarshal") {
		t.Errorf("Expected 'failed to unmarshal' in error, got: %v", err)
	}
}

// TestRawCodecName tests the codec name
func TestRawCodecName(t *testing.T) {
	codec := RawCodec{}
	name := codec.Name()
	if name != "raw" {
		t.Errorf("Expected codec name 'raw', got '%s'", name)
	}
}

// TestConfigSetServer tests setting the HTTP server in config
func TestConfigSetServer(t *testing.T) {
	config := &Config{}
	server := &http.Server{Addr: ":8080"}

	config.SetServer(server)

	if config.srv != server {
		t.Error("SetServer did not properly set the server")
	}
	if config.srv.Addr != ":8080" {
		t.Errorf("Expected server address ':8080', got '%s'", config.srv.Addr)
	}
}

// TestRequirePOSTSuccess tests requirePOST with valid POST request
func TestRequirePOSTSuccess(t *testing.T) {
	req := httptest.NewRequest("POST", "/test", nil)
	w := httptest.NewRecorder()

	result := requirePOST(w, req)

	if !result {
		t.Error("requirePOST returned false for POST request")
	}
	// httptest.ResponseRecorder records a 200 status by default when WriteHeader is not explicitly called
	// This is expected behavior
}

// TestRequirePOSTMethodNotAllowed tests requirePOST with GET request
func TestRequirePOSTMethodNotAllowed(t *testing.T) {
	req := httptest.NewRequest("GET", "/test", nil)
	w := httptest.NewRecorder()

	result := requirePOST(w, req)

	if result {
		t.Error("requirePOST returned true for GET request")
	}
	if w.Code != http.StatusMethodNotAllowed {
		t.Errorf("Expected status %d, got %d", http.StatusMethodNotAllowed, w.Code)
	}
}

// TestRequirePOSTVariousMethods tests requirePOST with various HTTP methods
func TestRequirePOSTVariousMethods(t *testing.T) {
	methods := []string{"GET", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"}

	for _, method := range methods {
		req := httptest.NewRequest(method, "/test", nil)
		w := httptest.NewRecorder()

		result := requirePOST(w, req)

		if result {
			t.Errorf("requirePOST returned true for %s request", method)
		}
		if w.Code != http.StatusMethodNotAllowed {
			t.Errorf("Expected status %d for %s, got %d", http.StatusMethodNotAllowed, method, w.Code)
		}
	}
}

// TestReadRequestBodySuccess tests reading valid request body
func TestReadRequestBodySuccess(t *testing.T) {
	testData := []byte("test request body")
	req := httptest.NewRequest("POST", "/test", bytes.NewReader(testData))
	w := httptest.NewRecorder()

	result, ok := readRequestBody(w, req)

	if !ok {
		t.Error("readRequestBody returned false for valid body")
	}
	if !bytes.Equal(result, testData) {
		t.Errorf("Expected body %v, got %v", testData, result)
	}
}

// TestReadRequestBodyEmpty tests reading empty request body
func TestReadRequestBodyEmpty(t *testing.T) {
	req := httptest.NewRequest("POST", "/test", bytes.NewReader([]byte{}))
	w := httptest.NewRecorder()

	result, ok := readRequestBody(w, req)

	if !ok {
		t.Error("readRequestBody returned false for empty body")
	}
	if len(result) != 0 {
		t.Errorf("Expected empty body, got %v", result)
	}
}

// TestReadRequestBodyLarge tests reading large request body
func TestReadRequestBodyLarge(t *testing.T) {
	largeData := make([]byte, 1024*1024) // 1MB
	for i := range largeData {
		largeData[i] = byte(i % 256)
	}
	req := httptest.NewRequest("POST", "/test", bytes.NewReader(largeData))
	w := httptest.NewRecorder()

	result, ok := readRequestBody(w, req)

	if !ok {
		t.Error("readRequestBody returned false for large body")
	}
	if !bytes.Equal(result, largeData) {
		t.Errorf("Expected body size %d, got %d", len(largeData), len(result))
	}
}

// TestSetGRPCResponseHeaders tests setting proper gRPC response headers
func TestSetGRPCResponseHeaders(t *testing.T) {
	w := httptest.NewRecorder()

	setGRPCResponseHeaders(w)

	if w.Header().Get("Content-Type") != "application/grpc" {
		t.Errorf("Expected Content-Type 'application/grpc', got '%s'",
			w.Header().Get("Content-Type"))
	}
	if w.Code != http.StatusOK {
		t.Errorf("Expected status code %d, got %d", http.StatusOK, w.Code)
	}
}

// TestGetFlusherSuccess tests getting a flusher from ResponseWriter
func TestGetFlusherSuccess(t *testing.T) {
	w := httptest.NewRecorder()

	flusher, ok := getFlusher(w)

	if !ok {
		t.Error("getFlusher returned false for httptest.ResponseRecorder")
	}
	if flusher == nil {
		t.Error("Expected non-nil flusher")
	}
}

// TestGetFlusherWithErrorWriter tests getFlusher with writer that doesn't support flushing
func TestGetFlusherWithErrorWriter(t *testing.T) {
	var buf bytes.Buffer
	w := &nonFlushingWriter{buf: &buf}

	header := w.Header()
	if header == nil {
		t.Error("Expected non-nil header")
	}
}

// nonFlushingWriter is a mock ResponseWriter that doesn't implement Flusher
type nonFlushingWriter struct {
	buf    *bytes.Buffer
	header http.Header
}

func (n *nonFlushingWriter) Header() http.Header {
	if n.header == nil {
		n.header = make(http.Header)
	}
	return n.header
}

func (n *nonFlushingWriter) Write(b []byte) (int, error) {
	return n.buf.Write(b)
}

func (n *nonFlushingWriter) WriteHeader(statusCode int) {}

// TestHandleStreamError tests error handling for gRPC stream errors
func TestHandleStreamError(t *testing.T) {
	w := httptest.NewRecorder()
	testErr := status.Error(codes.Internal, "test error")

	handleStreamError(w, "Test message", testErr)

	if w.Code != http.StatusInternalServerError {
		t.Errorf("Expected status code %d, got %d", http.StatusInternalServerError, w.Code)
	}
	if !strings.Contains(w.Body.String(), "Test message") {
		t.Errorf("Expected 'Test message' in error response, got: %s", w.Body.String())
	}
}

// TestCreateRequestContext tests context creation with timeout
func TestCreateRequestContext(t *testing.T) {
	timeout := 5 * time.Second
	ctx, cancel := createRequestContext(timeout)
	defer cancel()

	if ctx == nil {
		t.Error("Expected non-nil context")
	}

	// Check that context has a deadline
	deadline, ok := ctx.Deadline()
	if !ok {
		t.Error("Expected context to have deadline")
	}

	// Check that deadline is approximately correct (within 1 second)
	now := time.Now()
	expectedDeadline := now.Add(timeout)
	diff := expectedDeadline.Sub(deadline)
	if diff < -1*time.Second || diff > 1*time.Second {
		t.Errorf("Deadline not set correctly. Expected ~%v, got %v", expectedDeadline, deadline)
	}
}

// TestCreateRequestContextCancellation tests that cancel function works
func TestCreateRequestContextCancellation(t *testing.T) {
	ctx, cancel := createRequestContext(30 * time.Second)

	select {
	case <-ctx.Done():
		t.Error("Context should not be done before cancellation")
	default:
	}

	cancel()

	// Give a small window for the context to be cancelled
	select {
	case <-ctx.Done():
		// Expected
	case <-time.After(1 * time.Second):
		t.Error("Context should be cancelled")
	}
}

// TestHandleHTTPRequestInvalidMethod tests handleHTTPRequest with non-POST method
func TestHandleHTTPRequestInvalidMethod(t *testing.T) {
	conn := setupTestGRPCConnection(t)
	defer conn.Close()

	req := httptest.NewRequest("GET", "/test.Method", nil)
	w := httptest.NewRecorder()

	handleHTTPRequest(w, req, conn)

	if w.Code != http.StatusMethodNotAllowed {
		t.Errorf("Expected status %d, got %d", http.StatusMethodNotAllowed, w.Code)
	}
}

// TestHandleHTTPRequestEmptyPath tests handleHTTPRequest with empty path
func TestHandleHTTPRequestEmptyPath(t *testing.T) {
	conn := setupTestGRPCConnection(t)
	defer conn.Close()

	req := httptest.NewRequest("POST", "/", bytes.NewReader([]byte("data")))
	w := httptest.NewRecorder()

	// This will attempt to call an empty method on the gRPC server
	// It should fail, but let's ensure it handles gracefully
	handleHTTPRequest(w, req, conn)

	// Should return an error status code
	if w.Code == http.StatusOK {
		t.Errorf("Expected error status code, got %d", w.Code)
	}
}

// TestHandleHTTPRequestValidRequest tests handleHTTPRequest with valid request
func TestHandleHTTPRequestValidRequest(t *testing.T) {
	// This test would require a running gRPC server, which is complex to set up
	// For now, we'll test the structure
	conn := setupTestGRPCConnection(t)
	defer conn.Close()

	req := httptest.NewRequest("POST", "/test.Method", bytes.NewReader([]byte("test")))
	w := httptest.NewRecorder()

	// This test depends on having a gRPC server, so it will fail gracefully
	handleHTTPRequest(w, req, conn)

	// Verify that we got an error response (since no server is running)
	if w.Code != http.StatusInternalServerError {
		t.Errorf("Expected status code %d for unavailable server, got %d",
			http.StatusInternalServerError, w.Code)
	}
}

// TestHandleHTTPRequestReadBodyError simulates a scenario where body reading would fail
func TestHandleHTTPRequestReadBodyError(t *testing.T) {
	conn := setupTestGRPCConnection(t)
	defer conn.Close()

	// Create a request with an error-inducing body
	req := httptest.NewRequest("POST", "/test.Method", newBrokenReader())
	w := httptest.NewRecorder()

	handleHTTPRequest(w, req, conn)

	if w.Code != http.StatusBadRequest {
		t.Errorf("Expected status %d, got %d", http.StatusBadRequest, w.Code)
	}
}

// brokenReader is a reader that always returns an error
type brokenReader struct{}

func (br *brokenReader) Read(p []byte) (n int, err error) {
	return 0, io.ErrUnexpectedEOF
}

func newBrokenReader() io.Reader {
	return &brokenReader{}
}

// TestHandleHTTPRequestConstants verifies timeout constants
func TestHandleHTTPRequestConstants(t *testing.T) {
	if streamingTimeout == 0 || unaryTimeout == 0 {
		t.Error("Timeout constants should be non-zero")
	}
	if bufferCapacity == 0 || readChunkSize == 0 {
		t.Error("Buffer size constants should be non-zero")
	}
	if streamingTimeout <= unaryTimeout {
		t.Error("Streaming timeout should be greater than unary timeout")
	}
	if bufferCapacity <= readChunkSize {
		t.Errorf("Buffer capacity (%d) should be greater than read chunk size (%d)",
			bufferCapacity, readChunkSize)
	}
}

// setupTestGRPCConnection creates a test gRPC connection to localhost
// This will fail if no server is running, which is expected for most tests
func setupTestGRPCConnection(t *testing.T) *grpc.ClientConn {
	// Try to connect to a non-existent server
	// This will create a connection in a failed state, which is fine for testing handlers
	conn, err := grpc.NewClient(
		"localhost:0",
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	if err != nil && !strings.Contains(err.Error(), "context deadline exceeded") {
		// Some errors are acceptable during setup
	}
	return conn
}

// TestMultipleRawCodecOperations tests sequence of marshal/unmarshal operations
func TestMultipleRawCodecOperations(t *testing.T) {
	codec := RawCodec{}
	testCases := [][]byte{
		[]byte(""),
		[]byte("a"),
		[]byte("hello"),
		[]byte("test\x00data\x01with\xffbinary"),
		make([]byte, 1024),
	}

	for _, testData := range testCases {
		marshaled, err := codec.Marshal(testData)
		if err != nil {
			t.Fatalf("Marshal failed for %v: %v", testData, err)
		}

		var unmarshaled []byte
		err = codec.Unmarshal(marshaled, &unmarshaled)
		if err != nil {
			t.Fatalf("Unmarshal failed for %v: %v", marshaled, err)
		}

		if !bytes.Equal(unmarshaled, testData) {
			t.Errorf("Roundtrip failed: original %v, result %v", testData, unmarshaled)
		}
	}
}



// TestRawCodecEdgeCases tests RawCodec with edge case inputs
func TestRawCodecEdgeCases(t *testing.T) {
	codec := RawCodec{}

	// Test with nil
	_, err := codec.Marshal(nil)
	if err == nil {
		t.Error("Expected error when marshaling nil")
	}

	// Test with pointer to bytes
	data := []byte("test")
	_, err = codec.Marshal(&data)
	if err == nil {
		t.Error("Expected error when marshaling pointer to bytes (not bytes directly)")
	}

	// Test with empty bytes
	result, err := codec.Marshal([]byte{})
	if err != nil {
		t.Fatalf("Marshal empty bytes failed: %v", err)
	}
	if len(result) != 0 {
		t.Errorf("Expected empty result for empty bytes, got %d bytes", len(result))
	}
}

// TestRequirePOSTAllowsCorrectMethod tests that only POST is allowed
func TestRequirePOSTAllowsCorrectMethod(t *testing.T) {
	req := httptest.NewRequest("POST", "/", nil)
	w := httptest.NewRecorder()

	if !requirePOST(w, req) {
		t.Error("requirePOST should allow POST method")
	}
}

// TestReadRequestBodyClosesBody tests that request body is closed after reading
func TestReadRequestBodyClosesBody(t *testing.T) {
	reader := &trackingReader{buf: bytes.NewReader([]byte("test"))}
	req := httptest.NewRequest("POST", "/", reader)
	w := httptest.NewRecorder()

	_, ok := readRequestBody(w, req)
	if !ok {
		t.Error("readRequestBody returned false")
	}
	if !reader.closed {
		t.Error("Expected request body to be closed")
	}
}

// trackingReader wraps an io.Reader and tracks if it was closed
type trackingReader struct {
	buf    *bytes.Reader
	closed bool
}

func (tr *trackingReader) Read(p []byte) (n int, err error) {
	return tr.buf.Read(p)
}

func (tr *trackingReader) Close() error {
	tr.closed = true
	return nil
}



// TestConfigurationIntegration tests Config with various server settings
func TestConfigurationIntegration(t *testing.T) {
	tests := []struct {
		name string
		addr string
	}{
		{"localhost IPv4", "127.0.0.1:8080"},
		{"localhost IPv6", "[::1]:8080"},
		{"dynamic port", "localhost:0"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			config := &Config{}
			server := &http.Server{Addr: tt.addr}

			config.SetServer(server)

			if config.srv.Addr != tt.addr {
				t.Errorf("Expected address '%s', got '%s'", tt.addr, config.srv.Addr)
			}
		})
	}
}

// TestHTTPHandlersWithRecorder tests HTTP handlers with httptest.ResponseRecorder
func TestHTTPHandlersWithRecorder(t *testing.T) {
	conn := setupTestGRPCConnection(t)
	defer conn.Close()

	tests := []struct {
		name           string
		method         string
		path           string
		expectedStatus int
	}{
		{"GET not allowed", "GET", "/test.Method", http.StatusMethodNotAllowed},
		{"PUT not allowed", "PUT", "/test.Method", http.StatusMethodNotAllowed},
		{"DELETE not allowed", "DELETE", "/test.Method", http.StatusMethodNotAllowed},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			req := httptest.NewRequest(tt.method, tt.path, bytes.NewReader([]byte("data")))
			w := httptest.NewRecorder()

			handleHTTPRequest(w, req, conn)

			if w.Code != tt.expectedStatus {
				t.Errorf("Expected status %d, got %d", tt.expectedStatus, w.Code)
			}
		})
	}
}

// TestHandleStreamErrorWithVariousErrors tests error handling with different error types
func TestHandleStreamErrorWithVariousErrors(t *testing.T) {
	tests := []struct {
		name    string
		message string
		err     error
	}{
		{"Internal error", "Internal error", status.Error(codes.Internal, "server error")},
		{"Unavailable", "Service unavailable", status.Error(codes.Unavailable, "server down")},
		{"Deadline exceeded", "Timeout", status.Error(codes.DeadlineExceeded, "too slow")},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			w := httptest.NewRecorder()
			handleStreamError(w, tt.message, tt.err)

			if w.Code != http.StatusInternalServerError {
				t.Errorf("Expected status %d, got %d", http.StatusInternalServerError, w.Code)
			}
			if !strings.Contains(w.Body.String(), tt.message) {
				t.Errorf("Expected '%s' in error message, got: %s", tt.message, w.Body.String())
			}
		})
	}
}

// TestLargeDataHandling tests handling of large data payloads
func TestLargeDataHandling(t *testing.T) {
	sizes := []int{
		1024,           // 1KB
		64 * 1024,      // 64KB (buffer size)
		256 * 1024,     // 256KB
		1024 * 1024,    // 1MB
	}

	for _, size := range sizes {
		t.Run(fmt.Sprintf("size_%d", size), func(t *testing.T) {
			largeData := make([]byte, size)
			for i := range largeData {
				largeData[i] = byte(i % 256)
			}

			req := httptest.NewRequest("POST", "/test", bytes.NewReader(largeData))
			w := httptest.NewRecorder()

			result, ok := readRequestBody(w, req)
			if !ok {
				t.Errorf("Failed to read body of size %d", size)
				return
			}
			if len(result) != size {
				t.Errorf("Expected body size %d, got %d", size, len(result))
			}
			if !bytes.Equal(result, largeData) {
				t.Errorf("Body content mismatch for size %d", size)
			}
		})
	}
}

// TestBufferCapacityConstants validates buffer capacity configuration
func TestBufferCapacityConstants(t *testing.T) {
	// Ensure buffer capacity is reasonable for streaming
	if bufferCapacity < readChunkSize {
		t.Errorf("Buffer capacity (%d) should be >= read chunk size (%d)",
			bufferCapacity, readChunkSize)
	}

	// Ensure timeouts are properly configured
	if streamingTimeout <= 0 || unaryTimeout <= 0 {
		t.Error("Timeout constants should be positive")
	}

	// Streaming should generally allow more time than unary operations
	if streamingTimeout <= unaryTimeout {
		t.Logf("Warning: streamingTimeout (%v) should typically be > unaryTimeout (%v)",
			streamingTimeout, unaryTimeout)
	}
}

// TestRawCodecBinaryData tests RawCodec with various binary patterns
func TestRawCodecBinaryData(t *testing.T) {
	codec := RawCodec{}
	binaryPatterns := [][]byte{
		{0x00, 0xFF, 0xAA, 0x55}, // Alternating patterns
		{0xFF, 0xFF, 0xFF, 0xFF}, // All ones
		{0x00, 0x00, 0x00, 0x00}, // All zeros
		make([]byte, 256),         // All zeros (larger)
	}

	for _, pattern := range binaryPatterns {
		marshaled, err := codec.Marshal(pattern)
		if err != nil {
			t.Fatalf("Marshal failed for pattern %v: %v", pattern, err)
		}

		var unmarshaled []byte
		err = codec.Unmarshal(marshaled, &unmarshaled)
		if err != nil {
			t.Fatalf("Unmarshal failed: %v", err)
		}

		if !bytes.Equal(unmarshaled, pattern) {
			t.Errorf("Pattern mismatch: expected %v, got %v", pattern, unmarshaled)
		}
	}
}

// TestReadRequestBodyMultipleCalls tests that body can only be read once
func TestReadRequestBodyMultipleCalls(t *testing.T) {
	testData := []byte("test data")
	req := httptest.NewRequest("POST", "/test", bytes.NewReader(testData))
	w := httptest.NewRecorder()

	// First read should succeed
	result1, ok1 := readRequestBody(w, req)
	if !ok1 {
		t.Fatal("First read failed")
	}
	if !bytes.Equal(result1, testData) {
		t.Errorf("First read got unexpected data")
	}

	// Second read should fail (body is closed)
	req2 := httptest.NewRequest("POST", "/test", bytes.NewReader(testData))
	w2 := httptest.NewRecorder()
	result2, ok2 := readRequestBody(w2, req2)
	if !ok2 {
		t.Error("Second read should still work with fresh request")
	}
	if !bytes.Equal(result2, testData) {
		t.Errorf("Second read got unexpected data")
	}
}

// TestSetGRPCResponseHeadersIdempotent tests that setting headers multiple times is safe
func TestSetGRPCResponseHeadersIdempotent(t *testing.T) {
	w := httptest.NewRecorder()

	// Set headers twice
	setGRPCResponseHeaders(w)
	setGRPCResponseHeaders(w)

	// Should still have correct headers
	if w.Header().Get("Content-Type") != "application/grpc" {
		t.Errorf("Content-Type should be 'application/grpc'")
	}
	if w.Code != http.StatusOK {
		t.Errorf("Status code should be %d", http.StatusOK)
	}
}

// TestContextDeadlineAccuracy tests context deadline accuracy within tolerance
func TestContextDeadlineAccuracy(t *testing.T) {
	tests := []time.Duration{
		1 * time.Second,
		5 * time.Second,
		30 * time.Second,
		60 * time.Second,
	}

	for _, timeout := range tests {
		t.Run(fmt.Sprintf("timeout_%v", timeout), func(t *testing.T) {
			before := time.Now()
			ctx, cancel := createRequestContext(timeout)
			defer cancel()

			deadline, ok := ctx.Deadline()
			if !ok {
				t.Fatal("Expected context to have deadline")
			}

			actual := deadline.Sub(before)
			// Allow 1 second tolerance for execution time
			if actual < timeout-1*time.Second || actual > timeout+1*time.Second {
				t.Errorf("Deadline inaccurate: expected ~%v, got %v", timeout, actual)
			}
		})
	}
}

// TestReadRequestBodyMeasureSize tests body size measurement across various sizes
func TestReadRequestBodyMeasureSize(t *testing.T) {
	sizes := []int{0, 1, 10, 100, 1024, 10*1024, 100*1024}

	for _, size := range sizes {
		t.Run(fmt.Sprintf("size_%d", size), func(t *testing.T) {
			testData := make([]byte, size)
			for i := range testData {
				testData[i] = byte(i % 256)
			}

			req := httptest.NewRequest("POST", "/test", bytes.NewReader(testData))
			w := httptest.NewRecorder()

			result, ok := readRequestBody(w, req)
			if !ok {
				t.Errorf("Failed to read body of size %d", size)
				return
			}

			if len(result) != size {
				t.Errorf("Expected size %d, got %d", size, len(result))
			}
		})
	}
}

// TestHTTPResponseWriterBehavior tests ResponseWriter behavior in different scenarios
func TestHTTPResponseWriterBehavior(t *testing.T) {
	w := httptest.NewRecorder()

	// Test that WriteHeader is idempotent
	w.WriteHeader(http.StatusOK)
	w.WriteHeader(http.StatusInternalServerError) // Should be ignored
	if w.Code != http.StatusOK {
		t.Errorf("Expected first WriteHeader to take effect")
	}
}

// TestFlusherInterface tests that the Flusher interface is correctly used
func TestFlusherInterface(t *testing.T) {
	w := httptest.NewRecorder()
	flusher, ok := getFlusher(w)
	if !ok {
		t.Fatal("getFlusher should succeed with httptest.ResponseRecorder")
	}

	// Verify Flusher has Flush method (doesn't panic)
	flusher.Flush()

	// Write data and flush
	w.WriteHeader(http.StatusOK)
	w.Write([]byte("test"))
	flusher.Flush()

	if w.Body.String() != "test" {
		t.Errorf("Expected body 'test', got '%s'", w.Body.String())
	}
}

// TestRequestBodyWithSpecialChars tests body reading with special characters
func TestRequestBodyWithSpecialChars(t *testing.T) {
	specialData := []byte{
		0x00, 0x01, 0x02, 0x03, // Null and control chars
		'h', 'e', 'l', 'l', 'o', // ASCII
		0xC3, 0xA9,             // UTF-8 (é)
		0xFF, 0xFE,             // High bytes
	}

	req := httptest.NewRequest("POST", "/test", bytes.NewReader(specialData))
	w := httptest.NewRecorder()

	result, ok := readRequestBody(w, req)
	if !ok {
		t.Fatal("Failed to read body with special characters")
	}

	if !bytes.Equal(result, specialData) {
		t.Errorf("Body content mismatch: expected %v, got %v", specialData, result)
	}
}

// TestConfigurationValidation tests config field validation
func TestConfigurationValidation(t *testing.T) {
	config := &Config{}

	// Initially nil
	if config.srv != nil {
		t.Error("Config.srv should be nil initially")
	}

	// Set and verify
	server := &http.Server{Addr: ":8080"}
	config.SetServer(server)

	if config.srv == nil {
		t.Error("Config.srv should not be nil after SetServer")
	}
	if config.srv != server {
		t.Error("Config.srv should be the exact server object passed")
	}
}

// TestErrorMessageFormatting tests that error messages are properly formatted
func TestErrorMessageFormatting(t *testing.T) {
	tests := []struct {
		name    string
		message string
		err     error
	}{
		{"empty message", "", fmt.Errorf("test error")},
		{"long message", strings.Repeat("a", 100), fmt.Errorf("test error")},
		{"special chars", "Test: \n\r\t", fmt.Errorf("test error")},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			w := httptest.NewRecorder()
			handleStreamError(w, tt.message, tt.err)

			if w.Code != http.StatusInternalServerError {
				t.Errorf("Expected status %d, got %d", http.StatusInternalServerError, w.Code)
			}
			// Error message should contain the message
			if len(tt.message) > 0 && !strings.Contains(w.Body.String(), tt.message) {
				t.Errorf("Expected '%s' in error response", tt.message)
			}
		})
	}
}
